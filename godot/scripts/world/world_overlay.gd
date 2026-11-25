# Capa de superposición del mundo 2D: zonas, cultivos de zanahoria, marcas y selección táctil.
#
# Lógica: Dibuja de forma optimizada (solo dentro de la vista de cámara) las zonas
# de almacenamiento y cultivo, las etapas visuales `.svg` de las zanahorias desde
# tierra arada hasta cosecha lista, los marcadores de tala/minería (diferenciando
# si esperan reunión del líder o ya fueron aprobados) y el rectángulo de arrastre
# táctil al seleccionar áreas en pantalla.

extends Node2D

const TILE_PX := 64.0

var _w: int = 0
var _h: int = 0
var _cam: Camera2D
var _zones: PackedInt32Array
var _crops: Array = []
var _orders: Array = []

var _crop_textures: Dictionary = {}
var _drag_active: bool = false
var _drag_start_tile: Vector2i = Vector2i.ZERO
var _drag_end_tile: Vector2i = Vector2i.ZERO
var _drag_mode: String = "observe"


func _ready() -> void:
	_crop_textures[1] = load("res://assets/crops/soil_tilled.svg") as Texture2D
	_crop_textures[2] = load("res://assets/crops/carrot_stage_1_seeded.svg") as Texture2D
	_crop_textures[3] = load("res://assets/crops/carrot_stage_2_sprout.svg") as Texture2D
	_crop_textures[4] = load("res://assets/crops/carrot_stage_3_growing.svg") as Texture2D
	_crop_textures[5] = load("res://assets/crops/carrot_stage_4_ready.svg") as Texture2D


func configure(w: int, h: int, cam: Camera2D) -> void:
	_w = w
	_h = h
	_cam = cam
	queue_redraw()


func update_world_data(zones: PackedInt32Array, crops: Array, orders: Array) -> void:
	_zones = zones
	_crops = crops
	_orders = orders
	queue_redraw()


func set_drag_preview(active: bool, mode: String, start_tile: Vector2i, end_tile: Vector2i) -> void:
	_drag_active = active
	_drag_mode = mode
	_drag_start_tile = start_tile
	_drag_end_tile = end_tile
	queue_redraw()


func _draw() -> void:
	if _w <= 0 or _h <= 0 or _cam == null:
		return

	var vp_size := get_viewport_rect().size / _cam.zoom
	var top_left := _cam.position - vp_size * 0.55
	var bottom_right := _cam.position + vp_size * 0.55

	var min_x := maxi(0, int(floor(top_left.x / TILE_PX)) - 1)
	var min_y := maxi(0, int(floor(top_left.y / TILE_PX)) - 1)
	var max_x := mini(_w - 1, int(ceil(bottom_right.x / TILE_PX)) + 1)
	var max_y := mini(_h - 1, int(ceil(bottom_right.y / TILE_PX)) + 1)

	# 1. Zonas designadas (Almacén y Cultivo)
	if not _zones.is_empty():
		for y in range(min_y, max_y + 1):
			for x in range(min_x, max_x + 1):
				var z := _zones[y * _w + x]
				if z == 0:
					continue
				var rect := Rect2(x * TILE_PX + 2.0, y * TILE_PX + 2.0, TILE_PX - 4.0, TILE_PX - 4.0)
				if z == 1:
					# Zona de Almacenamiento (ámbar/latón cálido)
					draw_rect(rect, Color(0.84, 0.62, 0.25, 0.20), true)
					draw_rect(rect, Color(0.92, 0.72, 0.34, 0.72), false, 2.0)
				elif z == 2:
					# Zona de Cultivo de zanahorias (verde orgánico tierra)
					draw_rect(rect, Color(0.38, 0.58, 0.26, 0.18), true)
					draw_rect(rect, Color(0.55, 0.78, 0.36, 0.65), false, 2.0)

	# 2. Etapas visuales de cultivo de zanahoria (.svg)
	for crop_var in _crops:
		if not (crop_var is Dictionary):
			continue
		var crop: Dictionary = crop_var
		var cx := int(crop.get("x", 0))
		var cy := int(crop.get("y", 0))
		if cx < min_x or cx > max_x or cy < min_y or cy > max_y:
			continue
		var stage := int(crop.get("stage", 0))
		if stage <= 0:
			continue
		var tex: Texture2D = _crop_textures.get(stage, null)
		if tex != null:
			var rect := Rect2(cx * TILE_PX, cy * TILE_PX, TILE_PX, TILE_PX)
			draw_texture_rect(tex, rect, false)

	# 3. Marcadores de órdenes (Talar, Minar, Construir)
	for order_var in _orders:
		if not (order_var is Dictionary):
			continue
		var order: Dictionary = order_var
		var ox := int(order.get("x", 0))
		var oy := int(order.get("y", 0))
		if ox < min_x or ox > max_x or oy < min_y or oy > max_y:
			continue
		var kind := int(order.get("kind", 0))
		var status := int(order.get("status", 0)) # 0 = esperando líder, 1 = aprobado
		if kind == 1 or kind == 2:
			var rect := Rect2(ox * TILE_PX + 6.0, oy * TILE_PX + 6.0, TILE_PX - 12.0, TILE_PX - 12.0)
			var col := Color(0.96, 0.76, 0.32, 0.88) if status == 0 else Color(0.94, 0.44, 0.28, 0.92)
			draw_rect(rect, Color(col.r, col.g, col.b, 0.16), true)
			draw_rect(rect, col, false, 3.0)
			if status == 0:
				draw_circle(Vector2((ox + 0.8) * TILE_PX, (oy + 0.2) * TILE_PX), 6.0, Color("#f2e8d4"))

	# 4. Previsualización de rectángulo de selección táctil
	if _drag_active:
		var sx := mini(_drag_start_tile.x, _drag_end_tile.x)
		var sy := mini(_drag_start_tile.y, _drag_end_tile.y)
		var ex := maxi(_drag_start_tile.x, _drag_end_tile.x)
		var ey := maxi(_drag_start_tile.y, _drag_end_tile.y)
		var sel_rect := Rect2(
			sx * TILE_PX,
			sy * TILE_PX,
			float(ex - sx + 1) * TILE_PX,
			float(ey - sy + 1) * TILE_PX
		)
		var preview_col := Color(0.88, 0.69, 0.34, 0.85)
		match _drag_mode:
			"chop":
				preview_col = Color(0.90, 0.48, 0.30, 0.90)
			"mine":
				preview_col = Color(0.75, 0.68, 0.56, 0.90)
			"storage":
				preview_col = Color(0.92, 0.72, 0.32, 0.90)
			"farm":
				preview_col = Color(0.54, 0.82, 0.38, 0.90)
			"build_wall":
				preview_col = Color(0.55, 0.78, 0.96, 0.90)
			"cancel":
				preview_col = Color(0.86, 0.34, 0.30, 0.90)
		draw_rect(sel_rect, Color(preview_col.r, preview_col.g, preview_col.b, 0.22), true)
		draw_rect(sel_rect, preview_col, false, 3.5)
