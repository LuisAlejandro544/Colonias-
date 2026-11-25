# Interfaz superpuesta (HUD) del mundo 2D en vista centinela para móvil.
#
# Lógica: Muestra barra superior de colonia/velocidad/zoom, inspector táctil
# inferior para el colono seleccionado y un popup limpio de menú en partida
# con guardado en la ranura actual, acceso a ranuras, ajustes y salida.

class_name WorldHud
extends CanvasLayer

signal speed_changed(multiplier: float)
signal zoom_step_requested(delta_zoom: float)
signal save_requested
signal follow_toggled(enabled: bool)

var _colony_label: Label
var _stats_label: Label
var _speed_buttons: Dictionary = {}

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
	top_margin.add_theme_constant_override("margin_left", 16)
	top_margin.add_theme_constant_override("margin_right", 16)
	top_margin.add_theme_constant_override("margin_top", 14)
	top_margin.mouse_filter = Control.MOUSE_FILTER_IGNORE
	root.add_child(top_margin)

	var top_row := HBoxContainer.new()
	top_row.add_theme_constant_override("separation", 12)
	top_row.mouse_filter = Control.MOUSE_FILTER_IGNORE
	top_margin.add_child(top_row)

	# Tarjeta izquierda: nombre de colonia y bioma
	var info_panel := PanelContainer.new()
	UiTheme.apply_panel_style(info_panel, Color(0.16, 0.13, 0.10, 0.92), UiTheme.BORDER_BRASS, 12, 18, 10)
	top_row.add_child(info_panel)

	var info_v := VBoxContainer.new()
	info_v.add_theme_constant_override("separation", 3)
	info_panel.add_child(info_v)

	_colony_label = Label.new()
	_colony_label.text = GameSession.colony_name
	_colony_label.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
	_colony_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(21))
	info_v.add_child(_colony_label)

	_stats_label = Label.new()
	_stats_label.text = "%s • Día %d • %d colonos" % [
		SaveManager.biome_label_es(GameSession.biome_code),
		GameSession.day_count,
		GameSession.initial_colonists,
	]
	_stats_label.add_theme_color_override("font_color", UiTheme.TEXT_MUTED)
	_stats_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(16))
	info_v.add_child(_stats_label)

	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	spacer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	top_row.add_child(spacer)

	# Controles de velocidad y zoom (cómodos para dedos)
	var controls_panel := PanelContainer.new()
	UiTheme.apply_panel_style(controls_panel, Color(0.16, 0.13, 0.10, 0.92), UiTheme.BORDER_BRASS, 12, 12, 8)
	top_row.add_child(controls_panel)

	var ctrl_row := HBoxContainer.new()
	ctrl_row.add_theme_constant_override("separation", 8)
	controls_panel.add_child(ctrl_row)

	for spd in [{"label": "||", "val": 0.0}, {"label": "1x", "val": 1.0}, {"label": "2x", "val": 2.0}]:
		var b := Button.new()
		b.text = spd["label"]
		b.custom_minimum_size = Vector2(56, 50)
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
	zoom_out.text = " - "
	zoom_out.custom_minimum_size = Vector2(54, 50)
	UiTheme.style_button(zoom_out, "secondary", 50, 22)
	zoom_out.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		zoom_step_requested.emit(-0.18)
	)
	ctrl_row.add_child(zoom_out)

	var zoom_in := Button.new()
	zoom_in.text = " + "
	zoom_in.custom_minimum_size = Vector2(54, 50)
	UiTheme.style_button(zoom_in, "secondary", 50, 22)
	zoom_in.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		zoom_step_requested.emit(0.18)
	)
	ctrl_row.add_child(zoom_in)

	var menu_btn := Button.new()
	menu_btn.text = "Menú"
	UiTheme.style_button(menu_btn, "amber", 50, 19)
	menu_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_toggle_menu(true)
	)
	ctrl_row.add_child(menu_btn)

	# Inspector inferior izquierdo del colono tocado
	_inspector_panel = PanelContainer.new()
	_inspector_panel.visible = false
	_inspector_panel.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_LEFT)
	_inspector_panel.offset_left = 18
	_inspector_panel.offset_bottom = -18
	_inspector_panel.offset_right = 470
	_inspector_panel.offset_top = -156
	UiTheme.apply_panel_style(_inspector_panel, Color(0.16, 0.13, 0.10, 0.94), UiTheme.BORDER_LIGHT, 14, 18, 14)
	root.add_child(_inspector_panel)

	var insp_row := HBoxContainer.new()
	insp_row.add_theme_constant_override("separation", 14)
	_inspector_panel.add_child(insp_row)

	var insp_v := VBoxContainer.new()
	insp_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	insp_v.add_theme_constant_override("separation", 6)
	insp_row.add_child(insp_v)

	_inspector_name = Label.new()
	_inspector_name.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
	_inspector_name.add_theme_font_size_override("font_size", UiTheme.scaled_px(21))
	insp_v.add_child(_inspector_name)

	_inspector_meta = Label.new()
	_inspector_meta.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_inspector_meta.add_theme_color_override("font_color", UiTheme.TEXT_MUTED)
	_inspector_meta.add_theme_font_size_override("font_size", UiTheme.scaled_px(17))
	insp_v.add_child(_inspector_meta)

	_follow_btn = Button.new()
	_follow_btn.text = "Seguir"
	_follow_btn.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	UiTheme.style_button(_follow_btn, "secondary", 52, 19)
	_follow_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_following = not _following
		UiTheme.style_button(_follow_btn, "amber" if _following else "secondary", 52, 19)
		follow_toggled.emit(_following)
	)
	insp_row.add_child(_follow_btn)

	# Notificación flotante de guardado
	_toast_label = Label.new()
	_toast_label.visible = false
	_toast_label.set_anchors_and_offsets_preset(Control.PRESET_CENTER_TOP)
	_toast_label.offset_top = 98
	_toast_label.add_theme_color_override("font_color", Color("#eef6df"))
	_toast_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(20))
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
		UiTheme.style_button(_speed_buttons[k], "amber" if is_equal_approx(k, val) else "secondary", 50, 19)


func _toggle_menu(open: bool) -> void:
	_menu_backdrop.visible = open


func show_colonist_info(colonist: ColonistVisual) -> void:
	if colonist == null:
		_inspector_panel.visible = false
		_following = false
		UiTheme.style_button(_follow_btn, "secondary", 52, 19)
		return
	_inspector_panel.visible = true
	_inspector_name.text = "%s — %s" % [colonist.colonist_name, colonist.colonist_role]
	_inspector_meta.text = "Actividad: %s\nSalud: %d%%  •  Ánimo: %d%%" % [
		colonist.current_activity,
		colonist.health_value,
		colonist.mood_value,
	]


func show_toast(msg: String) -> void:
	_toast_label.text = msg
	_toast_label.visible = true
	_toast_timer = 2.6
