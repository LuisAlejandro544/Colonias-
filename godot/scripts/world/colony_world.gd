# Escena principal jugable: Mundo 2D terrestre en vista centinela (estilo RimWorld).
#
# Lógica: Conecta directamente con los nodos nativos en Rust (`WorldGenerator` y
# `ColonySimulation`), dibuja el piso optimizado dentro del área visible de la
# cámara, coloca árboles, arbolitos, rocas y piedras ordenados por eje Y junto a
# los colonos autónomos, y gestiona la cámara táctil (arrastre, pellizco y selección).

extends Node2D

const TILE_PX := 64.0
const MIN_ZOOM := 0.55
const MAX_ZOOM := 1.85

var _world_gen: WorldGenerator
var _colony_sim: ColonySimulation

var _map_width: int = 64
var _map_height: int = 64
var _ground_tiles: PackedInt32Array
var _blocked_tiles: PackedInt32Array

var _ground_canvas: _GroundCanvas
var _entities_layer: Node2D
var _camera: Camera2D
var _hud: WorldHud

var _colonist_nodes: Dictionary = {}
var _selected_colonist_id: int = -1
var _follow_selected: bool = false
var _sim_speed: float = 1.0

# Control táctil móvil (arrastre, detección de toque y pellizco)
var _active_touches: Dictionary = {}
var _dragging: bool = false
var _drag_moved: bool = false
var _press_screen_pos: Vector2 = Vector2.ZERO
var _last_pinch_dist: float = 0.0


func _ready() -> void:
	_world_gen = WorldGenerator.new()
	add_child(_world_gen)

	_colony_sim = ColonySimulation.new()
	add_child(_colony_sim)

	_camera = Camera2D.new()
	_camera.position_smoothing_enabled = true
	_camera.position_smoothing_speed = 9.0
	_camera.zoom = Vector2(1.0, 1.0)
	add_child(_camera)
	_camera.make_current()

	_ground_canvas = _GroundCanvas.new()
	add_child(_ground_canvas)

	_entities_layer = Node2D.new()
	_entities_layer.y_sort_enabled = true
	add_child(_entities_layer)

	_hud = WorldHud.new()
	add_child(_hud)
	_hud.speed_changed.connect(func(spd: float): _sim_speed = spd)
	_hud.zoom_step_requested.connect(_apply_zoom_delta)
	_hud.save_requested.connect(_save_current_colony)
	_hud.follow_toggled.connect(func(enabled: bool): _follow_selected = enabled)

	_initialize_world()


func _initialize_world() -> void:
	var data: Dictionary = _world_gen.generate_world(
		GameSession.seed_value,
		GameSession.map_size,
		GameSession.map_size,
		GameSession.biome_code
	)
	_map_width = int(data.get("width", GameSession.map_size))
	_map_height = int(data.get("height", GameSession.map_size))
	_ground_tiles = data.get("ground", PackedInt32Array())
	var props: PackedInt32Array = data.get("props", PackedInt32Array())
	_blocked_tiles = data.get("blocked", PackedInt32Array())

	_ground_canvas.configure(_map_width, _map_height, _ground_tiles, _camera)
	_camera.limit_left = 0
	_camera.limit_top = 0
	_camera.limit_right = int(_map_width * TILE_PX)
	_camera.limit_bottom = int(_map_height * TILE_PX)
	_camera.position = Vector2(_map_width * TILE_PX * 0.5, _map_height * TILE_PX * 0.5)

	_spawn_world_props(props)

	if GameSession.saved_colonists.is_empty():
		_colony_sim.setup_colony(
			GameSession.seed_value,
			_map_width,
			_map_height,
			_blocked_tiles,
			GameSession.initial_colonists
		)
	else:
		var arr := Array(GameSession.saved_colonists)
		_colony_sim.restore_colonists(
			GameSession.seed_value,
			_map_width,
			_map_height,
			_blocked_tiles,
			arr
		)

	_sync_colonist_visuals()


func _spawn_world_props(props: PackedInt32Array) -> void:
	var tex_oak := load("res://assets/props/tree_oak.svg") as Texture2D
	var tex_pine := load("res://assets/props/tree_pine.svg") as Texture2D
	var tex_sapling := load("res://assets/props/sapling.svg") as Texture2D
	var tex_granite := load("res://assets/props/boulder_granite.svg") as Texture2D
	var tex_iron := load("res://assets/props/boulder_iron.svg") as Texture2D
	var tex_stones := load("res://assets/props/stone_pebbles.svg") as Texture2D

	var rng := RandomNumberGenerator.new()
	rng.seed = GameSession.seed_value ^ 0x4455

	for y in range(_map_height):
		for x in range(_map_width):
			var p := props[y * _map_width + x]
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


func _process(delta: float) -> void:
	if _sim_speed > 0.0:
		_colony_sim.step_simulation(delta * _sim_speed)
	_sync_colonist_visuals()
	_ground_canvas.queue_redraw()

	if _follow_selected and _colonist_nodes.has(_selected_colonist_id):
		var target := _colonist_nodes[_selected_colonist_id] as ColonistVisual
		_camera.position = target.position


func _sync_colonist_visuals() -> void:
	var snapshots: Array = _colony_sim.get_colonists_snapshot()
	for snap in snapshots:
		if not (snap is Dictionary):
			continue
		var cid := int(snap.get("id", 0))
		var node: ColonistVisual
		if _colonist_nodes.has(cid):
			node = _colonist_nodes[cid]
			node.apply_snapshot(snap)
		else:
			node = ColonistVisual.new()
			node.setup_from_snapshot(snap)
			_entities_layer.add_child(node)
			_colonist_nodes[cid] = node

		if cid == _selected_colonist_id:
			_hud.show_colonist_info(node)


func _unhandled_input(event: InputEvent) -> void:
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
	var best_dist := 48.0

	for cid in _colonist_nodes.keys():
		var c := _colonist_nodes[cid] as ColonistVisual
		var d := (c.position + Vector2(0, -16)).distance_to(world_pos)
		if d < best_dist:
			best_dist = d
			best_id = cid

	for cid in _colonist_nodes.keys():
		(_colonist_nodes[cid] as ColonistVisual).set_selected(cid == best_id)

	_selected_colonist_id = best_id
	if best_id > 0:
		UiTheme.trigger_touch_feedback()
		_hud.show_colonist_info(_colonist_nodes[best_id])
	else:
		_follow_selected = false
		_hud.show_colonist_info(null)


func _save_current_colony() -> void:
	var snapshots := _colony_sim.get_colonists_snapshot()
	var payload := {
		"colony_name": GameSession.colony_name,
		"seed": GameSession.seed_value,
		"biome_code": GameSession.biome_code,
		"map_size": GameSession.map_size,
		"day": GameSession.day_count,
		"colonists": snapshots,
	}
	if SaveManager.save_game(GameSession.active_slot, payload):
		_hud.show_toast("Colonia guardada en Ranura %d" % GameSession.active_slot)
	else:
		_hud.show_toast("No se pudo guardar en Ranura %d" % GameSession.active_slot)


class _GroundCanvas extends Node2D:
	var _w: int = 0
	var _h: int = 0
	var _tiles: PackedInt32Array
	var _cam: Camera2D
	var _textures: Array[Texture2D] = []

	func configure(w: int, h: int, tiles: PackedInt32Array, cam: Camera2D) -> void:
		_w = w
		_h = h
		_tiles = tiles
		_cam = cam
		_textures = [
			load("res://assets/ground/ground_grass.svg") as Texture2D,
			load("res://assets/ground/ground_fertile.svg") as Texture2D,
			load("res://assets/ground/ground_dirt.svg") as Texture2D,
			load("res://assets/ground/ground_gravel.svg") as Texture2D,
			load("res://assets/ground/ground_stone.svg") as Texture2D,
		]
		queue_redraw()

	func _draw() -> void:
		if _w <= 0 or _h <= 0 or _tiles.is_empty() or _cam == null:
			return

		var vp_size := get_viewport_rect().size / _cam.zoom
		var top_left := _cam.position - vp_size * 0.55
		var bottom_right := _cam.position + vp_size * 0.55

		var min_x := maxi(0, int(floor(top_left.x / TILE_PX)) - 1)
		var min_y := maxi(0, int(floor(top_left.y / TILE_PX)) - 1)
		var max_x := mini(_w - 1, int(ceil(bottom_right.x / TILE_PX)) + 1)
		var max_y := mini(_h - 1, int(ceil(bottom_right.y / TILE_PX)) + 1)

		var draw_grid := SettingsManager.show_terrain_grid
		var grid_col := Color(0.15, 0.12, 0.08, 0.18)

		for y in range(min_y, max_y + 1):
			for x in range(min_x, max_x + 1):
				var kind := clampi(_tiles[y * _w + x], 0, _textures.size() - 1)
				var tex := _textures[kind]
				var rect := Rect2(x * TILE_PX, y * TILE_PX, TILE_PX + 0.5, TILE_PX + 0.5)
				# Ligera variación orgánica de tono para evitar repetición uniforme
				var shade := 0.96 + float(((x * 37 + y * 17) & 7)) * 0.01
				draw_texture_rect(tex, rect, false, Color(shade, shade, shade, 1.0))
				if draw_grid:
					draw_rect(rect, grid_col, false, 1.0)
