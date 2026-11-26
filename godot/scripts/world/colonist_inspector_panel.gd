# Panel modular de inspección rápida del colono seleccionado en el mundo.
#
# Lógica: Muestra en la esquina inferior izquierda el nombre, jerarquía,
# roles activos (x/5), actividad actual, herramientas en mano/espalda,
# energía, salud máxima, ánimo e Inteligencia (`INT`) del colono tocado, junto
# a los accesos de Inventario, Atributos, Trabajos y Seguir cámara.

class_name ColonistInspectorPanel
extends PanelContainer

signal inventory_requested
signal stats_requested
signal jobs_requested
signal follow_toggled(enabled: bool)

var _name_label: Label
var _meta_label: Label
var _follow_btn: Button
var _following: bool = false


func _ready() -> void:
	visible = false
	anchor_left = 0.0
	anchor_top = 1.0
	anchor_right = 0.0
	anchor_bottom = 1.0
	offset_left = 16.0
	offset_right = 16.0
	offset_top = -94.0
	offset_bottom = -94.0
	grow_horizontal = Control.GROW_DIRECTION_END
	grow_vertical = Control.GROW_DIRECTION_BEGIN
	custom_minimum_size = Vector2(580, 0)
	UiTheme.apply_panel_style(self, Color(0.16, 0.13, 0.10, 0.95), UiTheme.BORDER_LIGHT, 14, 18, 14)
	_build_ui()


func _build_ui() -> void:
	var insp_row := HBoxContainer.new()
	insp_row.add_theme_constant_override("separation", 12)
	add_child(insp_row)

	var insp_v := VBoxContainer.new()
	insp_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	insp_v.add_theme_constant_override("separation", 5)
	insp_row.add_child(insp_v)

	_name_label = Label.new()
	_name_label.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
	_name_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(20))
	insp_v.add_child(_name_label)

	_meta_label = Label.new()
	_meta_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_meta_label.add_theme_color_override("font_color", UiTheme.TEXT_MUTED)
	_meta_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(15))
	insp_v.add_child(_meta_label)

	var insp_btns := GridContainer.new()
	insp_btns.columns = 2
	insp_btns.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	insp_btns.add_theme_constant_override("h_separation", 8)
	insp_btns.add_theme_constant_override("v_separation", 8)
	insp_row.add_child(insp_btns)

	var inv_open_btn := Button.new()
	inv_open_btn.text = "Inventario"
	UiTheme.style_button(inv_open_btn, "primary", 46, 16)
	inv_open_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		inventory_requested.emit()
	)
	insp_btns.add_child(inv_open_btn)

	var stats_open_btn := Button.new()
	stats_open_btn.text = "Atributos"
	UiTheme.style_button(stats_open_btn, "secondary", 46, 16)
	stats_open_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		stats_requested.emit()
	)
	insp_btns.add_child(stats_open_btn)

	var jobs_open_btn := Button.new()
	jobs_open_btn.text = "Trabajos"
	UiTheme.style_button(jobs_open_btn, "amber", 46, 16)
	jobs_open_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		jobs_requested.emit()
	)
	insp_btns.add_child(jobs_open_btn)

	_follow_btn = Button.new()
	_follow_btn.text = "Seguir"
	UiTheme.style_button(_follow_btn, "secondary", 46, 16)
	_follow_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_following = not _following
		UiTheme.style_button(_follow_btn, "amber" if _following else "secondary", 46, 16)
		follow_toggled.emit(_following)
	)
	insp_btns.add_child(_follow_btn)


func show_colonist(colonist: Node) -> void:
	if colonist == null:
		visible = false
		_following = false
		if _follow_btn:
			UiTheme.style_button(_follow_btn, "secondary", 46, 16)
		return

	visible = true
	var leader_tag := " [Líder]" if bool(colonist.get("is_leader")) else ""
	_name_label.text = "%s%s — %s" % [
		str(colonist.get("colonist_name")),
		leader_tag,
		str(colonist.get("colonist_role")),
	]
	var hand_txt := str(colonist.get("main_hand_tool_label"))
	var back_txt := str(colonist.get("back_tool_label"))
	var carry_kind := str(colonist.get("carrying_kind"))
	var carry_amt := int(colonist.get("carrying_amount"))
	var carry_line := ""
	if not carry_kind.is_empty() and carry_amt > 0:
		carry_line = "  •  Carga: x%d" % carry_amt

	var roles_var = colonist.get("assigned_roles_labels")
	var roles_txt := "Sin roles"
	var roles_count := 0
	if roles_var is Array and not roles_var.is_empty():
		roles_count = roles_var.size()
		var joined := ""
		for i in range(roles_count):
			if i > 0:
				joined += ", "
			joined += str(roles_var[i])
		roles_txt = joined

	_meta_label.text = "Roles (%d/5): %s\nActividad: %s\nMano: %s  •  Espalda: %s%s\nEnergía: %d%%  •  Salud: %d/%d  •  Ánimo: %d%%  •  INT: %d" % [
		roles_count,
		roles_txt,
		str(colonist.get("current_activity")),
		hand_txt,
		back_txt,
		carry_line,
		int(colonist.get("energy_value")),
		int(colonist.get("health_value")),
		int(colonist.get("max_health_value")),
		int(colonist.get("mood_value")),
		int(colonist.get("stat_int")),
	]
