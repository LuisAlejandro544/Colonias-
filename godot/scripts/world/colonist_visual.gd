# Representación visual modular por capas de un colono estilo RimWorld.
#
# Lógica: Construye un peón 2D apilando sombra, herramienta en la espalda, torso
# (túnica o abrigo), cabeza, rostro expresivo, cabello, insignia del Líder de
# Colonia, globo de reunión, herramienta equipada o cubo de riego/vaso con agua
# en la mano principal, recurso transportado y animación de sueño Zzz.

class_name ColonistVisual
extends Node2D

const TILE_PX := 64.0

var colonist_id: int = 0
var colonist_name: String = "Colono"
var colonist_role: String = ""
var assigned_roles_labels: Array = []
var is_leader: bool = false
var current_activity: String = ""
var equipped_tool_code: String = "copper_shovel"
var equipped_tool_label: String = "Pala de cobre"
var main_hand_tool_code: String = "copper_shovel"
var main_hand_tool_label: String = "Pala de cobre"
var back_tool_code: String = "copper_axe"
var back_tool_label: String = "Hacha de cobre"
var helmet_slot: String = ""
var armor_slot: String = ""
var carrying_kind: String = ""
var carrying_label: String = ""
var carrying_amount: int = 0
var health_value: int = 100
var max_health_value: int = 100
var mood_value: int = 80
var energy_value: int = 100
var hydration_value: int = 100
var stat_int: int = 3
var stat_str: int = 4
var stat_dex: int = 4
var stat_end: int = 4
var stat_cha: int = 3
var craft_tier_label: String = ""
var stats_detail: Array = []
var is_sleeping: bool = false
var is_drinking: bool = false
var is_watering: bool = false
var is_selected: bool = false

var _rig: Node2D
var _shadow_sprite: Sprite2D
var _back_tool_sprite: Sprite2D
var _body_sprite: Sprite2D
var _head_sprite: Sprite2D
var _face_sprite: Sprite2D
var _hair_sprite: Sprite2D
var _hand_anchor: Node2D
var _tool_sprite: Sprite2D
var _carry_sprite: Sprite2D
var _leader_crown: Sprite2D
var _meeting_bubble: Sprite2D
var _sleep_zzz_a: Sprite2D
var _sleep_zzz_b: Sprite2D
var _name_label: Label
var _selection_ring: Node2D

var _sleep_anim_time: float = 0.0
var _tool_textures: Dictionary = {}
var _item_textures: Dictionary = {}


func setup_from_snapshot(data: Dictionary) -> void:
	colonist_id = int(data.get("id", 1))
	colonist_name = str(data.get("name", "Colono"))
	colonist_role = str(data.get("role", "Explorador"))
	is_leader = bool(data.get("is_leader", false))
	_sleep_anim_time = float(colonist_id) * 0.7
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
		"coal": load("res://assets/items/item_coal.svg") as Texture2D,
		"copper_ingot": load("res://assets/items/item_copper_ingot.svg") as Texture2D,
		"carrot_seed": load("res://assets/items/item_carrot_seeds.svg") as Texture2D,
		"carrot": load("res://assets/items/item_carrot.svg") as Texture2D,
		"potato_seed": load("res://assets/items/item_potato_seeds.svg") as Texture2D,
		"potato": load("res://assets/items/item_potato.svg") as Texture2D,
		"wheat_seed": load("res://assets/items/item_wheat_seeds.svg") as Texture2D,
		"wheat": load("res://assets/items/item_wheat.svg") as Texture2D,
		"beet_seed": load("res://assets/items/item_beet_seeds.svg") as Texture2D,
		"beet": load("res://assets/items/item_beet.svg") as Texture2D,
		"raw_chicken_breast": load("res://assets/items/item_raw_chicken_breast.svg") as Texture2D,
		"raw_chicken_thigh": load("res://assets/items/item_raw_chicken_thigh.svg") as Texture2D,
		"raw_pork_belly": load("res://assets/items/item_raw_pork_belly.svg") as Texture2D,
		"raw_pork_ribs": load("res://assets/items/item_raw_pork_ribs.svg") as Texture2D,
		"raw_mutton_shoulder": load("res://assets/items/item_raw_mutton_shoulder.svg") as Texture2D,
		"raw_mutton_leg": load("res://assets/items/item_raw_mutton_leg.svg") as Texture2D,
		"raw_beef_sirloin": load("res://assets/items/item_raw_beef_sirloin.svg") as Texture2D,
		"raw_beef_brisket": load("res://assets/items/item_raw_beef_brisket.svg") as Texture2D,
		"animal_bones": load("res://assets/items/item_animal_bones.svg") as Texture2D,
		"wood_plank": load("res://assets/items/item_wood_planks.svg") as Texture2D,
		"wood_bucket": load("res://assets/items/item_wood_bucket.svg") as Texture2D,
		"wood_cup": load("res://assets/items/item_wood_cup_empty.svg") as Texture2D,
		"wood_cup_filled": load("res://assets/items/item_wood_cup.svg") as Texture2D,
		"wood_door_item": load("res://assets/items/item_wood_door.svg") as Texture2D,
		"wood_crate": load("res://assets/items/item_wood_crate.svg") as Texture2D,
		"stone_bricks": load("res://assets/items/item_stone_bricks.svg") as Texture2D,
		"stone_mortar": load("res://assets/items/item_stone_mortar.svg") as Texture2D,
		"stone_millstone": load("res://assets/items/item_stone_millstone.svg") as Texture2D,
		"stone_statuette": load("res://assets/items/item_stone_statuette.svg") as Texture2D,
	}


func _build_layers(data: Dictionary) -> void:
	_selection_ring = _SelectionRing.new()
	_selection_ring.visible = false
	add_child(_selection_ring)

	_shadow_sprite = Sprite2D.new()
	_shadow_sprite.texture = load("res://assets/colonists/pawn_shadow.svg") as Texture2D
	_shadow_sprite.scale = Vector2(0.46, 0.46)
	_shadow_sprite.position = Vector2(0, -14)
	add_child(_shadow_sprite)

	_rig = Node2D.new()
	_rig.position = Vector2(0, -18)
	add_child(_rig)

	# Herramienta o arma colgada en la espalda (detrás del torso)
	_back_tool_sprite = Sprite2D.new()
	_back_tool_sprite.scale = Vector2(0.24, 0.24)
	_back_tool_sprite.position = Vector2(-5, -3)
	_back_tool_sprite.rotation = -0.62
	_back_tool_sprite.visible = false
	_rig.add_child(_back_tool_sprite)

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

	# Ancla de herramienta en mano principal
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

	# Partículas animadas de sueño (.svg Zzz) independientes de la rotación del cuerpo
	var zzz_tex := load("res://assets/colonists/sleep_zzz.svg") as Texture2D
	_sleep_zzz_a = Sprite2D.new()
	_sleep_zzz_a.texture = zzz_tex
	_sleep_zzz_a.visible = false
	add_child(_sleep_zzz_a)

	_sleep_zzz_b = Sprite2D.new()
	_sleep_zzz_b.texture = zzz_tex
	_sleep_zzz_b.visible = false
	add_child(_sleep_zzz_b)

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


func _process(delta: float) -> void:
	if not is_sleeping:
		return
	_sleep_anim_time += delta
	_animate_zzz_sprite(_sleep_zzz_a, fmod(_sleep_anim_time * 0.55, 1.0))
	_animate_zzz_sprite(_sleep_zzz_b, fmod(_sleep_anim_time * 0.55 + 0.5, 1.0))
	if _rig:
		var breath := 1.0 + sin(_sleep_anim_time * 2.5) * 0.035
		_rig.scale = Vector2(breath, 1.0 - (breath - 1.0) * 0.5)


func _animate_zzz_sprite(sprite: Sprite2D, phase: float) -> void:
	if sprite == null:
		return
	var rise := phase * 30.0
	var sway := sin(phase * TAU) * 5.0
	sprite.position = Vector2(10.0 + phase * 10.0 + sway, -18.0 - rise)
	var sc := lerpf(0.22, 0.38, phase)
	sprite.scale = Vector2(sc, sc)
	var alpha := sin(phase * PI)
	sprite.modulate = Color(1.0, 1.0, 1.0, clampf(alpha * 1.15, 0.0, 1.0))


func apply_snapshot(data: Dictionary) -> void:
	var tx := float(data.get("x", 0.0))
	var ty := float(data.get("y", 0.0))
	position = Vector2(tx * TILE_PX, ty * TILE_PX)

	is_leader = bool(data.get("is_leader", is_leader))
	colonist_role = str(data.get("role", colonist_role))
	var roles_var = data.get("assigned_roles_labels", [])
	if roles_var is Array:
		assigned_roles_labels = roles_var
	if _leader_crown:
		_leader_crown.visible = is_leader
	if _name_label:
		_name_label.text = ("★ " + colonist_name) if is_leader else colonist_name
		_name_label.add_theme_color_override("font_color", Color("#ffe39a") if is_leader else Color("#f5ebd6"))

	health_value = int(data.get("health", 100))
	max_health_value = int(data.get("max_health", 100))
	mood_value = int(data.get("mood", 80))
	energy_value = int(data.get("energy", 100))
	hydration_value = int(data.get("hydration", 100))
	stat_int = int(data.get("stat_int", 3))
	stat_str = int(data.get("stat_str", 4))
	stat_dex = int(data.get("stat_dex", 4))
	stat_end = int(data.get("stat_end", 4))
	stat_cha = int(data.get("stat_cha", 3))
	craft_tier_label = str(data.get("craft_tier_label", ""))
	var sd_var = data.get("stats_detail", [])
	if sd_var is Array:
		stats_detail = sd_var
	is_sleeping = bool(data.get("is_sleeping", false))
	is_drinking = bool(data.get("is_drinking", false))
	is_watering = bool(data.get("is_watering", false))
	current_activity = str(data.get("activity", "Observando"))

	helmet_slot = str(data.get("helmet_slot", ""))
	armor_slot = str(data.get("armor_slot", ""))

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
	_face_sprite.visible = true
	_face_sprite.position = Vector2(clampf(fx * 2.5, -2.5, 2.5), clampf(fy * 1.2, -1.5, 1.5))

	# Herramienta en la espalda
	back_tool_code = str(data.get("back_tool", "none"))
	back_tool_label = str(data.get("back_tool_label", "Vacío"))
	var back_tex: Texture2D = _tool_textures.get(back_tool_code, null)
	if back_tex != null and back_tool_code != "none":
		_back_tool_sprite.texture = back_tex
		_back_tool_sprite.visible = true
		_back_tool_sprite.flip_h = not flip
		_back_tool_sprite.position.x = 6.0 if flip else -6.0
		_back_tool_sprite.rotation = 0.62 if flip else -0.62
	else:
		_back_tool_sprite.visible = false

	# Herramienta en la mano principal
	main_hand_tool_code = str(data.get("main_hand_tool", data.get("equipped_tool", "copper_shovel")))
	main_hand_tool_label = str(data.get("main_hand_tool_label", data.get("equipped_tool_label", "Pala de cobre")))
	equipped_tool_code = main_hand_tool_code
	equipped_tool_label = main_hand_tool_label

	var cup_tex: Texture2D = _item_textures.get("wood_cup_filled", null)
	var bucket_tex: Texture2D = _item_textures.get("wood_bucket", null)
	if is_drinking and not is_sleeping and cup_tex != null:
		_tool_sprite.texture = cup_tex
		_tool_sprite.visible = true
	elif is_watering and not is_sleeping and bucket_tex != null:
		_tool_sprite.texture = bucket_tex
		_tool_sprite.visible = true
	else:
		var tool_tex: Texture2D = _tool_textures.get(main_hand_tool_code, null)
		if tool_tex != null and main_hand_tool_code != "none" and not is_sleeping:
			_tool_sprite.texture = tool_tex
			_tool_sprite.visible = true
		else:
			_tool_sprite.visible = false

	_hand_anchor.position.x = -14.0 if flip else 14.0
	_tool_sprite.flip_h = flip
	if is_drinking and not is_sleeping:
		var sip := sin(swing) * 0.25
		_hand_anchor.rotation = (0.78 + sip) if flip else (-0.78 - sip)
	elif is_watering and not is_sleeping:
		var pour := (0.42 + sin(swing) * 0.28) if working else 0.12
		_hand_anchor.rotation = -pour if flip else pour
	elif working:
		var swing_angle := sin(swing) * 0.72
		_hand_anchor.rotation = -swing_angle if flip else swing_angle
	else:
		_hand_anchor.rotation = -0.18 if flip else 0.18

	# Recurso cargado en mochila/brazos
	carrying_kind = str(data.get("carrying_kind", ""))
	carrying_label = str(data.get("carrying_label", ""))
	carrying_amount = int(data.get("carrying_amount", 0))
	if not carrying_kind.is_empty() and carrying_amount > 0 and _item_textures.has(carrying_kind):
		_carry_sprite.texture = _item_textures[carrying_kind]
		_carry_sprite.visible = true
		_carry_sprite.position.x = 13.0 if flip else -13.0
	else:
		_carry_sprite.visible = false

	# Globo de reunión del líder
	if _meeting_bubble:
		_meeting_bubble.visible = is_meeting and not is_sleeping
		if is_meeting:
			_meeting_bubble.position.y = -34.0 + sin(Time.get_ticks_msec() * 0.008) * 3.0

	# Estado acostado en el suelo + Zzz animadas al dormir
	if is_sleeping:
		_rig.position = Vector2(0, -6.0)
		_rig.rotation = PI * 0.5 if flip else -PI * 0.5
		if _shadow_sprite:
			_shadow_sprite.position = Vector2(0, -4.0)
			_shadow_sprite.scale = Vector2(0.64, 0.36)
		if _sleep_zzz_a:
			_sleep_zzz_a.visible = true
		if _sleep_zzz_b:
			_sleep_zzz_b.visible = true
	else:
		_rig.scale = Vector2.ONE
		if _shadow_sprite:
			_shadow_sprite.position = Vector2(0, -14.0)
			_shadow_sprite.scale = Vector2(0.46, 0.46)
		if _sleep_zzz_a:
			_sleep_zzz_a.visible = false
		if _sleep_zzz_b:
			_sleep_zzz_b.visible = false
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
