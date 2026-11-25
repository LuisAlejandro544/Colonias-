# Representación visual modular por capas de un colono estilo RimWorld.
#
# Lógica: Construye un peón 2D apilando sombra, torso (túnica o abrigo),
# cabeza, rostro expresivo, cabello, insignia del Líder de Colonia, globo de
# reunión, herramienta de cobre equipada en mano (hacha, pico, pala, azada o
# espada) y recurso transportado al hombro cuando acarrea hacia el almacén.

class_name ColonistVisual
extends Node2D

const TILE_PX := 64.0

var colonist_id: int = 0
var colonist_name: String = "Colono"
var colonist_role: String = ""
var is_leader: bool = false
var current_activity: String = ""
var equipped_tool_code: String = "copper_shovel"
var equipped_tool_label: String = "Pala de cobre"
var carrying_kind: String = ""
var carrying_amount: int = 0
var health_value: int = 100
var mood_value: int = 80
var is_selected: bool = false

var _rig: Node2D
var _body_sprite: Sprite2D
var _head_sprite: Sprite2D
var _face_sprite: Sprite2D
var _hair_sprite: Sprite2D
var _hand_anchor: Node2D
var _tool_sprite: Sprite2D
var _carry_sprite: Sprite2D
var _leader_crown: Sprite2D
var _meeting_bubble: Sprite2D
var _name_label: Label
var _selection_ring: Node2D

var _tool_textures: Dictionary = {}
var _item_textures: Dictionary = {}


func setup_from_snapshot(data: Dictionary) -> void:
	colonist_id = int(data.get("id", 1))
	colonist_name = str(data.get("name", "Colono"))
	colonist_role = str(data.get("role", "Explorador"))
	is_leader = bool(data.get("is_leader", false))
	_preload_textures()
	_build_layers(data)
	apply_snapshot(data)


func _preload_textures() -> void:
	_tool_textures = {
		"copper_axe": load("res://assets/tools/tool_copper_axe.svg") as Texture2D,
		"copper_pickaxe": load("res://assets/tools/tool_copper_pickaxe.svg") as Texture2D,
		"copper_shovel": load("res://assets/tools/tool_copper_shovel.svg") as Texture2D,
		"copper_hoe": load("res://assets/tools/tool_copper_hoe.svg") as Texture2D,
		"copper_sword": load("res://assets/tools/tool_copper_sword.svg") as Texture2D,
	}
	_item_textures = {
		"wood_logs": load("res://assets/items/item_wood_logs.svg") as Texture2D,
		"stone_chunk": load("res://assets/items/item_stone_chunk.svg") as Texture2D,
		"iron_ore": load("res://assets/items/item_iron_ore.svg") as Texture2D,
		"copper_ore": load("res://assets/items/item_copper_ore.svg") as Texture2D,
		"carrot_seed": load("res://assets/items/item_carrot_seeds.svg") as Texture2D,
		"carrot": load("res://assets/items/item_carrot.svg") as Texture2D,
	}


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

	# Ancla de herramienta de cobre en mano
	_hand_anchor = Node2D.new()
	_hand_anchor.position = Vector2(14, 7)
	_rig.add_child(_hand_anchor)

	_tool_sprite = Sprite2D.new()
	_tool_sprite.scale = Vector2(0.24, 0.24)
	_tool_sprite.position = Vector2(4, -4)
	_hand_anchor.add_child(_tool_sprite)

	# Ítem transportado (cuando acarrea madera, piedra, zanahorias, etc.)
	_carry_sprite = Sprite2D.new()
	_carry_sprite.scale = Vector2(0.22, 0.22)
	_carry_sprite.position = Vector2(-13, 6)
	_carry_sprite.visible = false
	_rig.add_child(_carry_sprite)

	# Insignia del Líder de Colonia
	_leader_crown = Sprite2D.new()
	_leader_crown.texture = load("res://assets/colonists/leader_crown.svg") as Texture2D
	_leader_crown.scale = Vector2(0.36, 0.36)
	_leader_crown.position = Vector2(0, -27)
	_leader_crown.visible = is_leader
	_rig.add_child(_leader_crown)

	# Globo de reunión / instrucciones del líder
	_meeting_bubble = Sprite2D.new()
	_meeting_bubble.texture = load("res://assets/colonists/meeting_bubble.svg") as Texture2D
	_meeting_bubble.scale = Vector2(0.42, 0.42)
	_meeting_bubble.position = Vector2(20, -34)
	_meeting_bubble.visible = false
	_rig.add_child(_meeting_bubble)

	# Etiqueta de nombre estilo RimWorld bajo el colono
	var plate := PanelContainer.new()
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(0.18, 0.13, 0.08, 0.86) if is_leader else Color(0.11, 0.09, 0.07, 0.78)
	if is_leader:
		sb.border_color = Color("#e0b15e")
		sb.set_border_width_all(1)
	sb.set_corner_radius_all(5)
	sb.content_margin_left = 7
	sb.content_margin_right = 7
	sb.content_margin_top = 2
	sb.content_margin_bottom = 2
	plate.add_theme_stylebox_override("panel", sb)
	plate.position = Vector2(-54, 12)
	plate.custom_minimum_size = Vector2(108, 0)
	add_child(plate)

	_name_label = Label.new()
	_name_label.text = ("★ " + colonist_name) if is_leader else colonist_name
	_name_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_name_label.add_theme_color_override("font_color", Color("#ffe39a") if is_leader else Color("#f5ebd6"))
	_name_label.add_theme_font_size_override("font_size", 15)
	plate.add_child(_name_label)


func apply_snapshot(data: Dictionary) -> void:
	var tx := float(data.get("x", 0.0))
	var ty := float(data.get("y", 0.0))
	position = Vector2(tx * TILE_PX, ty * TILE_PX)

	is_leader = bool(data.get("is_leader", is_leader))
	if _leader_crown:
		_leader_crown.visible = is_leader

	health_value = int(data.get("health", 100))
	mood_value = int(data.get("mood", 80))
	current_activity = str(data.get("activity", "Observando"))

	var fx := float(data.get("facing_x", 0.0))
	var fy := float(data.get("facing_y", 1.0))
	var moving := bool(data.get("is_moving", false))
	var working := bool(data.get("is_working", false))
	var is_meeting := bool(data.get("is_meeting", false))
	var phase := float(data.get("walk_phase", 0.0))
	var swing := float(data.get("work_swing_phase", 0.0))

	var flip := fx < -0.15
	_body_sprite.flip_h = flip
	_hair_sprite.flip_h = flip
	_face_sprite.visible = fy >= -0.45
	_face_sprite.position.x = clampf(fx * 2.5, -2.5, 2.5)

	# Herramienta de cobre en mano
	equipped_tool_code = str(data.get("equipped_tool", "copper_sword" if is_leader else "copper_shovel"))
	equipped_tool_label = str(data.get("equipped_tool_label", "Espada de cobre"))
	var tool_tex: Texture2D = _tool_textures.get(equipped_tool_code, null)
	if tool_tex != null:
		_tool_sprite.texture = tool_tex
		_tool_sprite.visible = true
	else:
		_tool_sprite.visible = false

	_hand_anchor.position.x = -14.0 if flip else 14.0
	_tool_sprite.flip_h = flip
	if working:
		var swing_angle := sin(swing) * 0.72
		_hand_anchor.rotation = -swing_angle if flip else swing_angle
	else:
		_hand_anchor.rotation = -0.18 if flip else 0.18

	# Recurso cargado hacia el almacén
	carrying_kind = str(data.get("carrying_kind", ""))
	carrying_amount = int(data.get("carrying_amount", 0))
	if not carrying_kind.is_empty() and _item_textures.has(carrying_kind):
		_carry_sprite.texture = _item_textures[carrying_kind]
		_carry_sprite.visible = true
		_carry_sprite.position.x = 13.0 if flip else -13.0
	else:
		_carry_sprite.visible = false

	# Globo de reunión del líder
	if _meeting_bubble:
		_meeting_bubble.visible = is_meeting
		if is_meeting:
			_meeting_bubble.position.y = -34.0 + sin(Time.get_ticks_msec() * 0.008) * 3.0

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
