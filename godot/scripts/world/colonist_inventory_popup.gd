# Popup modular de Inventario y Ranuras de Equipo de un colono.
#
# Lógica: Muestra las ranuras personales del colono inspeccionado (Casco futuro,
# Armadura futura, Espalda, Mano Principal y Mochila / Carga Actual) con iconos
# vectoriales `.svg` actualizados en tiempo real.

class_name ColonistInventoryPopup
extends ColorRect

var _title_label: Label
var _subtitle_label: Label
var _helmet_val: Label
var _helmet_empty: Label
var _armor_val: Label
var _armor_empty: Label
var _back_val: Label
var _back_icon: TextureRect
var _back_empty: Label
var _hand_val: Label
var _hand_icon: TextureRect
var _hand_empty: Label
var _carry_val: Label
var _carry_icon: TextureRect
var _carry_empty: Label


func _ready() -> void:
	visible = false
	color = Color(0.05, 0.04, 0.03, 0.72)
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_build_ui()


func _build_ui() -> void:
	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(center)

	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(740, 0)
	UiTheme.apply_panel_style(panel, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 26, 22)
	center.add_child(panel)

	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 14)
	panel.add_child(v)

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 12)
	v.add_child(head)

	var title_v := VBoxContainer.new()
	title_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title_v.add_theme_constant_override("separation", 3)
	head.add_child(title_v)

	_title_label = Label.new()
	_title_label.text = "Inventario del Colono"
	UiTheme.style_title(_title_label, 26)
	title_v.add_child(_title_label)

	_subtitle_label = Label.new()
	_subtitle_label.text = "Equipo personal y carga actual"
	UiTheme.style_subtitle(_subtitle_label, 15)
	title_v.add_child(_subtitle_label)

	var close_btn := Button.new()
	close_btn.text = "Cerrar"
	close_btn.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	UiTheme.style_button(close_btn, "secondary", 46, 18)
	close_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		close_popup()
	)
	head.add_child(close_btn)

	var equip_heading := Label.new()
	equip_heading.text = "Ranuras de Equipo y Herramientas"
	UiTheme.style_section_label(equip_heading, 16)
	v.add_child(equip_heading)

	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 12)
	grid.add_theme_constant_override("v_separation", 12)
	v.add_child(grid)

	var helmet_slot := _make_equipment_slot("Casco (Futuro)", "Sin equipar", "—")
	_helmet_val = helmet_slot["val"]
	_helmet_empty = helmet_slot["empty"]
	grid.add_child(helmet_slot["card"])

	var armor_slot := _make_equipment_slot("Armadura (Futura)", "Sin equipar", "—")
	_armor_val = armor_slot["val"]
	_armor_empty = armor_slot["empty"]
	grid.add_child(armor_slot["card"])

	var back_slot := _make_equipment_slot("Espalda", "Vacío", "—")
	_back_val = back_slot["val"]
	_back_icon = back_slot["icon"]
	_back_empty = back_slot["empty"]
	grid.add_child(back_slot["card"])

	var hand_slot := _make_equipment_slot("Mano Principal", "Vacío", "—")
	_hand_val = hand_slot["val"]
	_hand_icon = hand_slot["icon"]
	_hand_empty = hand_slot["empty"]
	grid.add_child(hand_slot["card"])

	var carry_heading := Label.new()
	carry_heading.text = "Transporte Personal"
	UiTheme.style_section_label(carry_heading, 16)
	v.add_child(carry_heading)

	var carry_slot := _make_equipment_slot("Mochila / Carga Actual", "Sin carga", "—")
	_carry_val = carry_slot["val"]
	_carry_icon = carry_slot["icon"]
	_carry_empty = carry_slot["empty"]
	v.add_child(carry_slot["card"])


func _make_equipment_slot(slot_title: String, default_text: String, empty_glyph: String) -> Dictionary:
	var card := PanelContainer.new()
	card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.apply_panel_style(card, UiTheme.BG_PANEL_ALT, UiTheme.BORDER_BRASS, 12, 14, 11)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 12)
	row.alignment = BoxContainer.ALIGNMENT_CENTER
	card.add_child(row)

	var icon_box := PanelContainer.new()
	icon_box.custom_minimum_size = Vector2(58, 58)
	icon_box.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	icon_box.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	UiTheme.apply_panel_style(icon_box, Color("#1b1510"), UiTheme.BORDER_BRASS, 10, 6, 6)
	row.add_child(icon_box)

	var empty_lbl := Label.new()
	empty_lbl.text = empty_glyph
	empty_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	empty_lbl.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	empty_lbl.add_theme_color_override("font_color", Color("#6e5c47"))
	empty_lbl.add_theme_font_size_override("font_size", UiTheme.scaled_px(20))
	icon_box.add_child(empty_lbl)

	var icon := TextureRect.new()
	icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	icon.custom_minimum_size = Vector2(42, 42)
	icon.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	icon.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	icon_box.add_child(icon)

	var txt_v := VBoxContainer.new()
	txt_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	txt_v.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	txt_v.add_theme_constant_override("separation", 2)
	row.add_child(txt_v)

	var lbl_head := Label.new()
	lbl_head.text = slot_title
	UiTheme.style_section_label(lbl_head, 15)
	txt_v.add_child(lbl_head)

	var lbl_val := Label.new()
	lbl_val.text = default_text
	lbl_val.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	lbl_val.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
	lbl_val.add_theme_font_size_override("font_size", UiTheme.scaled_px(17))
	txt_v.add_child(lbl_val)

	return {"card": card, "val": lbl_val, "icon": icon, "empty": empty_lbl}


func open_for_colonist(colonist: Node, tool_icons: Dictionary, item_icons: Dictionary) -> void:
	visible = true
	if colonist != null:
		refresh_colonist(colonist, tool_icons, item_icons)


func refresh_if_visible(colonist: Node, tool_icons: Dictionary, item_icons: Dictionary) -> void:
	if visible and colonist != null:
		refresh_colonist(colonist, tool_icons, item_icons)


func refresh_colonist(colonist: Node, tool_icons: Dictionary, item_icons: Dictionary) -> void:
	if colonist == null:
		return
	var leader_tag := " [Líder]" if bool(colonist.get("is_leader")) else ""
	_title_label.text = "Inventario de %s%s" % [str(colonist.get("colonist_name")), leader_tag]
	_subtitle_label.text = "%s  •  Energía: %d%%  •  Hidratación: %d%%  •  Salud: %d%%  •  Ánimo: %d%%" % [
		str(colonist.get("colonist_role")),
		int(colonist.get("energy_value")),
		int(colonist.get("hydration_value")),
		int(colonist.get("health_value")),
		int(colonist.get("mood_value")),
	]

	var helmet := str(colonist.get("helmet_slot"))
	_helmet_val.text = helmet if not helmet.is_empty() else "Sin equipar (próximamente)"
	_helmet_val.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT if not helmet.is_empty() else UiTheme.TEXT_MUTED)
	if _helmet_empty:
		_helmet_empty.visible = helmet.is_empty()

	var armor := str(colonist.get("armor_slot"))
	_armor_val.text = armor if not armor.is_empty() else "Sin equipar (próximamente)"
	_armor_val.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT if not armor.is_empty() else UiTheme.TEXT_MUTED)
	if _armor_empty:
		_armor_empty.visible = armor.is_empty()

	var back_code := str(colonist.get("back_tool_code"))
	var back_label := str(colonist.get("back_tool_label"))
	var back_tex: Texture2D = tool_icons.get(back_code, null)
	_back_val.text = back_label if back_code != "none" else "Vacío"
	_back_val.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT if back_code != "none" else UiTheme.TEXT_MUTED)
	_back_icon.texture = back_tex
	_back_empty.visible = back_tex == null

	var hand_code := str(colonist.get("main_hand_tool_code"))
	var hand_label := str(colonist.get("main_hand_tool_label"))
	var hand_tex: Texture2D = tool_icons.get(hand_code, null)
	_hand_val.text = hand_label if hand_code != "none" else "Manos libres"
	_hand_val.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT if hand_code != "none" else UiTheme.TEXT_MUTED)
	_hand_icon.texture = hand_tex
	_hand_empty.visible = hand_tex == null

	var carry_kind := str(colonist.get("carrying_kind"))
	var carry_label := str(colonist.get("carrying_label"))
	var carry_amt := int(colonist.get("carrying_amount"))
	if not carry_kind.is_empty() and carry_amt > 0:
		var carry_tex: Texture2D = item_icons.get(carry_kind, null)
		_carry_val.text = "%s (x%d)" % [carry_label if not carry_label.is_empty() else carry_kind, carry_amt]
		_carry_val.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
		_carry_icon.texture = carry_tex
		_carry_empty.visible = carry_tex == null
	else:
		_carry_val.text = "Mochila vacía — lista para transportar recursos"
		_carry_val.add_theme_color_override("font_color", UiTheme.TEXT_MUTED)
		_carry_icon.texture = null
		_carry_empty.visible = true


func close_popup() -> void:
	visible = false
