# Popup modular de gestión de Trabajos, Roles (máx. 5) y Niveles (1 al 10).
#
# Lógica: Permite seleccionar a cualquier colono de la colonia, ver los 8 roles
# disponibles (Leñador, Minero, Transportador, Constructor, Agricultor, Botánico,
# Explorador y Líder de Colonia), activar o desactivar hasta 5 roles por colono y
# consultar en tiempo real su nivel (1 a 10), barra de experiencia y bonificaciones
# de velocidad, carga y rendimiento.

class_name JobsPopup
extends ColorRect

signal role_toggled(colonist_id: int, role_code: String, enabled: bool)
signal closed

var _colonists_cache: Array = []
var _selected_colonist_id: int = -1
var _colony_sim: Node = null

var _colonist_tabs_box: HFlowContainer
var _summary_label: Label
var _feedback_label: Label
var _roles_grid: GridContainer
var _scroll: ScrollContainer
var _role_cards: Dictionary = {}
var _refresh_accum: float = 0.0


func _ready() -> void:
	visible = false
	color = Color(0.05, 0.04, 0.03, 0.74)
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_build_ui()


func _process(delta: float) -> void:
	if not visible or _colony_sim == null or _selected_colonist_id <= 0:
		return
	_refresh_accum += delta
	if _refresh_accum >= 0.35:
		_refresh_accum = 0.0
		_refresh_selected_roles()


func _build_ui() -> void:
	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(center)

	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(960, 580)
	UiTheme.apply_panel_style(panel, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 24, 20)
	center.add_child(panel)

	var root_v := VBoxContainer.new()
	root_v.add_theme_constant_override("separation", 12)
	panel.add_child(root_v)

	# Cabecera
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 12)
	root_v.add_child(head)

	var head_txt := VBoxContainer.new()
	head_txt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head_txt.add_theme_constant_override("separation", 3)
	head.add_child(head_txt)

	var title := Label.new()
	title.text = "Trabajos y Niveles de Colonos"
	UiTheme.style_title(title, 28)
	head_txt.add_child(title)

	var sub := Label.new()
	sub.text = "Asigna hasta 5 roles por colono (Máx. Nivel 10). La experiencia mejora rapidez, carga y rendimiento."
	sub.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(sub, 15)
	head_txt.add_child(sub)

	var close_btn := Button.new()
	close_btn.text = "Cerrar"
	UiTheme.style_button(close_btn, "secondary", 48, 18)
	close_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		close_popup()
	)
	head.add_child(close_btn)

	# Selector rápido de colonos
	_colonist_tabs_box = HFlowContainer.new()
	_colonist_tabs_box.add_theme_constant_override("h_separation", 8)
	_colonist_tabs_box.add_theme_constant_override("v_separation", 8)
	root_v.add_child(_colonist_tabs_box)

	# Franja de resumen del colono seleccionado
	var status_bar := PanelContainer.new()
	UiTheme.apply_panel_style(status_bar, Color("#1e1812"), UiTheme.BORDER_BRASS, 10, 14, 9)
	root_v.add_child(status_bar)

	var status_row := HBoxContainer.new()
	status_row.add_theme_constant_override("separation", 12)
	status_bar.add_child(status_row)

	_summary_label = Label.new()
	_summary_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_summary_label.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
	_summary_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(17))
	status_row.add_child(_summary_label)

	_feedback_label = Label.new()
	_feedback_label.add_theme_color_override("font_color", UiTheme.TEXT_ACCENT)
	_feedback_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(15))
	status_row.add_child(_feedback_label)

	# Lista desplazable con los 8 roles
	_scroll = ScrollContainer.new()
	_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll.custom_minimum_size = Vector2(0, 340)
	root_v.add_child(_scroll)

	_roles_grid = GridContainer.new()
	_roles_grid.columns = 2
	_roles_grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_roles_grid.add_theme_constant_override("h_separation", 12)
	_roles_grid.add_theme_constant_override("v_separation", 12)
	_scroll.add_child(_roles_grid)

	UiTheme.enable_touch_scroll(_scroll)


func open_for_colonists(colonists: Array, focus_colonist_id: int, colony_sim: Node) -> void:
	_colonists_cache = colonists
	_colony_sim = colony_sim
	_feedback_label.text = ""
	if focus_colonist_id > 0:
		_selected_colonist_id = focus_colonist_id
	elif not colonists.is_empty():
		var first = colonists[0]
		if first is Node:
			_selected_colonist_id = int(first.get("colonist_id"))
	visible = true
	_refresh_colonist_tabs()
	_refresh_selected_roles()


func refresh_from_sim(colonists: Array, colony_sim: Node) -> void:
	_colonists_cache = colonists
	_colony_sim = colony_sim
	if not visible:
		return
	_refresh_colonist_tabs()
	_refresh_selected_roles()


func show_feedback(msg: String, is_error: bool) -> void:
	_feedback_label.text = msg
	_feedback_label.add_theme_color_override(
		"font_color",
		Color("#e58a76") if is_error else Color("#a7d489")
	)


func close_popup() -> void:
	visible = false
	closed.emit()


func _refresh_colonist_tabs() -> void:
	for ch in _colonist_tabs_box.get_children():
		_colonist_tabs_box.remove_child(ch)
		ch.queue_free()

	var valid_ids: Array[int] = []
	for c_var in _colonists_cache:
		if not (c_var is Node):
			continue
		var c := c_var as Node
		var cid := int(c.get("colonist_id"))
		valid_ids.append(cid)
		var cname := str(c.get("colonist_name"))
		var is_leader := bool(c.get("is_leader"))
		var roles_arr = c.get("assigned_roles_labels")
		var r_count := roles_arr.size() if roles_arr is Array else 0

		var btn := Button.new()
		btn.text = "%s%s (%d/5)" % ["★ " if is_leader else "", cname, r_count]
		UiTheme.style_button(btn, "amber" if cid == _selected_colonist_id else "secondary", 42, 16)
		btn.pressed.connect(func():
			UiTheme.trigger_touch_feedback()
			_selected_colonist_id = cid
			_feedback_label.text = ""
			_refresh_colonist_tabs()
			_refresh_selected_roles()
		)
		_colonist_tabs_box.add_child(btn)

	if not valid_ids.has(_selected_colonist_id) and not valid_ids.is_empty():
		_selected_colonist_id = valid_ids[0]


func _refresh_selected_roles() -> void:
	if _colony_sim == null or _selected_colonist_id <= 0:
		return

	var selected_node: Node = null
	for c_var in _colonists_cache:
		if c_var is Node and int(c_var.get("colonist_id")) == _selected_colonist_id:
			selected_node = c_var
			break

	var details_var: Variant = _colony_sim.call("get_colonist_roles_detail", _selected_colonist_id)
	if not (details_var is Array):
		return
	var details: Array = details_var

	var active_count := 0
	for item_var in details:
		if item_var is Dictionary and bool(item_var.get("assigned", false)):
			active_count += 1

	if selected_node != null:
		var leader_tag := " [Líder]" if bool(selected_node.get("is_leader")) else ""
		_summary_label.text = "%s%s  •  Roles activos: %d / 5  •  %s" % [
			str(selected_node.get("colonist_name")),
			leader_tag,
			active_count,
			str(selected_node.get("current_activity")),
		]

	if _role_cards.is_empty():
		_build_role_cards(details)

	for item_var in details:
		if not (item_var is Dictionary):
			continue
		var item: Dictionary = item_var
		var code := str(item.get("code", ""))
		if not _role_cards.has(code):
			continue
		var refs: Dictionary = _role_cards[code]
		var assigned := bool(item.get("assigned", false))
		var order_num := int(item.get("order", 0))
		var level := int(item.get("level", 1))
		var max_level := int(item.get("max_level", 10))
		var xp := float(item.get("xp", 0.0))
		var xp_next := float(item.get("xp_next", 25.0))
		var effect_txt := str(item.get("effect_summary", ""))

		(refs["level_lbl"] as Label).text = "Nv. %d / %d" % [level, max_level]
		var xp_bar := refs["xp_bar"] as ProgressBar
		var xp_lbl := refs["xp_lbl"] as Label
		if level >= max_level or xp_next <= 0.0:
			xp_bar.max_value = 1.0
			xp_bar.value = 1.0
			xp_lbl.text = "Nivel Máximo"
		else:
			xp_bar.max_value = xp_next
			xp_bar.value = clampf(xp, 0.0, xp_next)
			xp_lbl.text = "XP: %d / %d" % [int(floor(xp)), int(round(xp_next))]

		(refs["effect_lbl"] as Label).text = effect_txt

		var btn := refs["toggle_btn"] as Button
		if assigned:
			btn.text = "Activo (#%d)" % order_num
			UiTheme.style_button(btn, "amber" if code == "colony_leader" else "primary", 42, 16)
		else:
			btn.text = "Asignar"
			UiTheme.style_button(btn, "secondary", 42, 16)
		refs["assigned"] = assigned


func _build_role_cards(details: Array) -> void:
	for item_var in details:
		if not (item_var is Dictionary):
			continue
		var item: Dictionary = item_var
		var code := str(item.get("code", ""))
		var title_txt := str(item.get("label", ""))
		var desc_txt := str(item.get("description", ""))

		var card := PanelContainer.new()
		card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		UiTheme.apply_panel_style(card, UiTheme.BG_PANEL_ALT, UiTheme.BORDER_BRASS, 12, 14, 12)
		_roles_grid.add_child(card)

		var v := VBoxContainer.new()
		v.add_theme_constant_override("separation", 6)
		card.add_child(v)

		var top_row := HBoxContainer.new()
		top_row.add_theme_constant_override("separation", 8)
		v.add_child(top_row)

		var name_lbl := Label.new()
		name_lbl.text = title_txt
		name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		UiTheme.style_title(name_lbl, 20)
		top_row.add_child(name_lbl)

		var level_lbl := Label.new()
		level_lbl.text = "Nv. 1 / 10"
		level_lbl.add_theme_color_override("font_color", UiTheme.TEXT_ACCENT)
		level_lbl.add_theme_font_size_override("font_size", UiTheme.scaled_px(17))
		top_row.add_child(level_lbl)

		var toggle_btn := Button.new()
		toggle_btn.text = "Asignar"
		toggle_btn.pressed.connect(func():
			UiTheme.trigger_touch_feedback()
			var currently_assigned := bool(_role_cards[code].get("assigned", false))
			role_toggled.emit(_selected_colonist_id, code, not currently_assigned)
		)
		top_row.add_child(toggle_btn)

		var desc_lbl := Label.new()
		desc_lbl.text = desc_txt
		desc_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		UiTheme.style_subtitle(desc_lbl, 14)
		v.add_child(desc_lbl)

		var xp_row := HBoxContainer.new()
		xp_row.add_theme_constant_override("separation", 8)
		v.add_child(xp_row)

		var xp_bar := ProgressBar.new()
		xp_bar.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		xp_bar.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		xp_bar.custom_minimum_size = Vector2(0, 14)
		xp_bar.show_percentage = false
		var bar_bg := StyleBoxFlat.new()
		bar_bg.bg_color = Color("#1b1510")
		bar_bg.border_color = UiTheme.BORDER_BRASS
		bar_bg.set_border_width_all(1)
		bar_bg.set_corner_radius_all(5)
		xp_bar.add_theme_stylebox_override("background", bar_bg)
		var bar_fill := StyleBoxFlat.new()
		bar_fill.bg_color = Color("#c8933f")
		bar_fill.set_corner_radius_all(4)
		xp_bar.add_theme_stylebox_override("fill", bar_fill)
		xp_row.add_child(xp_bar)

		var xp_lbl := Label.new()
		xp_lbl.text = "XP: 0 / 25"
		xp_lbl.add_theme_color_override("font_color", UiTheme.TEXT_MUTED)
		xp_lbl.add_theme_font_size_override("font_size", UiTheme.scaled_px(13))
		xp_row.add_child(xp_lbl)

		var effect_lbl := Label.new()
		effect_lbl.text = ""
		effect_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		effect_lbl.add_theme_color_override("font_color", Color("#d9be82"))
		effect_lbl.add_theme_font_size_override("font_size", UiTheme.scaled_px(14))
		v.add_child(effect_lbl)

		_role_cards[code] = {
			"level_lbl": level_lbl,
			"xp_bar": xp_bar,
			"xp_lbl": xp_lbl,
			"effect_lbl": effect_lbl,
			"toggle_btn": toggle_btn,
			"assigned": false,
		}

	UiTheme.make_scroll_pass(_roles_grid)
