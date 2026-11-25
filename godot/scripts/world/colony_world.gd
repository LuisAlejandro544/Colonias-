# Escena principal jugable: Mundo 2D terrestre en vista centinela (estilo RimWorld).
#
# Lógica: Conecta Godot con los nodos nativos Rust (`WorldGenerator` y
# `ColonySimulation`), renderiza el terreno, zonas, cultivos de zanahoria,
# planos/paredes de madera, recursos sueltos en el piso y colonos con
# herramientas de cobre. Gestiona la cámara táctil, la selección por arrastre
# de áreas (talar, minar, almacén, cultivo, construir) y el guardado completo.

extends Node2D

const TILE_PX := 64.0
const MIN_ZOOM := 0.55
const MAX_ZOOM := 1.85
const PROPS_PER_FRAME := 240

const WorldHudScript := preload("res://scripts/world/world_hud.gd")
const ColonistVisualScript := preload("res://scripts/world/colonist_visual.gd")
const GroundCanvasScript := preload("res://scripts/world/ground_canvas.gd")
const WorldOverlayScript := preload("res://scripts/world/world_overlay.gd")

var _world_gen: Node
var _colony_sim: Node

var _map_width: int = 64
var _map_height: int = 64
var _ground_tiles: PackedInt32Array
var _prop_tiles: PackedInt32Array
var _blocked_tiles: PackedInt32Array

var _ground_canvas: Node2D
var _world_overlay: Node2D
var _entities_layer: Node2D
var _camera: Camera2D
var _hud: CanvasLayer

var _loading_layer: CanvasLayer
var _loading_title: Label
var _loading_status: Label
var _loading_bar: ProgressBar
var _error_back_btn: Button

var _prop_sprites: Dictionary = {}
var _structure_sprites: Dictionary = {}
var _item_nodes: Dictionary = {}
var _colonist_nodes: Dictionary = {}
var _selected_colonist_id: int = -1
var _follow_selected: bool = false
var _sim_speed: float = 1.0
var _world_ready: bool = false

var _structure_textures: Dictionary = {}
var _item_textures: Dictionary = {}

# Control táctil móvil (cámara o arrastre de área según modo activo)
var _tool_mode: String = "observe"
var _active_touches: Dictionary = {}
var _dragging: bool = false
var _drag_moved: bool = false
var _press_screen_pos: Vector2 = Vector2.ZERO
var _last_pinch_dist: float = 0.0

var _area_selecting: bool = false
var _area_start_tile: Vector2i = Vector2i.ZERO
var _area_end_tile: Vector2i = Vector2i.ZERO


func _ready() -> void:
	start_colony_world()


func start_colony_world() -> void:
	if _loading_layer != null:
		return
	_preload_world_textures()
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

	_entities_layer = Node2D.new()
	_entities_layer.y_sort_enabled = true
	add_child(_entities_layer)

	_hud = WorldHudScript.new() as CanvasLayer
	add_child(_hud)
	_hud.connect("speed_changed", func(spd: float): _sim_speed = spd)
	_hud.connect("zoom_step_requested", _apply_zoom_delta)
	_hud.connect("save_requested", _save_current_colony)
	_hud.connect("follow_toggled", func(enabled: bool): _follow_selected = enabled)
	_hud.connect("tool_mode_changed", _on_tool_mode_changed)

	call_deferred("_initialize_world_async")


func _preload_world_textures() -> void:
	_structure_textures = {
		1: load("res://assets/structures/wall_wood_blueprint.svg") as Texture2D,
		2: load("res://assets/structures/wall_wood.svg") as Texture2D,
		201: load("res://assets/structures/wall_wood_crown.svg") as Texture2D,
		202: load("res://assets/structures/wall_wood_front.svg") as Texture2D,
	}
	_item_textures = {
		"wood_logs": load("res://assets/items/item_wood_logs.svg") as Texture2D,
		"stone_chunk": load("res://assets/items/item_stone_chunk.svg") as Texture2D,
		"iron_ore": load("res://assets/items/item_iron_ore.svg") as Texture2D,
		"copper_ore": load("res://assets/items/item_copper_ore.svg") as Texture2D,
		"carrot_seed": load("res://assets/items/item_carrot_seeds.svg") as Texture2D,
		"carrot": load("res://assets/items/item_carrot.svg") as Texture2D,
	}


func _on_tool_mode_changed(mode_code: String) -> void:
	_tool_mode = mode_code
	_area_selecting = false
	if _world_overlay:
		_world_overlay.call("set_drag_preview", false, _tool_mode, Vector2i.ZERO, Vector2i.ZERO)


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
	_prop_tiles = data.get("props", PackedInt32Array())
	_blocked_tiles = data.get("blocked", PackedInt32Array())

	if _ground_tiles.is_empty():
		_show_core_error("El generador procedural devolvió un terreno vacío.")
		return

	# Si cargamos una partida guardada con árboles talados/rocas minadas, respetar su estado
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
	_loading_status.text = "Preparando suelo terrestre (%dx%d)..." % [_map_width, _map_height]
	await get_tree().process_frame

	_ground_canvas.call("configure", _map_width, _map_height, _ground_tiles, _camera)
	_world_overlay.call("configure", _map_width, _map_height, _camera)

	_camera.limit_left = 0
	_camera.limit_top = 0
	_camera.limit_right = int(_map_width * TILE_PX)
	_camera.limit_bottom = int(_map_height * TILE_PX)
	_camera.position = Vector2(_map_width * TILE_PX * 0.5, _map_height * TILE_PX * 0.5)

	await _spawn_world_props_async(_prop_tiles)

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
		var state_copy := _convert_save_dict_for_rust(GameSession.saved_world_state)
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
	_loading_layer.visible = false


func _convert_save_dict_for_rust(raw: Dictionary) -> Dictionary:
	var out := raw.duplicate(true)
	out["day"] = int(out.get("day", 1))
	for key in ["props", "blocked", "zones", "structures"]:
		var val = out.get(key, [])
		if val is Array:
			var packed := PackedInt32Array()
			packed.resize(val.size())
			for i in range(val.size()):
				packed[i] = int(val[i])
			out[key] = packed

	var crops_raw = out.get("crops", [])
	if crops_raw is Array:
		var crops_clean: Array = []
		for entry in crops_raw:
			if entry is Dictionary:
				crops_clean.append({
					"x": int(entry.get("x", 0)),
					"y": int(entry.get("y", 0)),
					"stage": int(entry.get("stage", 0)),
					"progress": float(entry.get("progress", 0.0)),
				})
		out["crops"] = crops_clean

	var items_raw = out.get("items", [])
	if items_raw is Array:
		var items_clean: Array = []
		for entry in items_raw:
			if entry is Dictionary:
				items_clean.append({
					"kind": str(entry.get("kind", "")),
					"x": int(entry.get("x", 0)),
					"y": int(entry.get("y", 0)),
					"amount": int(entry.get("amount", 1)),
					"stored": bool(entry.get("stored", false)),
				})
		out["items"] = items_clean

	var orders_raw = out.get("orders", [])
	if orders_raw is Array:
		var orders_clean: Array = []
		for entry in orders_raw:
			if entry is Dictionary:
				orders_clean.append({
					"kind": int(entry.get("kind", 1)),
					"x": int(entry.get("x", 0)),
					"y": int(entry.get("y", 0)),
					"status": int(entry.get("status", 0)),
				})
		out["orders"] = orders_clean

	var cols_raw = out.get("colonists", [])
	if cols_raw is Array:
		var cols_clean: Array = []
		for entry in cols_raw:
			if entry is Dictionary:
				cols_clean.append({
					"id": int(entry.get("id", 1)),
					"name": str(entry.get("name", "Colono")),
					"role": str(entry.get("role", "Explorador")),
					"is_leader": bool(entry.get("is_leader", false)),
					"x": float(entry.get("x", 32.0)),
					"y": float(entry.get("y", 32.0)),
					"health": int(entry.get("health", 100)),
					"mood": int(entry.get("mood", 80)),
					"body_style": int(entry.get("body_style", 0)),
					"hair_style": int(entry.get("hair_style", 0)),
					"skin_color": str(entry.get("skin_color", "#e4b892")),
					"hair_color": str(entry.get("hair_color", "#2c221e")),
					"outfit_color": str(entry.get("outfit_color", "#4f6b47")),
					"equipped_tool": str(entry.get("equipped_tool", "")),
				})
		out["colonists"] = cols_clean

	return out


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
			_prop_sprites[idx] = sprite
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
	_sync_world_snapshot(false)
	_sync_colonist_visuals()
	_ground_canvas.queue_redraw()
	_world_overlay.queue_redraw()

	if _follow_selected and _colonist_nodes.has(_selected_colonist_id):
		var target := _colonist_nodes[_selected_colonist_id] as Node2D
		_camera.position = target.position


func _sync_world_snapshot(force_full: bool) -> void:
	var snap_var: Variant = _colony_sim.call("get_world_snapshot", force_full)
	if not (snap_var is Dictionary):
		return
	var snap: Dictionary = snap_var
	_hud.call("update_world_summary", snap, _colonist_nodes.size() if not _colonist_nodes.is_empty() else GameSession.initial_colonists)

	if not bool(snap.get("dirty", false)):
		return

	# 1. Props (eliminar árboles talados o rocas picadas)
	var props: PackedInt32Array = snap.get("props", PackedInt32Array())
	if not props.is_empty():
		var to_remove: Array[int] = []
		for idx in _prop_sprites.keys():
			if idx < props.size() and props[idx] == 0:
				var sp := _prop_sprites[idx] as Node
				if is_instance_valid(sp):
					sp.queue_free()
				to_remove.append(idx)
		for idx in to_remove:
			_prop_sprites.erase(idx)

	# 2. Estructuras (planos y paredes de madera conectables por vecinos)
	var structures: PackedInt32Array = snap.get("structures", PackedInt32Array())
	if not structures.is_empty():
		for idx in range(structures.size()):
			var st := structures[idx]
			if st == 0:
				if _structure_sprites.has(idx):
					var old_node := _structure_sprites[idx] as Node
					if is_instance_valid(old_node):
						old_node.queue_free()
					_structure_sprites.erase(idx)
			else:
				var node: _WallTileVisual
				if _structure_sprites.has(idx):
					node = _structure_sprites[idx] as _WallTileVisual
				else:
					node = _WallTileVisual.new()
					var tx := idx % _map_width
					var ty := int(idx / _map_width)
					node.position = Vector2((tx + 0.5) * TILE_PX, (ty + 0.5) * TILE_PX)
					_entities_layer.add_child(node)
					_structure_sprites[idx] = node

				if st == 1:
					node.configure(_structure_textures.get(1, null), true, true, true, true, false)
				else:
					var tx := idx % _map_width
					var ty := int(idx / _map_width)
					var has_n := ty > 0 and structures[(ty - 1) * _map_width + tx] == 2
					var has_s := ty + 1 < _map_height and structures[(ty + 1) * _map_width + tx] == 2
					var has_w := tx > 0 and structures[ty * _map_width + (tx - 1)] == 2
					var has_e := tx + 1 < _map_width and structures[ty * _map_width + (tx + 1)] == 2
					var isolated := not has_n and not has_s and not has_w and not has_e
					var tex: Texture2D
					if isolated:
						tex = _structure_textures.get(2, null)
					elif has_s:
						tex = _structure_textures.get(201, null)
					else:
						tex = _structure_textures.get(202, null)
					node.configure(tex, has_n, has_s, has_w, has_e, not isolated)

	# 3. Recursos sueltos en el piso (troncos, piedra, hierro, cobre, semillas, zanahorias)
	var items_arr: Array = snap.get("items", [])
	var seen_ids: Dictionary = {}
	for it_var in items_arr:
		if not (it_var is Dictionary):
			continue
		var it: Dictionary = it_var
		var iid := int(it.get("id", 0))
		seen_ids[iid] = true
		var node: Node2D
		if _item_nodes.has(iid):
			node = _item_nodes[iid] as Node2D
		else:
			node = _create_ground_item_node()
			_entities_layer.add_child(node)
			_item_nodes[iid] = node
		_update_ground_item_node(node, it)

	var dead_items: Array[int] = []
	for iid in _item_nodes.keys():
		if not seen_ids.has(iid):
			var n := _item_nodes[iid] as Node
			if is_instance_valid(n):
				n.queue_free()
			dead_items.append(iid)
	for iid in dead_items:
		_item_nodes.erase(iid)

	# 4. Zonas, cultivos y órdenes para WorldOverlay
	var zones: PackedInt32Array = snap.get("zones", PackedInt32Array())
	var crops: Array = snap.get("crops", [])
	var orders: Array = snap.get("orders", [])
	_world_overlay.call("update_world_data", zones, crops, orders)


func _create_ground_item_node() -> Node2D:
	var root := Node2D.new()
	var sp := Sprite2D.new()
	sp.name = "Icon"
	sp.scale = Vector2(0.34, 0.34)
	root.add_child(sp)

	var lbl := Label.new()
	lbl.name = "Qty"
	lbl.position = Vector2(4, 4)
	lbl.add_theme_color_override("font_color", Color("#fff5dc"))
	lbl.add_theme_color_override("font_outline_color", Color("#1b1510"))
	lbl.add_theme_constant_override("outline_size", 4)
	lbl.add_theme_font_size_override("font_size", 14)
	root.add_child(lbl)
	return root


func _update_ground_item_node(node: Node2D, it: Dictionary) -> void:
	var ix := int(it.get("x", 0))
	var iy := int(it.get("y", 0))
	node.position = Vector2((ix + 0.5) * TILE_PX, (iy + 0.54) * TILE_PX)
	var kind := str(it.get("kind", "wood_logs"))
	var amount := int(it.get("amount", 1))
	var sp := node.get_node("Icon") as Sprite2D
	if sp:
		sp.texture = _item_textures.get(kind, null)
	var lbl := node.get_node("Qty") as Label
	if lbl:
		lbl.text = "x%d" % amount


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


func _screen_to_tile(screen_pos: Vector2) -> Vector2i:
	var world_pos := get_viewport().get_canvas_transform().affine_inverse() * screen_pos
	var tx := clampi(int(floor(world_pos.x / TILE_PX)), 1, _map_width - 2)
	var ty := clampi(int(floor(world_pos.y / TILE_PX)), 1, _map_height - 2)
	return Vector2i(tx, ty)


func _commit_area_selection() -> void:
	if not _area_selecting:
		return
	_area_selecting = false
	_world_overlay.call("set_drag_preview", false, _tool_mode, Vector2i.ZERO, Vector2i.ZERO)
	var affected := int(_colony_sim.call(
		"designate_area",
		_tool_mode,
		_area_start_tile.x,
		_area_start_tile.y,
		_area_end_tile.x,
		_area_end_tile.y
	))
	if affected > 0:
		UiTheme.trigger_touch_feedback()
		if _tool_mode == "cancel":
			_hud.call("show_toast", "Canceladas %d casillas" % affected)
		else:
			_hud.call("show_toast", "%d casillas marcadas • El líder reunirá al equipo" % affected)
	else:
		_hud.call("show_toast", "No hay objetivos válidos en esa selección")


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
				if _tool_mode != "observe":
					_area_selecting = true
					_area_start_tile = _screen_to_tile(touch.position)
					_area_end_tile = _area_start_tile
					_world_overlay.call("set_drag_preview", true, _tool_mode, _area_start_tile, _area_end_tile)
			elif _active_touches.size() == 2:
				_area_selecting = false
				_world_overlay.call("set_drag_preview", false, _tool_mode, Vector2i.ZERO, Vector2i.ZERO)
				var pts: Array = _active_touches.values()
				var p0 := pts[0] as Vector2
				var p1 := pts[1] as Vector2
				_last_pinch_dist = p0.distance_to(p1)
		else:
			if _area_selecting and _active_touches.size() == 1:
				_commit_area_selection()
			elif _active_touches.size() == 1 and not _drag_moved and _tool_mode == "observe":
				_handle_world_tap(touch.position)
			_active_touches.erase(touch.index)
			_last_pinch_dist = 0.0

	elif event is InputEventScreenDrag:
		var drag := event as InputEventScreenDrag
		_active_touches[drag.index] = drag.position
		if _active_touches.size() == 1:
			if _tool_mode != "observe" and _area_selecting:
				_area_end_tile = _screen_to_tile(drag.position)
				_world_overlay.call("set_drag_preview", true, _tool_mode, _area_start_tile, _area_end_tile)
			else:
				if drag.position.distance_to(_press_screen_pos) > 12.0:
					_drag_moved = true
					_follow_selected = false
				_pan_camera(-drag.relative)
		elif _active_touches.size() == 2:
			_drag_moved = true
			var pts: Array = _active_touches.values()
			var p0 := pts[0] as Vector2
			var p1 := pts[1] as Vector2
			var dist: float = p0.distance_to(p1)
			if _last_pinch_dist > 0.0:
				var ratio: float = (dist - _last_pinch_dist) * 0.004 * SettingsManager.zoom_speed
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
				if _tool_mode != "observe":
					_area_selecting = true
					_area_start_tile = _screen_to_tile(mb.position)
					_area_end_tile = _area_start_tile
					_world_overlay.call("set_drag_preview", true, _tool_mode, _area_start_tile, _area_end_tile)
			else:
				if _area_selecting:
					_commit_area_selection()
				elif _dragging and not _drag_moved and _tool_mode == "observe":
					_handle_world_tap(mb.position)
				_dragging = false
		elif mb.button_index == MOUSE_BUTTON_RIGHT:
			_dragging = mb.pressed
		elif mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
			_apply_zoom_delta(0.12)
		elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
			_apply_zoom_delta(-0.12)

	elif event is InputEventMouseMotion and _dragging:
		var mm := event as InputEventMouseMotion
		if mm.device == InputEvent.DEVICE_ID_EMULATION:
			return
		if _tool_mode != "observe" and _area_selecting and (mm.button_mask & MOUSE_BUTTON_MASK_LEFT) != 0:
			_area_end_tile = _screen_to_tile(mm.position)
			_world_overlay.call("set_drag_preview", true, _tool_mode, _area_start_tile, _area_end_tile)
		else:
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
	var state_var: Variant = _colony_sim.call("export_save_state")
	var state: Dictionary = state_var if state_var is Dictionary else {}
	var payload := {
		"colony_name": GameSession.colony_name,
		"seed": GameSession.seed_value,
		"biome_code": GameSession.biome_code,
		"map_size": GameSession.map_size,
		"day": int(state.get("day", GameSession.day_count)),
		"colonists": state.get("colonists", []),
		"props": Array(state.get("props", PackedInt32Array())),
		"blocked": Array(state.get("blocked", PackedInt32Array())),
		"zones": Array(state.get("zones", PackedInt32Array())),
		"structures": Array(state.get("structures", PackedInt32Array())),
		"crops": state.get("crops", []),
		"items": state.get("items", []),
		"orders": state.get("orders", []),
	}
	if SaveManager.save_game(GameSession.active_slot, payload):
		_hud.call("show_toast", "Colonia guardada en Ranura %d" % GameSession.active_slot)
	else:
		_hud.call("show_toast", "No se pudo guardar en Ranura %d" % GameSession.active_slot)


class _WallTileVisual extends Node2D:
	var _sprite: Sprite2D
	var _border: _WallBorderDrawer

	func _init() -> void:
		_sprite = Sprite2D.new()
		_sprite.scale = Vector2(0.50, 0.50)
		add_child(_sprite)
		_border = _WallBorderDrawer.new()
		add_child(_border)

	func configure(
		tex: Texture2D,
		has_n: bool,
		has_s: bool,
		has_w: bool,
		has_e: bool,
		draw_borders: bool
	) -> void:
		_sprite.texture = tex
		_border.visible = draw_borders
		_border.set_edges(not has_n, not has_s, not has_w, not has_e)


class _WallBorderDrawer extends Node2D:
	var _n: bool = false
	var _s: bool = false
	var _w: bool = false
	var _e: bool = false

	func set_edges(n: bool, s: bool, w: bool, e: bool) -> void:
		if _n == n and _s == s and _w == w and _e == e:
			return
		_n = n
		_s = s
		_w = w
		_e = e
		queue_redraw()

	func _draw() -> void:
		var outline := Color("#1e1813")
		var w := 2.4
		if _n:
			draw_line(Vector2(-32.0, -31.0), Vector2(32.0, -31.0), outline, w)
		if _s:
			draw_line(Vector2(-32.0, 31.0), Vector2(32.0, 31.0), outline, w)
		if _w:
			draw_line(Vector2(-31.0, -32.0), Vector2(-31.0, 32.0), outline, w)
		if _e:
			draw_line(Vector2(31.0, -32.0), Vector2(31.0, 32.0), outline, w)

