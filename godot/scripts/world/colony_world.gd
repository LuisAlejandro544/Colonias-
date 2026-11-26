# Orquestador modular de la escena jugable: Mundo 2D terrestre en vista centinela.
#
# Lógica: Conecta Godot con los nodos nativos Rust (`WorldGenerator` y
# `ColonySimulation`), gestiona la pantalla de carga inicial y delega las
# responsabilidades especializadas a submódulos dedicados:
# - `GroundCanvas`: renderizado del suelo visible.
# - `WorldOverlay`: zonas, cultivos y previsualización de arrastre.
# - `WorldEntitiesRenderer`: props, muros conectables, ítems y colonos.
# - `WorldCameraInput`: paneo táctil, pinch-zoom y selección rectangular.
# - `WorldSfx`: pool de efectos de sonido `.ogg`.
# - `SaveBridge`: normalización y empaquetado de guardado JSON.
# - `WorldHud`: interfaz superpuesta y popups modulares.

extends Node2D

const TILE_PX := 64.0

const WorldHudScript := preload("res://scripts/world/world_hud.gd")
const GroundCanvasScript := preload("res://scripts/world/ground_canvas.gd")
const WorldOverlayScript := preload("res://scripts/world/world_overlay.gd")
const WorldEntitiesRendererScript := preload("res://scripts/world/world_entities_renderer.gd")
const WorldCameraInputScript := preload("res://scripts/world/world_camera_input.gd")
const WorldSfxScript := preload("res://scripts/world/world_sfx.gd")
const SaveBridgeScript := preload("res://scripts/world/save_bridge.gd")

var _world_gen: Node
var _colony_sim: Node

var _map_width: int = 64
var _map_height: int = 64
var _ground_tiles: PackedInt32Array
var _prop_tiles: PackedInt32Array
var _blocked_tiles: PackedInt32Array

var _ground_canvas: Node2D
var _world_overlay: Node2D
var _entities_renderer
var _camera: Camera2D
var _camera_input: Node
var _sfx: Node
var _hud: CanvasLayer

var _loading_layer: CanvasLayer
var _loading_title: Label
var _loading_status: Label
var _loading_bar: ProgressBar
var _error_back_btn: Button

var _selected_colonist_id: int = -1
var _follow_selected: bool = false
var _sim_speed: float = 1.0
var _world_ready: bool = false


func _ready() -> void:
	start_colony_world()


func start_colony_world() -> void:
	if _loading_layer != null:
		return
	_build_loading_overlay()

	if not ClassDB.class_exists(&"WorldGenerator") or not ClassDB.class_exists(&"ColonySimulation"):
		_show_core_error(
			"No se pudo inicializar el núcleo nativo Rust (`colonia_core`).\n" +
			"Verifica que el APK incluya `libcolonia_core.so` para la arquitectura de tu dispositivo."
		)
		return

	_world_gen = ClassDB.instantiate(&"WorldGenerator") as Node
	_colony_sim = ClassDB.instantiate(&"ColonySimulation") as Node
	if _world_gen == null or _colony_sim == null:
		_show_core_error("Error al instanciar `WorldGenerator` o `ColonySimulation` desde Rust.")
		return

	add_child(_world_gen)
	add_child(_colony_sim)

	_camera = Camera2D.new()
	_camera.position_smoothing_enabled = true
	_camera.position_smoothing_speed = 9.0
	_camera.zoom = Vector2(1.0, 1.0)
	add_child(_camera)
	_camera.make_current()

	_ground_canvas = GroundCanvasScript.new() as Node2D
	add_child(_ground_canvas)

	_world_overlay = WorldOverlayScript.new() as Node2D
	add_child(_world_overlay)

	_entities_renderer = WorldEntitiesRendererScript.new()
	add_child(_entities_renderer)

	_sfx = WorldSfxScript.new() as Node
	add_child(_sfx)

	_camera_input = WorldCameraInputScript.new() as Node
	add_child(_camera_input)
	_camera_input.connect("area_committed", _on_area_committed)
	_camera_input.connect("world_tapped", _handle_world_tap)
	_camera_input.connect("follow_cancelled", func(): _follow_selected = false)

	_hud = WorldHudScript.new() as CanvasLayer
	add_child(_hud)
	_hud.connect("speed_changed", func(spd: float): _sim_speed = spd)
	_hud.connect("zoom_step_requested", func(dz: float): _camera_input.call("apply_zoom_delta", dz))
	_hud.connect("save_requested", _save_current_colony)
	_hud.connect("follow_toggled", func(enabled: bool): _follow_selected = enabled)
	_hud.connect("tool_mode_changed", func(mode_code: String): _camera_input.call("set_tool_mode", mode_code))
	_hud.connect("jobs_opened", _on_jobs_opened)
	_hud.connect("role_toggle_requested", _on_role_toggle_requested)

	call_deferred("_initialize_world_async")


func _on_jobs_opened() -> void:
	if _hud == null or _colony_sim == null:
		return
	var nodes: Array = _entities_renderer.get_colonist_nodes_array()
	_hud.call("open_jobs_popup", nodes, _selected_colonist_id, _colony_sim)


func _on_role_toggle_requested(colonist_id: int, role_code: String, enabled: bool) -> void:
	if _colony_sim == null or _hud == null:
		return
	var res_var: Variant = _colony_sim.call("set_colonist_role_enabled", colonist_id, role_code, enabled)
	var ok := true
	var msg := ""
	if res_var is Dictionary:
		var res: Dictionary = res_var
		ok = bool(res.get("ok", false))
		msg = str(res.get("message", ""))
	if ok:
		_sync_world_snapshot(false)
		_sync_colonist_visuals()
	var nodes: Array = _entities_renderer.get_colonist_nodes_array()
	_hud.call("refresh_jobs_popup", nodes, _colony_sim, msg, not ok)


func _build_loading_overlay() -> void:
	_loading_layer = CanvasLayer.new()
	_loading_layer.layer = 20
	add_child(_loading_layer)

	var bg := ColorRect.new()
	bg.color = UiTheme.BG_DARK
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_loading_layer.add_child(bg)

	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_loading_layer.add_child(center)

	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(680, 0)
	UiTheme.apply_panel_style(panel, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 36, 32)
	center.add_child(panel)

	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 16)
	panel.add_child(v)

	_loading_title = Label.new()
	_loading_title.text = "Generando mundo: %s" % GameSession.colony_name
	_loading_title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_loading_title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_title(_loading_title, 32)
	v.add_child(_loading_title)

	_loading_bar = ProgressBar.new()
	_loading_bar.min_value = 0.0
	_loading_bar.max_value = 100.0
	_loading_bar.value = 35.0
	_loading_bar.show_percentage = false
	_loading_bar.custom_minimum_size = Vector2(0, 30)

	var bar_bg := StyleBoxFlat.new()
	bar_bg.bg_color = Color("#1b1510")
	bar_bg.border_color = UiTheme.BORDER_BRASS
	bar_bg.set_border_width_all(2)
	bar_bg.set_corner_radius_all(8)
	_loading_bar.add_theme_stylebox_override("background", bar_bg)

	var bar_fill := StyleBoxFlat.new()
	bar_fill.bg_color = UiTheme.BTN_PRIMARY_HOVER
	bar_fill.set_corner_radius_all(6)
	_loading_bar.add_theme_stylebox_override("fill", bar_fill)
	v.add_child(_loading_bar)

	_loading_status = Label.new()
	_loading_status.text = "Explorando terreno terrestre..."
	_loading_status.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_loading_status.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(_loading_status, 19)
	v.add_child(_loading_status)

	_error_back_btn = Button.new()
	_error_back_btn.text = "Volver al Menú Principal"
	_error_back_btn.visible = false
	UiTheme.style_button(_error_back_btn, "danger", 56, 20)
	_error_back_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		get_tree().change_scene_to_file("res://scenes/ui/main_menu.tscn")
	)
	v.add_child(_error_back_btn)


func _show_core_error(msg: String) -> void:
	_loading_title.text = "No se pudo abrir la colonia"
	_loading_bar.visible = false
	_loading_status.text = msg
	_loading_status.add_theme_color_override("font_color", Color("#e58a76"))
	_error_back_btn.visible = true


func _on_props_progress(pct: float) -> void:
	_loading_bar.value = pct
	_loading_status.text = "Colocando bosques y rocas (%d%%)..." % int(round(pct))


func _initialize_world_async() -> void:
	await get_tree().process_frame

	var data_var: Variant = _world_gen.call(
		"generate_world",
		GameSession.seed_value,
		GameSession.map_size,
		GameSession.map_size,
		GameSession.biome_code
	)
	if not (data_var is Dictionary):
		_show_core_error("El núcleo nativo no devolvió datos válidos del terreno.")
		return

	var data: Dictionary = data_var
	_map_width = int(data.get("width", GameSession.map_size))
	_map_height = int(data.get("height", GameSession.map_size))
	_ground_tiles = data.get("ground", PackedInt32Array())
	_prop_tiles = data.get("props", PackedInt32Array())
	_blocked_tiles = data.get("blocked", PackedInt32Array())

	if _ground_tiles.is_empty():
		_show_core_error("El generador procedural devolvió un terreno vacío.")
		return

	if not GameSession.saved_world_state.is_empty():
		var saved_props = GameSession.saved_world_state.get("props", [])
		if saved_props is Array and saved_props.size() == _prop_tiles.size():
			for i in range(saved_props.size()):
				_prop_tiles[i] = int(saved_props[i])
		var saved_blocked = GameSession.saved_world_state.get("blocked", [])
		if saved_blocked is Array and saved_blocked.size() == _blocked_tiles.size():
			for i in range(saved_blocked.size()):
				_blocked_tiles[i] = int(saved_blocked[i])

	_loading_bar.value = 45.0
	_loading_status.text = "Preparando campamento terrestre..."
	await get_tree().process_frame

	_ground_canvas.call("configure", _map_width, _map_height, _ground_tiles, _camera)
	_world_overlay.call("configure", _map_width, _map_height, _camera)
	_camera_input.call("configure", _map_width, _map_height, _camera, _world_overlay)

	_camera.limit_left = 0
	_camera.limit_top = 0
	_camera.limit_right = int(_map_width * TILE_PX)
	_camera.limit_bottom = int(_map_height * TILE_PX)
	_camera.position = Vector2(_map_width * TILE_PX * 0.5, _map_height * TILE_PX * 0.5)
	_camera_input.call("clamp_camera_to_bounds")
	_camera.reset_smoothing()

	await _entities_renderer.spawn_world_props_async(
		_map_width,
		_map_height,
		_prop_tiles,
		GameSession.seed_value,
		Callable(self, "_on_props_progress")
	)

	_loading_bar.value = 94.0
	_loading_status.text = "Eligiendo líder y desplegando colonos..."
	await get_tree().process_frame

	if GameSession.saved_world_state.is_empty():
		_colony_sim.call(
			"setup_colony",
			GameSession.seed_value,
			_map_width,
			_map_height,
			_ground_tiles,
			_prop_tiles,
			_blocked_tiles,
			GameSession.initial_colonists
		)
	else:
		var state_copy: Dictionary = SaveBridgeScript.convert_save_dict_for_rust(GameSession.saved_world_state)
		_colony_sim.call(
			"restore_colony",
			GameSession.seed_value,
			_map_width,
			_map_height,
			_ground_tiles,
			_prop_tiles,
			_blocked_tiles,
			state_copy
		)

	_sync_world_snapshot(true)
	_sync_colonist_visuals()
	_ground_canvas.queue_redraw()
	_world_overlay.queue_redraw()
	_loading_bar.value = 100.0
	_world_ready = true
	_camera_input.call("set_enabled", true)
	_camera_input.call("clamp_camera_to_bounds")
	_loading_layer.visible = false


func _process(delta: float) -> void:
	if not _world_ready or _colony_sim == null:
		return
	if _sim_speed > 0.0:
		_colony_sim.call("step_simulation", delta * _sim_speed)
	_sync_world_snapshot(false)
	_sync_colonist_visuals()
	_ground_canvas.queue_redraw()
	_world_overlay.queue_redraw()

	if _follow_selected and _selected_colonist_id > 0:
		var target := _entities_renderer.get_colonist_node(_selected_colonist_id) as Node2D
		if target != null:
			_camera.position = target.position
			_camera_input.call("clamp_camera_to_bounds")


func _sync_world_snapshot(force_full: bool) -> void:
	var snap_var: Variant = _colony_sim.call("get_world_snapshot", force_full)
	if not (snap_var is Dictionary):
		return
	var snap: Dictionary = snap_var
	var col_count: int = _entities_renderer.get_colonist_count()
	_hud.call("update_world_summary", snap, col_count if col_count > 0 else GameSession.initial_colonists)
	_sfx.call("play_events", snap.get("sfx_events", []))

	if not bool(snap.get("dirty", false)):
		return

	_entities_renderer.sync_world_entities(_map_width, _map_height, snap)

	var zones: PackedInt32Array = snap.get("zones", PackedInt32Array())
	var crops: Array = snap.get("crops", [])
	var orders: Array = snap.get("orders", [])
	_world_overlay.call("update_world_data", zones, crops, orders)
	_hud.call("refresh_carpenter_bench_if_visible")


func _sync_colonist_visuals() -> void:
	var snapshots_var: Variant = _colony_sim.call("get_colonists_snapshot")
	if not (snapshots_var is Array):
		return
	_entities_renderer.sync_colonists(snapshots_var as Array)
	if _selected_colonist_id > 0:
		var selected_node := _entities_renderer.get_colonist_node(_selected_colonist_id) as Node2D
		if selected_node != null:
			_hud.call("show_colonist_info", selected_node)


func _on_area_committed(mode_code: String, start_tile: Vector2i, end_tile: Vector2i) -> void:
	var affected := int(_colony_sim.call(
		"designate_area",
		mode_code,
		start_tile.x,
		start_tile.y,
		end_tile.x,
		end_tile.y
	))
	if affected > 0:
		UiTheme.trigger_touch_feedback()
		if mode_code == "cancel":
			_hud.call("show_toast", "Canceladas %d casillas" % affected)
		else:
			_hud.call("show_toast", "%d casillas marcadas • El líder reunirá al equipo" % affected)
	else:
		_hud.call("show_toast", "No hay objetivos válidos en esa selección")


func _handle_world_tap(screen_pos: Vector2) -> void:
	var world_pos: Vector2 = _camera_input.call("screen_to_world", screen_pos)
	var best_id: int = _entities_renderer.find_colonist_at_world_pos(world_pos, 54.0)
	var selected_node := _entities_renderer.set_selected_colonist(best_id) as Node2D
	_selected_colonist_id = best_id
	if best_id > 0 and selected_node != null:
		UiTheme.trigger_touch_feedback()
		_hud.call("show_colonist_info", selected_node)
	else:
		_follow_selected = false
		_hud.call("show_colonist_info", null)
		var tile_x := int(floor(world_pos.x / TILE_PX))
		var tile_y := int(floor(world_pos.y / TILE_PX))
		if bool(_hud.call("open_carpenter_bench", tile_x, tile_y, _colony_sim)):
			UiTheme.trigger_touch_feedback()


func _save_current_colony() -> void:
	if _colony_sim == null:
		return
	var state_var: Variant = _colony_sim.call("export_save_state")
	var state: Dictionary = state_var if state_var is Dictionary else {}
	var payload: Dictionary = SaveBridgeScript.build_save_payload(state)
	if SaveManager.save_game(GameSession.active_slot, payload):
		_hud.call("show_toast", "Colonia guardada en Ranura %d" % GameSession.active_slot)
	else:
		_hud.call("show_toast", "No se pudo guardar en Ranura %d" % GameSession.active_slot)
