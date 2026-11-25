# Interfaz superpuesta (HUD) del mundo 2D en vista centinela para móvil.
#
# Lógica: Muestra la barra superior (colonia, día, estado del Líder de Colonia,
# inventario de recursos, velocidad y zoom), una barra inferior de órdenes táctiles
# (Observar, Talar, Minar, Almacén, Cultivo de Zanahorias, Pared de Madera y
# Cancelar), el inspector del colono tocado (con su herramienta de cobre y carga)
# y el popup modal del menú en partida.

class_name WorldHud
extends CanvasLayer

signal speed_changed(multiplier: float)
signal zoom_step_requested(delta_zoom: float)
signal save_requested
signal follow_toggled(enabled: bool)
signal tool_mode_changed(mode_code: String)

var _colony_label: Label
var _stats_label: Label
var _leader_label: Label
var _inventory_label: Label
var _speed_buttons: Dictionary = {}
var _tool_buttons: Dictionary = {}
var _active_tool_mode: String = "observe"

var _mode_hint_panel: PanelContainer
var _mode_hint_label: Label

var _inspector_panel: PanelContainer
var _inspector_name: Label
var _inspector_meta: Label
var _follow_btn: Button
var _following: bool = false

var _menu_backdrop: ColorRect
var _menu_popup: PanelContainer
var _toast_label: Label
var _toast_timer: float = 0.0


func _ready() -> void:
	_build_hud()


func _process(delta: float) -> void:
	if _toast_timer > 0.0:
		_toast_timer -= delta
		if _toast_timer <= 0.0:
			_toast_label.visible = false


func _build_hud() -> void:
	var root := Control.new()
	root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(root)

	# Barra superior compacta estilo pergamino/madera
	var top_margin := MarginContainer.new()
	top_margin.set_anchors_and_offsets_preset(Control.PRESET_TOP_WIDE)
	top_margin.add_theme_constant_override("margin_left", 14)
	top_margin.add_theme_constant_override("margin_right", 14)
	top_margin.add_theme_constant_override("margin_top", 12)
	top_margin.mouse_filter = Control.MOUSE_FILTER_IGNORE
	root.add_child(top_margin)

	var top_row := HBoxContainer.new()
	top_row.add_theme_constant_override("separation", 10)
	top_row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	top_margin.add_child(top_row)

	# Tarjeta izquierda: colonia, líder e inventario
	var info_panel := PanelContainer.new()
	UiTheme.apply_panel_style(info_panel, Color(0.16, 0.13, 0.10, 0.93), UiTheme.BORDER_BRASS, 12, 16, 9)
	top_row.add_child(info_panel)

	var info_v := VBoxContainer.new()
	info_v.add_theme_constant_override("separation", 3)
	info_panel.add_child(info_v)

	var title_line := HBoxContainer.new()
	title_line.add_theme_constant_override("separation", 10)
	info_v.add_child(title_line)

	_colony_label = Label.new()
	_colony_label.text = GameSession.colony_name
	_colony_label.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
	_colony_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(20))
	title_line.add_child(_colony_label)

	_leader_label = Label.new()
	_leader_label.text = "★ Líder: ..."
	_leader_label.add_theme_color_override("font_color", UiTheme.TEXT_ACCENT)
	_leader_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(16))
	title_line.add_child(_leader_label)

	_stats_label = Label.new()
	_stats_label.text = "%s • Día %d • %d colonos" % [
		SaveManager.biome_label_es(GameSession.biome_code),
		GameSession.day_count,
		GameSession.initial_colonists,
	]
	_stats_label.add_theme_color_override("font_color", UiTheme.TEXT_MUTED)
	_stats_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(15))
	info_v.add_child(_stats_label)

	_inventory_label = Label.new()
	_inventory_label.text = "Madera: 0  •  Piedra: 0  •  Hierro: 0  •  Cobre: 0  •  Semillas: 0  •  Zanahorias: 0"
	_inventory_label.add_theme_color_override("font_color", Color("#e5d7be"))
	_inventory_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(15))
	info_v.add_child(_inventory_label)

	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	spacer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	top_row.add_child(spacer)

	# Controles de velocidad y zoom (cómodos para dedos)
	var controls_panel := PanelContainer.new()
	controls_panel.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	UiTheme.apply_panel_style(controls_panel, Color(0.16, 0.13, 0.10, 0.93), UiTheme.BORDER_BRASS, 12, 10, 8)
	top_row.add_child(controls_panel)

	var ctrl_row := HBoxContainer.new()
	ctrl_row.add_theme_constant_override("separation", 6)
	controls_panel.add_child(ctrl_row)

	for spd in [{"label": "||", "val": 0.0}, {"label": "1x", "val": 1.0}, {"label": "2x", "val": 2.0}]:
		var b := Button.new()
		b.text = spd["label"]
		b.custom_minimum_size = Vector2(50, 48)
		var v: float = spd["val"]
		b.pressed.connect(func():
			UiTheme.trigger_touch_feedback()
			_set_active_speed(v)
			speed_changed.emit(v)
		)
		ctrl_row.add_child(b)
		_speed_buttons[v] = b
	_set_active_speed(1.0)

	var zoom_out := Button.new()
	zoom_out.text = "-"
	zoom_out.custom_minimum_size = Vector2(48, 48)
	UiTheme.style_button(zoom_out, "secondary", 48, 21)
	zoom_out.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		zoom_step_requested.emit(-0.18)
	)
	ctrl_row.add_child(zoom_out)

	var zoom_in := Button.new()
	zoom_in.text = "+"
	zoom_in.custom_minimum_size = Vector2(48, 48)
	UiTheme.style_button(zoom_in, "secondary", 48, 21)
	zoom_in.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		zoom_step_requested.emit(0.18)
	)
	ctrl_row.add_child(zoom_in)

	var menu_btn := Button.new()
	menu_btn.text = "Menú"
	UiTheme.style_button(menu_btn, "amber", 48, 18)
	menu_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_toggle_menu(true)
	)
	ctrl_row.add_child(menu_btn)

	# Barra inferior de órdenes y zonas para el Líder de Colonia
	var bottom_margin := MarginContainer.new()
	bottom_margin.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_WIDE)
	bottom_margin.add_theme_constant_override("margin_left", 14)
	bottom_margin.add_theme_constant_override("margin_right", 14)
	bottom_margin.add_theme_constant_override("margin_bottom", 12)
	bottom_margin.mouse_filter = Control.MOUSE_FILTER_IGNORE
	root.add_child(bottom_margin)

	var bottom_v := VBoxContainer.new()
	bottom_v.alignment = BoxContainer.ALIGNMENT_END
	bottom_v.add_theme_constant_override("separation", 8)
	bottom_v.mouse_filter = Control.MOUSE_FILTER_IGNORE
	bottom_margin.add_child(bottom_v)

	_mode_hint_panel = PanelContainer.new()
	_mode_hint_panel.visible = false
	_mode_hint_panel.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	UiTheme.apply_panel_style(_mode_hint_panel, Color(0.14, 0.11, 0.08, 0.92), UiTheme.BORDER_LIGHT, 10, 16, 7)
	bottom_v.add_child(_mode_hint_panel)

	_mode_hint_label = Label.new()
	_mode_hint_label.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
	_mode_hint_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(16))
	_mode_hint_panel.add_child(_mode_hint_label)

	var tool_panel := PanelContainer.new()
	tool_panel.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	UiTheme.apply_panel_style(tool_panel, Color(0.16, 0.13, 0.10, 0.94), UiTheme.BORDER_BRASS, 14, 12, 8)
	bottom_v.add_child(tool_panel)

	var tool_row := HBoxContainer.new()
	tool_row.add_theme_constant_override("separation", 8)
	tool_panel.add_child(tool_row)

	var tools_def := [
		{"code": "observe", "label": "Observar"},
		{"code": "chop", "label": "Talar"},
		{"code": "mine", "label": "Minar"},
		{"code": "storage", "label": "Almacén"},
		{"code": "farm", "label": "Cultivo"},
		{"code": "build_wall", "label": "Pared Madera"},
		{"code": "cancel", "label": "Cancelar"},
	]
	for td in tools_def:
		var code: String = td["code"]
		var btn := Button.new()
		btn.text = td["label"]
		btn.pressed.connect(func():
			UiTheme.trigger_touch_feedback()
			_set_active_tool_mode(code)
		)
		tool_row.add_child(btn)
		_tool_buttons[code] = btn
	_set_active_tool_mode("observe")

	# Inspector inferior izquierdo del colono tocado (sobre la barra de herramientas)
	_inspector_panel = PanelContainer.new()
	_inspector_panel.visible = false
	_inspector_panel.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_LEFT)
	_inspector_panel.offset_left = 18
	_inspector_panel.offset_bottom = -96
	_inspector_panel.offset_right = 520
	_inspector_panel.offset_top = -254
	UiTheme.apply_panel_style(_inspector_panel, Color(0.16, 0.13, 0.10, 0.95), UiTheme.BORDER_LIGHT, 14, 18, 14)
	root.add_child(_inspector_panel)

	var insp_row := HBoxContainer.new()
	insp_row.add_theme_constant_override("separation", 14)
	_inspector_panel.add_child(insp_row)

	var insp_v := VBoxContainer.new()
	insp_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	insp_v.add_theme_constant_override("separation", 5)
	insp_row.add_child(insp_v)

	_inspector_name = Label.new()
	_inspector_name.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
	_inspector_name.add_theme_font_size_override("font_size", UiTheme.scaled_px(20))
	insp_v.add_child(_inspector_name)

	_inspector_meta = Label.new()
	_inspector_meta.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_inspector_meta.add_theme_color_override("font_color", UiTheme.TEXT_MUTED)
	_inspector_meta.add_theme_font_size_override("font_size", UiTheme.scaled_px(16))
	insp_v.add_child(_inspector_meta)

	_follow_btn = Button.new()
	_follow_btn.text = "Seguir"
	_follow_btn.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	UiTheme.style_button(_follow_btn, "secondary", 50, 18)
	_follow_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_following = not _following
		UiTheme.style_button(_follow_btn, "amber" if _following else "secondary", 50, 18)
		follow_toggled.emit(_following)
	)
	insp_row.add_child(_follow_btn)

	# Notificación flotante de acciones y guardado
	_toast_label = Label.new()
	_toast_label.visible = false
	_toast_label.set_anchors_and_offsets_preset(Control.PRESET_CENTER_TOP)
	_toast_label.offset_top = 112
	_toast_label.add_theme_color_override("font_color", Color("#f6eccf"))
	_toast_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(19))
	root.add_child(_toast_label)

	# Popup modal del menú en partida
	_menu_backdrop = ColorRect.new()
	_menu_backdrop.visible = false
	_menu_backdrop.color = Color(0.05, 0.04, 0.03, 0.72)
	_menu_backdrop.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	root.add_child(_menu_backdrop)

	_menu_popup = PanelContainer.new()
	_menu_popup.set_anchors_and_offsets_preset(Control.PRESET_CENTER)
	_menu_popup.custom_minimum_size = Vector2(450, 0)
	UiTheme.apply_panel_style(_menu_popup, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 28, 24)
	_menu_backdrop.add_child(_menu_popup)

	var mv := VBoxContainer.new()
	mv.add_theme_constant_override("separation", 14)
	_menu_popup.add_child(mv)

	var mtitle := Label.new()
	mtitle.text = "Menú de Colonia"
	mtitle.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	UiTheme.style_title(mtitle, 28)
	mv.add_child(mtitle)

	var resume_btn := Button.new()
	resume_btn.text = "Continuar jugando"
	UiTheme.style_button(resume_btn, "primary", 56, 20)
	resume_btn.pressed.connect(func(): _toggle_menu(false))
	mv.add_child(resume_btn)

	var save_btn := Button.new()
	save_btn.text = "Guardar en Ranura %d" % GameSession.active_slot
	UiTheme.style_button(save_btn, "amber", 56, 20)
	save_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		save_requested.emit()
		_toggle_menu(false)
	)
	mv.add_child(save_btn)

	var slots_btn := Button.new()
	slots_btn.text = "Ranuras de Partidas"
	UiTheme.style_button(slots_btn, "secondary", 56, 20)
	slots_btn.pressed.connect(func():
		save_requested.emit()
		GameSession.return_scene_path = "res://scenes/world/colony_world.tscn"
		get_tree().change_scene_to_file("res://scenes/ui/save_slots_screen.tscn")
	)
	mv.add_child(slots_btn)

	var settings_btn := Button.new()
	settings_btn.text = "Ajustes del Juego"
	UiTheme.style_button(settings_btn, "secondary", 56, 20)
	settings_btn.pressed.connect(func():
		save_requested.emit()
		GameSession.return_scene_path = "res://scenes/world/colony_world.tscn"
		get_tree().change_scene_to_file("res://scenes/ui/settings_screen.tscn")
	)
	mv.add_child(settings_btn)

	var main_btn := Button.new()
	main_btn.text = "Volver al Menú Principal"
	UiTheme.style_button(main_btn, "danger", 56, 20)
	main_btn.pressed.connect(func():
		save_requested.emit()
		get_tree().change_scene_to_file("res://scenes/ui/main_menu.tscn")
	)
	mv.add_child(main_btn)


func _set_active_speed(val: float) -> void:
	for k in _speed_buttons.keys():
		UiTheme.style_button(_speed_buttons[k], "amber" if is_equal_approx(k, val) else "secondary", 48, 18)


func _set_active_tool_mode(mode_code: String) -> void:
	_active_tool_mode = mode_code
	for code in _tool_buttons.keys():
		var variant := "secondary"
		if code == mode_code:
			variant = "danger" if code == "cancel" else "amber"
		UiTheme.style_button(_tool_buttons[code], variant, 48, 17)

	match mode_code:
		"chop":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Talar: arrastra sobre árboles • El líder reunirá al grupo para dar la orden"
		"mine":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Minar: arrastra sobre rocas o vetas de hierro • El líder comunicará la orden"
		"storage":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Zona de Almacén: arrastra donde guardar madera, piedra, menas y cosechas"
		"farm":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Zona de Cultivo (Zanahorias): arrastra tierra o pasto para arar y sembrar"
		"build_wall":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Pared de Madera (2 madera/bloque): arrastra para colocar planos de casa"
		"cancel":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Cancelar: arrastra sobre áreas, planos o marcas para limpiarlas"
		_:
			_mode_hint_panel.visible = false

	tool_mode_changed.emit(mode_code)


func _toggle_menu(open: bool) -> void:
	_menu_backdrop.visible = open


func update_world_summary(snap: Dictionary, colonist_count: int) -> void:
	var day := int(snap.get("day", GameSession.day_count))
	var leader_name := str(snap.get("leader_name", "..."))
	var meeting_active := bool(snap.get("meeting_active", false))
	var partner := str(snap.get("meeting_partner_name", ""))
	var pending := int(snap.get("pending_briefing_count", 0))

	GameSession.day_count = day
	_stats_label.text = "%s • Día %d • %d colonos" % [
		SaveManager.biome_label_es(GameSession.biome_code),
		day,
		colonist_count,
	]

	if meeting_active:
		if partner.is_empty():
			_leader_label.text = "★ Líder %s: planificando órdenes..." % leader_name
		else:
			_leader_label.text = "★ Líder %s reuniéndose con %s..." % [leader_name, partner]
		_leader_label.add_theme_color_override("font_color", Color("#ffd678"))
	elif pending > 0:
		_leader_label.text = "★ Líder %s (%d órdenes por comunicar)" % [leader_name, pending]
		_leader_label.add_theme_color_override("font_color", Color("#f0ad5e"))
	else:
		_leader_label.text = "★ Líder: %s" % leader_name
		_leader_label.add_theme_color_override("font_color", UiTheme.TEXT_ACCENT)

	var inv_var = snap.get("inventory", {})
	if inv_var is Dictionary:
		var inv: Dictionary = inv_var
		_inventory_label.text = "Madera: %d  •  Piedra: %d  •  Hierro: %d  •  Cobre: %d  •  Semillas: %d  •  Zanahorias: %d" % [
			int(inv.get("wood", 0)),
			int(inv.get("stone", 0)),
			int(inv.get("iron", 0)),
			int(inv.get("copper", 0)),
			int(inv.get("carrot_seeds", 0)),
			int(inv.get("carrots", 0)),
		]


func show_colonist_info(colonist: Node) -> void:
	if colonist == null:
		_inspector_panel.visible = false
		_following = false
		UiTheme.style_button(_follow_btn, "secondary", 50, 18)
		return
	_inspector_panel.visible = true
	var leader_tag := " [Líder]" if bool(colonist.get("is_leader")) else ""
	_inspector_name.text = "%s%s — %s" % [
		str(colonist.get("colonist_name")),
		leader_tag,
		str(colonist.get("colonist_role")),
	]
	var tool_txt := str(colonist.get("equipped_tool_label"))
	var carry_kind := str(colonist.get("carrying_kind"))
	var carry_amt := int(colonist.get("carrying_amount"))
	var carry_line := ""
	if not carry_kind.is_empty() and carry_amt > 0:
		carry_line = "  •  Cargando: x%d" % carry_amt
	_inspector_meta.text = "Actividad: %s\nHerramienta: %s%s\nSalud: %d%%  •  Ánimo: %d%%" % [
		str(colonist.get("current_activity")),
		tool_txt,
		carry_line,
		int(colonist.get("health_value")),
		int(colonist.get("mood_value")),
	]


func show_toast(msg: String) -> void:
	_toast_label.text = msg
	_toast_label.visible = true
	_toast_timer = 2.8
