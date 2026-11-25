# Representación visual modular por capas de un colono estilo RimWorld.
#
# Lógica: Construye un peón 2D apilando sombra, torso (túnica o abrigo),
# cabeza, rostro expresivo y cabello con tintes independientes (`modulate`).
# Sincroniza posición, orientación horizontal/trasera y balanceo procedural de
# caminata a partir de los snapshots generados en Rust (`ColonySimulation`).

class_name ColonistVisual
extends Node2D

const TILE_PX := 64.0

var colonist_id: int = 0
var colonist_name: String = "Colono"
var colonist_role: String = ""
var current_activity: String = ""
var health_value: int = 100
var mood_value: int = 80
var is_selected: bool = false

var _rig: Node2D
var _body_sprite: Sprite2D
var _head_sprite: Sprite2D
var _face_sprite: Sprite2D
var _hair_sprite: Sprite2D
var _name_label: Label
var _selection_ring: Node2D


func setup_from_snapshot(data: Dictionary) -> void:
	colonist_id = int(data.get("id", 1))
	colonist_name = str(data.get("name", "Colono"))
	colonist_role = str(data.get("role", "Explorador"))
	_build_layers(data)
	apply_snapshot(data)


func _build_layers(data: Dictionary) -> void:
	_selection_ring = _SelectionRing.new()
	_selection_ring.visible = false
	add_child(_selection_ring)

	var shadow := Sprite2D.new()
	shadow.texture = load("res://assets/colonists/pawn_shadow.svg") as Texture2D
	shadow.scale = Vector2(0.46, 0.46)
	shadow.position = Vector2(0, -14)
	add_child(shadow)

	_rig = Node2D.new()
	_rig.position = Vector2(0, -18)
	add_child(_rig)

	var body_style := int(data.get("body_style", 0))
	var body_path := "res://assets/colonists/pawn_body_parka.svg" if body_style == 1 else "res://assets/colonists/pawn_body_tunic.svg"
	_body_sprite = Sprite2D.new()
	_body_sprite.texture = load(body_path) as Texture2D
	_body_sprite.scale = Vector2(0.48, 0.48)
	_body_sprite.modulate = Color(str(data.get("outfit_color", "#4f6b47")))
	_rig.add_child(_body_sprite)

	_head_sprite = Sprite2D.new()
	_head_sprite.texture = load("res://assets/colonists/pawn_head.svg") as Texture2D
	_head_sprite.scale = Vector2(0.48, 0.48)
	_head_sprite.modulate = Color(str(data.get("skin_color", "#e4b892")))
	_rig.add_child(_head_sprite)

	_face_sprite = Sprite2D.new()
	_face_sprite.texture = load("res://assets/colonists/pawn_face.svg") as Texture2D
	_face_sprite.scale = Vector2(0.48, 0.48)
	_rig.add_child(_face_sprite)

	var hair_style := int(data.get("hair_style", 0))
	var hair_path := "res://assets/colonists/pawn_hair_short.svg"
	if hair_style == 1:
		hair_path = "res://assets/colonists/pawn_hair_bob.svg"
	elif hair_style == 2:
		hair_path = "res://assets/colonists/pawn_hair_ponytail.svg"

	_hair_sprite = Sprite2D.new()
	_hair_sprite.texture = load(hair_path) as Texture2D
	_hair_sprite.scale = Vector2(0.48, 0.48)
	_hair_sprite.modulate = Color(str(data.get("hair_color", "#2c221e")))
	_rig.add_child(_hair_sprite)

	# Etiqueta de nombre estilo RimWorld bajo el colono
	var plate := PanelContainer.new()
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(0.11, 0.09, 0.07, 0.78)
	sb.set_corner_radius_all(5)
	sb.content_margin_left = 7
	sb.content_margin_right = 7
	sb.content_margin_top = 2
	sb.content_margin_bottom = 2
	plate.add_theme_stylebox_override("panel", sb)
	plate.position = Vector2(-40, 12)
	plate.custom_minimum_size = Vector2(80, 0)
	add_child(plate)

	_name_label = Label.new()
	_name_label.text = colonist_name
	_name_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_name_label.add_theme_color_override("font_color", Color("#f5ebd6"))
	_name_label.add_theme_font_size_override("font_size", 12)
	plate.add_child(_name_label)


func apply_snapshot(data: Dictionary) -> void:
	var tx := float(data.get("x", 0.0))
	var ty := float(data.get("y", 0.0))
	position = Vector2(tx * TILE_PX, ty * TILE_PX)

	health_value = int(data.get("health", 100))
	mood_value = int(data.get("mood", 80))
	current_activity = str(data.get("activity", "Observando"))

	var fx := float(data.get("facing_x", 0.0))
	var fy := float(data.get("facing_y", 1.0))
	var moving := bool(data.get("is_moving", false))
	var phase := float(data.get("walk_phase", 0.0))

	var flip := fx < -0.15
	_body_sprite.flip_h = flip
	_hair_sprite.flip_h = flip
	_face_sprite.visible = fy >= -0.45
	_face_sprite.position.x = clampf(fx * 2.5, -2.5, 2.5)

	if moving:
		_rig.position.y = -18.0 - absf(sin(phase)) * 3.5
		_rig.rotation = sin(phase) * 0.06
	else:
		_rig.position.y = -18.0
		_rig.rotation = 0.0


func set_selected(selected: bool) -> void:
	is_selected = selected
	if _selection_ring:
		_selection_ring.visible = selected


class _SelectionRing extends Node2D:
	func _draw() -> void:
		draw_arc(Vector2(0, 4), 22.0, 0.0, TAU, 32, Color("#e6b85c", 0.95), 2.5, true)
		draw_circle(Vector2(0, 4), 20.0, Color("#e6b85c", 0.16))
