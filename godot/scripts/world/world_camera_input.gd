# Controlador modular de entrada táctil/ratón, cámara y selección de áreas.
#
# Lógica: Gestiona el paneo táctil con un dedo, zoom por pellizco con dos dedos
# o rueda del ratón, toques simples para seleccionar colonos y arrastre por
# rectángulo de casillas para designar órdenes y zonas en `WorldOverlay`.

class_name WorldCameraInput
extends Node

signal area_committed(mode_code: String, start_tile: Vector2i, end_tile: Vector2i)
signal world_tapped(screen_pos: Vector2)
signal follow_cancelled

const TILE_PX := 64.0
const MIN_ZOOM := 0.55
const MAX_ZOOM := 1.85

var _enabled: bool = false
var _map_width: int = 64
var _map_height: int = 64
var _camera: Camera2D
var _world_overlay: Node2D

var _tool_mode: String = "observe"
var _active_touches: Dictionary = {}
var _dragging: bool = false
var _drag_moved: bool = false
var _press_screen_pos: Vector2 = Vector2.ZERO
var _last_pinch_dist: float = 0.0

var _area_selecting: bool = false
var _area_start_tile: Vector2i = Vector2i.ZERO
var _area_end_tile: Vector2i = Vector2i.ZERO


func configure(map_width: int, map_height: int, camera: Camera2D, world_overlay: Node2D) -> void:
	_map_width = map_width
	_map_height = map_height
	_camera = camera
	_world_overlay = world_overlay
	var vp := get_viewport()
	if vp != null and not vp.size_changed.is_connected(clamp_camera_to_bounds):
		vp.size_changed.connect(clamp_camera_to_bounds)


func set_enabled(enabled: bool) -> void:
	_enabled = enabled


func set_tool_mode(mode_code: String) -> void:
	_tool_mode = mode_code
	_area_selecting = false
	if _world_overlay:
		_world_overlay.call("set_drag_preview", false, _tool_mode, Vector2i.ZERO, Vector2i.ZERO)


func screen_to_tile(screen_pos: Vector2) -> Vector2i:
	var world_pos := get_viewport().get_canvas_transform().affine_inverse() * screen_pos
	var tx := clampi(int(floor(world_pos.x / TILE_PX)), 1, _map_width - 2)
	var ty := clampi(int(floor(world_pos.y / TILE_PX)), 1, _map_height - 2)
	return Vector2i(tx, ty)


func screen_to_world(screen_pos: Vector2) -> Vector2:
	return get_viewport().get_canvas_transform().affine_inverse() * screen_pos


func clamp_camera_to_bounds() -> void:
	if _camera == null:
		return
	var vp := get_viewport()
	var vp_size := vp.get_visible_rect().size if vp != null else Vector2(1280.0, 720.0)
	var map_w_px := _map_width * TILE_PX
	var map_h_px := _map_height * TILE_PX

	var min_fit_zoom := maxf(MIN_ZOOM, maxf(vp_size.x / maxf(1.0, map_w_px), vp_size.y / maxf(1.0, map_h_px)))
	var clamped_zoom := clampf(_camera.zoom.x, min_fit_zoom, MAX_ZOOM)
	if not is_equal_approx(clamped_zoom, _camera.zoom.x):
		_camera.zoom = Vector2(clamped_zoom, clamped_zoom)

	var half_vp := (vp_size / maxf(0.1, _camera.zoom.x)) * 0.5
	if half_vp.x * 2.0 >= map_w_px:
		_camera.position.x = map_w_px * 0.5
	else:
		_camera.position.x = clampf(_camera.position.x, half_vp.x, map_w_px - half_vp.x)

	if half_vp.y * 2.0 >= map_h_px:
		_camera.position.y = map_h_px * 0.5
	else:
		_camera.position.y = clampf(_camera.position.y, half_vp.y, map_h_px - half_vp.y)


func pan_camera(screen_delta: Vector2) -> void:
	if _camera == null:
		return
	var sens := SettingsManager.pan_sensitivity / maxf(0.25, _camera.zoom.x)
	_camera.position += screen_delta * sens
	clamp_camera_to_bounds()


func apply_zoom_delta(delta_zoom: float) -> void:
	if _camera == null:
		return
	var vp := get_viewport()
	var vp_size := vp.get_visible_rect().size if vp != null else Vector2(1280.0, 720.0)
	var min_fit_zoom := maxf(MIN_ZOOM, maxf(vp_size.x / maxf(1.0, _map_width * TILE_PX), vp_size.y / maxf(1.0, _map_height * TILE_PX)))
	var next_z := clampf(_camera.zoom.x + delta_zoom * SettingsManager.zoom_speed, min_fit_zoom, MAX_ZOOM)
	_camera.zoom = Vector2(next_z, next_z)
	clamp_camera_to_bounds()


func _commit_area_selection() -> void:
	if not _area_selecting:
		return
	_area_selecting = false
	if _world_overlay:
		_world_overlay.call("set_drag_preview", false, _tool_mode, Vector2i.ZERO, Vector2i.ZERO)
	area_committed.emit(_tool_mode, _area_start_tile, _area_end_tile)


func _unhandled_input(event: InputEvent) -> void:
	if not _enabled:
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
					_area_start_tile = screen_to_tile(touch.position)
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
				world_tapped.emit(touch.position)
			_active_touches.erase(touch.index)
			_last_pinch_dist = 0.0

	elif event is InputEventScreenDrag:
		var drag := event as InputEventScreenDrag
		_active_touches[drag.index] = drag.position
		if _active_touches.size() == 1:
			if _tool_mode != "observe" and _area_selecting:
				_area_end_tile = screen_to_tile(drag.position)
				_world_overlay.call("set_drag_preview", true, _tool_mode, _area_start_tile, _area_end_tile)
			else:
				if drag.position.distance_to(_press_screen_pos) > 12.0:
					_drag_moved = true
					follow_cancelled.emit()
				pan_camera(-drag.relative)
		elif _active_touches.size() == 2:
			_drag_moved = true
			var pts: Array = _active_touches.values()
			var p0 := pts[0] as Vector2
			var p1 := pts[1] as Vector2
			var dist: float = p0.distance_to(p1)
			if _last_pinch_dist > 0.0:
				var ratio: float = (dist - _last_pinch_dist) * 0.004 * SettingsManager.zoom_speed
				apply_zoom_delta(ratio)
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
					_area_start_tile = screen_to_tile(mb.position)
					_area_end_tile = _area_start_tile
					_world_overlay.call("set_drag_preview", true, _tool_mode, _area_start_tile, _area_end_tile)
			else:
				if _area_selecting:
					_commit_area_selection()
				elif _dragging and not _drag_moved and _tool_mode == "observe":
					world_tapped.emit(mb.position)
				_dragging = false
		elif mb.button_index == MOUSE_BUTTON_RIGHT:
			_dragging = mb.pressed
		elif mb.button_index == MOUSE_BUTTON_WHEEL_UP and mb.pressed:
			apply_zoom_delta(0.12)
		elif mb.button_index == MOUSE_BUTTON_WHEEL_DOWN and mb.pressed:
			apply_zoom_delta(-0.12)

	elif event is InputEventMouseMotion and _dragging:
		var mm := event as InputEventMouseMotion
		if mm.device == InputEvent.DEVICE_ID_EMULATION:
			return
		if _tool_mode != "observe" and _area_selecting and (mm.button_mask & MOUSE_BUTTON_MASK_LEFT) != 0:
			_area_end_tile = screen_to_tile(mm.position)
			_world_overlay.call("set_drag_preview", true, _tool_mode, _area_start_tile, _area_end_tile)
		else:
			if mm.position.distance_to(_press_screen_pos) > 8.0:
				_drag_moved = true
				follow_cancelled.emit()
			pan_camera(-mm.relative)
