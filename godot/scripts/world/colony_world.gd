# Escena principal jugable: Mundo 2D terrestre en vista centinela (estilo RimWorld).
#
# Lógica: Carga módulos mediante `preload` directo (sin depender de caché global
# de `class_name`) y se enlaza con los nodos nativos Rust (`WorldGenerator` y
# `ColonySimulation`). Muestra un overlay de progreso mientras genera el terreno
# y distribuye árboles, arbolitos, rocas y piedras por lotes sin congelar la
# pantalla, además de gestionar la cámara táctil y el guardado de la colonia.

extends Node2D

const TILE_PX := 64.0
const MIN_ZOOM := 0.55
const MAX_ZOOM := 1.85
const PROPS_PER_FRAME := 240

const WorldHudScript := preload("res://scripts/world/world_hud.gd")
const ColonistVisualScript := preload("res://scripts/world/colonist_visual.gd")
const GroundCanvasScript := preload("res://scripts/world/ground_canvas.gd")

var _world_gen: Node
var _colony_sim: Node

var _map_width: int = 64
var _map_height: int = 64
var _ground_tiles: PackedInt32Array
var _blocked_tiles: PackedInt32Array

var _ground_canvas: Node2D
var _entities_layer: Node2D
var _camera: Camera2D
var _hud: CanvasLayer

var _loading_layer: CanvasLayer
var _loading_title: Label
var _loading_status: Label
var _loading_bar: ProgressBar
var _error_back_btn: Button

var _colonist_nodes: Dictionary = {}
var _selected_colonist_id: int = -1
var _follow_selected: bool = false
var _sim_speed: float = 1.0
var _world_ready: bool = false

# Control táctil móvil (arrastre, detección de toque y pellizco)
var _active_touches: Dictionary = {}
var _dragging: bool = false
var _drag_moved: bool = false
var _press_screen_pos: Vector2 = Vector2.ZERO
var _last_pinch_dist: float = 0.0


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

	_entities_layer = Node2D.new()
	_entities_layer.y_sort_enabled = true
	add_child(_entities_layer)

	_hud = WorldHudScript.new() as CanvasLayer
	add_child(_hud)
	_hud.connect("speed_changed", func(spd: float): _sim_speed = spd)
	_hud.connect("zoom_step_requested", _apply_zoom_delta)
	_hud.connect("save_requested", _save_current_colony)
	_hud.connect("follow_toggled", func(enabled: bool): _follow_selected = enabled)

	call_deferred("_initialize_world_async")


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
	_loading_status.text = "Calculando bioma y elevación en núcleo Rust..."
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
	var props: PackedInt32Array = data.get("props", PackedInt32Array())
	_blocked_tiles = data.get("blocked", PackedInt32Array())

	if _ground_tiles.is_empty():
		_show_core_error("El generador procedural devolvió un terreno vacío.")
		return

	_loading_bar.value = 45.0
	_loading_status.text = "Preparando suelo terrestre (%dx%d)..." % [_map_width, _map_height]
	await get_tree().process_frame

	_ground_canvas.call("configure", _map_width, _map_height, _ground_tiles, _camera)
	_camera.limit_left = 0
	_camera.limit_top = 0
	_camera.limit_right = int(_map_width * TILE_PX)
	_camera.limit_bottom = int(_map_height * TILE_PX)
	_camera.position = Vector2(_map_width * TILE_PX * 0.5, _map_height * TILE_PX * 0.5)

	await _spawn_world_props_async(props)

	_loading_bar.value = 94.0
	_loading_status.text = "Desplegando colonos en el claro central..."
	await get_tree().process_frame

	if GameSession.saved_colonists.is_empty():
		_colony_sim.call(
			"setup_colony",
			GameSession.seed_value,
			_map_width,
			_map_height,
			_blocked_tiles,
			GameSession.initial_colonists
		)
	else:
		var arr := Array(GameSession.saved_colonists)
		_colony_sim.call(
			"restore_colonists",
			GameSession.seed_value,
			_map_width,
			_map_height,
			_blocked_tiles,
			arr
		)

	_sync_colonist_visuals()
	_ground_canvas.queue_redraw()
	_loading_bar.value = 100.0
	_world_ready = true
	_loading_layer.visible = false


func _spawn_world_props_async(props: PackedInt32Array) -> void:
	var tex_oak := load("res://assets/props/tree_oak.svg") as Texture2D
	var tex_pine := load("res://assets/props/tree_pine.svg") as Texture2D
	var tex_sapling := load("res://assets/props/sapling.svg") as Texture2D
	var tex_granite := load("res://assets/props/boulder_granite.svg") as Texture2D
	var tex_iron := load("res://assets/props/boulder_iron.svg") as Texture2D
	var tex_stones := load("res://assets/props/stone_pebbles.svg") as Texture2D

	var rng := RandomNumberGenerator.new()
	rng.seed = GameSession.seed_value ^ 0x4455

	var spawned_in_batch := 0
	var total_tiles := _map_width * _map_height

	for y in range(_map_height):
		for x in range(_map_width):
			var idx := y * _map_width + x
			var p := props[idx]
			if p == 0:
				continue

			var sprite := Sprite2D.new()
			var base_pos := Vector2((x + 0.5) * TILE_PX, (y + 0.5) * TILE_PX)
			var jitter := Vector2(rng.randf_range(-8.0, 8.0), rng.randf_range(-6.0, 6.0))
			sprite.position = base_pos + jitter

			match p:
				1:
					sprite.texture = tex_oak
					var s := rng.randf_range(0.56, 0.72)
					sprite.scale = Vector2(s, s)
					sprite.offset = Vector2(0, -34)
				2:
					sprite.texture = tex_pine
					var s := rng.randf_range(0.58, 0.74)
					sprite.scale = Vector2(s, s)
					sprite.offset = Vector2(0, -38)
				3:
					sprite.texture = tex_sapling
					var s := rng.randf_range(0.42, 0.52)
					sprite.scale = Vector2(s, s)
					sprite.offset = Vector2(0, -18)
				4:
					sprite.texture = tex_granite
					var s := rng.randf_range(0.46, 0.58)
					sprite.scale = Vector2(s, s)
					sprite.flip_h = rng.randf() < 0.5
					sprite.offset = Vector2(0, -14)
				5:
					sprite.texture = tex_iron
					var s := rng.randf_range(0.46, 0.58)
					sprite.scale = Vector2(s, s)
					sprite.flip_h = rng.randf() < 0.5
					sprite.offset = Vector2(0, -14)
				6:
					sprite.texture = tex_stones
					var s := rng.randf_range(0.44, 0.56)
					sprite.scale = Vector2(s, s)
					sprite.flip_h = rng.randf() < 0.5

			_entities_layer.add_child(sprite)
			spawned_in_batch += 1
			if spawned_in_batch >= PROPS_PER_FRAME:
				spawned_in_batch = 0
				var pct := 45.0 + (float(idx) / maxf(1.0, float(total_tiles))) * 47.0
				_loading_bar.value = pct
				_loading_status.text = "Colocando bosques y rocas (%d%%)..." % int(round(pct))
				await get_tree().process_frame


func _process(delta: float) -> void:
	if not _world_ready or _colony_sim == null:
		return
	if _sim_speed > 0.0:
		_colony_sim.call("step_simulation", delta * _sim_speed)
	_sync_colonist_visuals()
	_ground_canvas.queue_redraw()

	if _follow_selected and _colonist_nodes.has(_selected_colonist_id):
		var target := _colonist_nodes[_selected_colonist_id] as Node2D
		_camera.position = target.position


func _sync_colonist_visuals() -> void:
	var snapshots_var: Variant = _colony_sim.call("get_colonists_snapshot")
	if not (snapshots_var is Array):
		return
	var snapshots: Array = snapshots_var
	for snap in snapshots:
		if not (snap is Dictionary):
			continue
		var cid := int(snap.get("id", 0))
		var node: Node2D
		if _colonist_nodes.has(cid):
			node = _colonist_nodes[cid]
			node.call("apply_snapshot", snap)
		else:
			node = ColonistVisualScript.new() as Node2D
			node.call("setup_from_snapshot", snap)
			_entities_layer.add_child(node)
			_colonist_nodes[cid] = node

		if cid == _selected_colonist_id:
			_hud.call("show_colonist_info", node)


func _unhandled_input(event: InputEvent) -> void:
	if not _world_ready:
		return
	if event is InputEventScreenTouch:
		var touch := event as InputEventScreenTouch
		if touch.pressed:
			_active_touches[touch.index] = touch.position
			if _active_touches.size() == 1:
				_press_screen_pos = touch.position
				_drag_moved = false
			elif _active_touches.size() == 2:
				var pts := _active_touches.values()
				_last_pinch_dist = pts[0].distance_to(pts[1])
		else:
			if _active_touches.size() == 1 and not _drag_moved:
				_handle_world_tap(touch.position)
			_active_touches.erase(touch.index)
			_last_pinch_dist = 0.0

	elif event is InputEventScreenDrag:
		var drag := event as InputEventScreenDrag
		_active_touches[drag.index] = drag.position
		if _active_touches.size() == 1:
			if drag.position.distance_to(_press_screen_pos) > 12.0:
				_drag_moved = true
				_follow_selected = false
			_pan_camera(-drag.relative)
		elif _active_touches.size() == 2:
			_drag_moved = true
			var pts := _active_touches.values()
			var dist := pts[0].distance_to(pts[1])
			if _last_pinch_dist > 0.0:
				var ratio := (dist - _last_pinch_dist) * 0.004 * SettingsManager.zoom_speed
				_apply_zoom_delta(ratio)
			_last_pinch_dist = dist

	elif event is InputEventMouseButton:
		var mb := event as InputEventMouseButton
		if mb.device == InputEvent.DEVICE_ID_EMULATION:
			return
		if mb.button_index == MOUSE_BUTTON_LEFT:
			if mb.pressed:
				_dragging = true
				_drag_moved = false
				_press_screen_pos = mb.position
			else:
				if _dragging and not _drag_moved:
					_handle_world_tap(mb.position)
				_dragging = false
		elif mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
			_apply_zoom_delta(0.12)
		elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
			_apply_zoom_delta(-0.12)

	elif event is InputEventMouseMotion and _dragging:
		var mm := event as InputEventMouseMotion
		if mm.device == InputEvent.DEVICE_ID_EMULATION:
			return
		if mm.position.distance_to(_press_screen_pos) > 8.0:
			_drag_moved = true
			_follow_selected = false
		_pan_camera(-mm.relative)


func _pan_camera(screen_delta: Vector2) -> void:
	var sens := SettingsManager.pan_sensitivity / maxf(0.25, _camera.zoom.x)
	_camera.position += screen_delta * sens
	_camera.position.x = clampf(_camera.position.x, 128.0, _map_width * TILE_PX - 128.0)
	_camera.position.y = clampf(_camera.position.y, 128.0, _map_height * TILE_PX - 128.0)


func _apply_zoom_delta(delta_zoom: float) -> void:
	var next_z := clampf(_camera.zoom.x + delta_zoom * SettingsManager.zoom_speed, MIN_ZOOM, MAX_ZOOM)
	_camera.zoom = Vector2(next_z, next_z)


func _handle_world_tap(screen_pos: Vector2) -> void:
	var world_pos := get_viewport().get_canvas_transform().affine_inverse() * screen_pos
	var best_id := -1
	var best_dist := 54.0

	for cid in _colonist_nodes.keys():
		var c := _colonist_nodes[cid] as Node2D
		var d := (c.position + Vector2(0, -16)).distance_to(world_pos)
		if d < best_dist:
			best_dist = d
			best_id = cid

	for cid in _colonist_nodes.keys():
		(_colonist_nodes[cid] as Node2D).call("set_selected", cid == best_id)

	_selected_colonist_id = best_id
	if best_id > 0:
		UiTheme.trigger_touch_feedback()
		_hud.call("show_colonist_info", _colonist_nodes[best_id])
	else:
		_follow_selected = false
		_hud.call("show_colonist_info", null)


func _save_current_colony() -> void:
	if _colony_sim == null:
		return
	var snapshots_var: Variant = _colony_sim.call("get_colonists_snapshot")
	var snapshots: Array = snapshots_var if snapshots_var is Array else []
	var payload := {
		"colony_name": GameSession.colony_name,
		"seed": GameSession.seed_value,
		"biome_code": GameSession.biome_code,
		"map_size": GameSession.map_size,
		"day": GameSession.day_count,
		"colonists": snapshots,
	}
	if SaveManager.save_game(GameSession.active_slot, payload):
		_hud.call("show_toast", "Colonia guardada en Ranura %d" % GameSession.active_slot)
	else:
		_hud.call("show_toast", "No se pudo guardar en Ranura %d" % GameSession.active_slot)
