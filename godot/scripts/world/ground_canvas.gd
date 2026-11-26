# Lienzo optimizado del suelo terrestre (pasto, tierra fértil, tierra batida, grava, piedra y río).
#
# Lógica: Dibuja únicamente las casillas visibles dentro del rectángulo de cámara
# usando las texturas vectoriales `.svg` con variación orgánica sutil de tono y
# cuadrícula opcional según la configuración en `SettingsManager`.

extends Node2D

const TILE_PX := 64.0

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
	_textures.clear()
	_textures.append(load("res://assets/ground/ground_grass.svg") as Texture2D)
	_textures.append(load("res://assets/ground/ground_fertile.svg") as Texture2D)
	_textures.append(load("res://assets/ground/ground_dirt.svg") as Texture2D)
	_textures.append(load("res://assets/ground/ground_gravel.svg") as Texture2D)
	_textures.append(load("res://assets/ground/ground_stone.svg") as Texture2D)
	_textures.append(load("res://assets/ground/ground_river.svg") as Texture2D)
	queue_redraw()


func _draw() -> void:
	if _w <= 0 or _h <= 0 or _tiles.is_empty() or _cam == null or _textures.is_empty():
		return

	var vp := get_viewport()
	var top_left: Vector2
	var bottom_right: Vector2
	if vp != null:
		var inv := vp.get_canvas_transform().affine_inverse()
		var vp_rect := vp.get_visible_rect()
		var p0 := inv * vp_rect.position
		var p1 := inv * (vp_rect.position + vp_rect.size)
		top_left = Vector2(minf(p0.x, p1.x), minf(p0.y, p1.y))
		bottom_right = Vector2(maxf(p0.x, p1.x), maxf(p0.y, p1.y))
	else:
		var vp_size := get_viewport_rect().size / _cam.zoom
		var cam_center := _cam.get_screen_center_position()
		top_left = cam_center - vp_size * 0.55
		bottom_right = cam_center + vp_size * 0.55

	var min_x := maxi(0, int(floor(top_left.x / TILE_PX)) - 2)
	var min_y := maxi(0, int(floor(top_left.y / TILE_PX)) - 2)
	var max_x := mini(_w - 1, int(ceil(bottom_right.x / TILE_PX)) + 2)
	var max_y := mini(_h - 1, int(ceil(bottom_right.y / TILE_PX)) + 2)

	var draw_grid := SettingsManager.show_terrain_grid
	var grid_col := Color(0.15, 0.12, 0.08, 0.18)

	for y in range(min_y, max_y + 1):
		for x in range(min_x, max_x + 1):
			var kind := clampi(_tiles[y * _w + x], 0, _textures.size() - 1)
			var tex := _textures[kind]
			if tex == null:
				continue
			var rect := Rect2(x * TILE_PX, y * TILE_PX, TILE_PX + 0.5, TILE_PX + 0.5)
			var shade := 0.96 + float(((x * 37 + y * 17) & 7)) * 0.01
			draw_texture_rect(tex, rect, false, Color(shade, shade, shade, 1.0))
			if draw_grid:
				draw_rect(rect, grid_col, false, 1.0)
