# Popup modular de Estadísticas y Atributos personales del colono.
#
# Lógica: Muestra los 5 atributos del colono seleccionado (Inteligencia, Fuerza,
# Destreza, Resistencia y Carisma) en escala 1..=20 con su progreso práctico,
# efectos reales en la simulación y el nivel de complejidad de fabricación
# desbloqueado por su Inteligencia (`INT`).

class_name ColonistStatsPopup
extends ColorRect

var _colonist_id: int = -1
var _title_label: Label
var _summary_label: Label
var _craft_tier_label: Label
var _cards_box: VBoxContainer
var _scroll: ScrollContainer


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
	panel.custom_minimum_size = Vector2(800, 520)
	UiTheme.apply_panel_style(panel, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 24, 20)
	center.add_child(panel)

	var root_v := VBoxContainer.new()
	root_v.add_theme_constant_override("separation", 12)
	panel.add_child(root_v)

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 12)
	root_v.add_child(head)

	var head_txt := VBoxContainer.new()
	head_txt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head_txt.add_theme_constant_override("separation", 3)
	head.add_child(head_txt)

	_title_label = Label.new()
	_title_label.text = "Atributos del Colono"
	UiTheme.style_title(_title_label, 26)
	head_txt.add_child(_title_label)

	_summary_label = Label.new()
	_summary_label.text = ""
	_summary_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(_summary_label, 15)
	head_txt.add_child(_summary_label)

	var close_btn := Button.new()
	close_btn.text = "Cerrar"
	UiTheme.style_button(close_btn, "secondary", 48, 18)
	close_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		close_popup()
	)
	head.add_child(close_btn)

	var tier_box := PanelContainer.new()
	UiTheme.apply_panel_style(tier_box, Color("#1e1812"), UiTheme.BORDER_BRASS, 10, 14, 9)
	root_v.add_child(tier_box)

	_craft_tier_label = Label.new()
	_craft_tier_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_craft_tier_label.add_theme_color_override("font_color", UiTheme.TEXT_ACCENT)
	_craft_tier_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(15))
	tier_box.add_child(_craft_tier_label)

	_scroll = ScrollContainer.new()
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.custom_minimum_size = Vector2(0, 320)
	root_v.add_child(_scroll)

	_cards_box = VBoxContainer.new()
	_cards_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_cards_box.add_theme_constant_override("separation", 10)
	_scroll.add_child(_cards_box)

	UiTheme.enable_touch_scroll(_scroll)


func open_for_colonist(colonist: Node) -> void:
	if colonist == null:
		return
	_colonist_id = int(colonist.get("colonist_id"))
	visible = true
	_populate(colonist)


func refresh_if_visible(colonist: Node) -> void:
	if not visible or colonist == null:
		return
	if int(colonist.get("colonist_id")) != _colonist_id:
		return
	_populate(colonist)


func close_popup() -> void:
	visible = false


func _populate(colonist: Node) -> void:
	var cname := str(colonist.get("colonist_name"))
	var leader_tag := " [Líder]" if bool(colonist.get("is_leader")) else ""
	_title_label.text = "Atributos de %s%s" % [cname, leader_tag]
	_summary_label.text = "%s  •  Salud: %d / %d  •  Energía: %d%%  •  Ánimo: %d%%" % [
		str(colonist.get("colonist_role")),
		int(colonist.get("health_value")),
		int(colonist.get("max_health_value")),
		int(colonist.get("energy_value")),
		int(colonist.get("mood_value")),
	]
	_craft_tier_label.text = "Capacidad intelectual (INT %d): %s" % [
		int(colonist.get("stat_int")),
		str(colonist.get("craft_tier_label")),
	]

	for ch in _cards_box.get_children():
		_cards_box.remove_child(ch)
		ch.queue_free()

	var details_var = colonist.get("stats_detail")
	if not (details_var is Array):
		return
	var details: Array = details_var

	for entry_var in details:
		if not (entry_var is Dictionary):
			continue
		var entry: Dictionary = entry_var
		var short_code := str(entry.get("short", "INT"))
		var label_txt := str(entry.get("label", ""))
		var val := int(entry.get("value", 1))
		var max_val := int(entry.get("max_value", 20))
		var xp := float(entry.get("xp", 0.0))
		var xp_next := float(entry.get("xp_next", 30.0))
		var summary := str(entry.get("summary", ""))

		var card := PanelContainer.new()
		card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		UiTheme.apply_panel_style(card, UiTheme.BG_PANEL_ALT, UiTheme.BORDER_BRASS, 12, 14, 10)
		_cards_box.add_child(card)

		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 12)
		card.add_child(row)

		var badge := PanelContainer.new()
		badge.custom_minimum_size = Vector2(68, 56)
		badge.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		UiTheme.apply_panel_style(badge, Color("#1b1510"), UiTheme.BORDER_LIGHT, 10, 8, 6)
		row.add_child(badge)

		var badge_lbl := Label.new()
		badge_lbl.text = "%s\n%d" % [short_code, val]
		badge_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		badge_lbl.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		badge_lbl.add_theme_color_override("font_color", UiTheme.TEXT_ACCENT)
		badge_lbl.add_theme_font_size_override("font_size", UiTheme.scaled_px(16))
		badge.add_child(badge_lbl)

		var info_v := VBoxContainer.new()
		info_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		info_v.add_theme_constant_override("separation", 4)
		row.add_child(info_v)

		var top_h := HBoxContainer.new()
		info_v.add_child(top_h)

		var name_lbl := Label.new()
		name_lbl.text = label_txt
		name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		UiTheme.style_title(name_lbl, 19)
		top_h.add_child(name_lbl)

		var val_lbl := Label.new()
		val_lbl.text = "%d / %d" % [val, max_val]
		val_lbl.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
		val_lbl.add_theme_font_size_override("font_size", UiTheme.scaled_px(16))
		top_h.add_child(val_lbl)

		var bar_row := HBoxContainer.new()
		bar_row.add_theme_constant_override("separation", 8)
		info_v.add_child(bar_row)

		var bar := ProgressBar.new()
		bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		bar.custom_minimum_size = Vector2(0, 14)
		bar.show_percentage = false
		bar.max_value = float(max_val)
		bar.value = float(val)
		var bar_bg := StyleBoxFlat.new()
		bar_bg.bg_color = Color("#1b1510")
		bar_bg.border_color = UiTheme.BORDER_BRASS
		bar_bg.set_border_width_all(1)
		bar_bg.set_corner_radius_all(5)
		bar.add_theme_stylebox_override("background", bar_bg)
		var bar_fill := StyleBoxFlat.new()
		bar_fill.bg_color = Color("#c8933f") if short_code == "INT" else Color("#7da657")
		bar_fill.set_corner_radius_all(4)
		bar.add_theme_stylebox_override("fill", bar_fill)
		bar_row.add_child(bar)

		var xp_lbl := Label.new()
		if val >= max_val or xp_next <= 0.0:
			xp_lbl.text = "Máx."
		else:
			xp_lbl.text = "Práctica %d/%d" % [int(floor(xp)), int(round(xp_next))]
		xp_lbl.add_theme_color_override("font_color", UiTheme.TEXT_MUTED)
		xp_lbl.add_theme_font_size_override("font_size", UiTheme.scaled_px(13))
		bar_row.add_child(xp_lbl)

		var sum_lbl := Label.new()
		sum_lbl.text = summary
		sum_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		UiTheme.style_subtitle(sum_lbl, 14)
		info_v.add_child(sum_lbl)

	UiTheme.make_scroll_pass(_cards_box)
