# Pantalla 4: Ajustes del Juego (Audio, Pantalla Móvil y Cámara Centinela).
#
# Lógica: Edita y persiste las preferencias del jugador mediante `SettingsManager`
# en `user://settings.cfg`, organizadas en bloques claros con controles cómodos
# para dedos en teléfonos móviles.

extends Control

var _master_slider: HSlider
var _master_val: Label
var _ambient_slider: HSlider
var _ambient_val: Label
var _ui_slider: HSlider
var _ui_val: Label

var _scale_normal_btn: Button
var _scale_large_btn: Button
var _haptic_check: CheckButton
var _grid_check: CheckButton

var _pan_slider: HSlider
var _pan_val: Label
var _zoom_slider: HSlider
var _zoom_val: Label

var _status_label: Label


func _ready() -> void:
	_build_ui()
	_sync_from_manager()


func _build_ui() -> void:
	var bg := ColorRect.new()
	bg.color = UiTheme.BG_DARK
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(bg)

	var margin := MarginContainer.new()
	margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	margin.add_theme_constant_override("margin_left", 40)
	margin.add_theme_constant_override("margin_right", 40)
	margin.add_theme_constant_override("margin_top", 26)
	margin.add_theme_constant_override("margin_bottom", 26)
	add_child(margin)

	var root_v := VBoxContainer.new()
	root_v.add_theme_constant_override("separation", 16)
	margin.add_child(root_v)

	# Cabecera
	var header := HBoxContainer.new()
	header.add_theme_constant_override("separation", 16)
	root_v.add_child(header)

	var title := Label.new()
	title.text = "Ajustes del Juego"
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.style_title(title, 36)
	header.add_child(title)

	var back_btn := Button.new()
	back_btn.text = "Volver"
	UiTheme.style_button(back_btn, "secondary", 54, 20)
	back_btn.pressed.connect(_on_back_pressed)
	header.add_child(back_btn)

	# Contenedor con scroll por si la pantalla del teléfono es estrecha
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root_v.add_child(scroll)

	var cols := HBoxContainer.new()
	cols.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cols.add_theme_constant_override("separation", 22)
	scroll.add_child(cols)

	# Panel 1: Audio
	var audio_panel := PanelContainer.new()
	audio_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.apply_panel_style(audio_panel, UiTheme.BG_PANEL, UiTheme.BORDER_BRASS, 14, 24, 22)
	cols.add_child(audio_panel)

	var audio_v := VBoxContainer.new()
	audio_v.add_theme_constant_override("separation", 18)
	audio_panel.add_child(audio_v)
	audio_v.add_child(_section_head("Sonido y Ambiente"))

	var r1 := _make_slider_row("Volumen maestro", 0, 100, 1)
	_master_slider = r1["slider"]
	_master_val = r1["label"]
	audio_v.add_child(r1["row"])

	var r2 := _make_slider_row("Ambiente natural", 0, 100, 1)
	_ambient_slider = r2["slider"]
	_ambient_val = r2["label"]
	audio_v.add_child(r2["row"])

	var r3 := _make_slider_row("Interfaz y toques", 0, 100, 1)
	_ui_slider = r3["slider"]
	_ui_val = r3["label"]
	audio_v.add_child(r3["row"])

	# Panel 2: Pantalla móvil y Cámara Centinela
	var mobile_panel := PanelContainer.new()
	mobile_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.apply_panel_style(mobile_panel, UiTheme.BG_PANEL, UiTheme.BORDER_BRASS, 14, 24, 22)
	cols.add_child(mobile_panel)

	var mob_v := VBoxContainer.new()
	mob_v.add_theme_constant_override("separation", 16)
	mobile_panel.add_child(mob_v)
	mob_v.add_child(_section_head("Pantalla Móvil y Cámara"))

	var scale_lbl := Label.new()
	scale_lbl.text = "Tamaño de botones táctiles"
	UiTheme.style_subtitle(scale_lbl, 19)
	mob_v.add_child(scale_lbl)

	var scale_row := HBoxContainer.new()
	scale_row.add_theme_constant_override("separation", 12)
	mob_v.add_child(scale_row)

	_scale_normal_btn = Button.new()
	_scale_normal_btn.text = "Normal"
	_scale_normal_btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scale_normal_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		SettingsManager.ui_scale_mode = 0
		_apply_live_ui_scale()
	)
	scale_row.add_child(_scale_normal_btn)

	_scale_large_btn = Button.new()
	_scale_large_btn.text = "Grande (Dedos)"
	_scale_large_btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_scale_large_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		SettingsManager.ui_scale_mode = 1
		_apply_live_ui_scale()
	)
	scale_row.add_child(_scale_large_btn)

	_haptic_check = CheckButton.new()
	_haptic_check.text = "Vibración háptica al tocar"
	UiTheme.style_check_button(_haptic_check, 19)
	mob_v.add_child(_haptic_check)

	_grid_check = CheckButton.new()
	_grid_check.text = "Mostrar cuadrícula sutil del suelo"
	UiTheme.style_check_button(_grid_check, 19)
	mob_v.add_child(_grid_check)

	var rp := _make_slider_row("Sensibilidad de cámara", 0.5, 2.0, 0.1)
	_pan_slider = rp["slider"]
	_pan_val = rp["label"]
	mob_v.add_child(rp["row"])

	var rz := _make_slider_row("Velocidad de zoom", 0.5, 2.0, 0.1)
	_zoom_slider = rz["slider"]
	_zoom_val = rz["label"]
	mob_v.add_child(rz["row"])

	# Pie de acciones
	var footer := HBoxContainer.new()
	footer.add_theme_constant_override("separation", 14)
	root_v.add_child(footer)

	_status_label = Label.new()
	_status_label.text = ""
	_status_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_status_label.add_theme_color_override("font_color", Color("#98c47e"))
	_status_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(18))
	footer.add_child(_status_label)

	var reset_btn := Button.new()
	reset_btn.text = "Restablecer"
	UiTheme.style_button(reset_btn, "danger", 56, 20)
	reset_btn.pressed.connect(_on_reset_pressed)
	footer.add_child(reset_btn)

	var save_btn := Button.new()
	save_btn.text = "Guardar Ajustes"
	UiTheme.style_button(save_btn, "primary", 56, 20)
	save_btn.pressed.connect(_on_save_pressed)
	footer.add_child(save_btn)


func _section_head(txt: String) -> Label:
	var lbl := Label.new()
	lbl.text = txt
	UiTheme.style_section_label(lbl, 22)
	return lbl


func _make_slider_row(title_txt: String, min_v: float, max_v: float, step_v: float) -> Dictionary:
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 6)

	var top := HBoxContainer.new()
	box.add_child(top)

	var lbl := Label.new()
	lbl.text = title_txt
	lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.style_subtitle(lbl, 19)
	top.add_child(lbl)

	var val_lbl := Label.new()
	val_lbl.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
	val_lbl.add_theme_font_size_override("font_size", UiTheme.scaled_px(19))
	top.add_child(val_lbl)

	var slider := HSlider.new()
	slider.min_value = min_v
	slider.max_value = max_v
	slider.step = step_v
	UiTheme.style_slider(slider)
	slider.value_changed.connect(func(v: float):
		if max_v <= 5.0:
			val_lbl.text = "%.1fx" % v
		else:
			val_lbl.text = "%d%%" % int(round(v))
	)
	box.add_child(slider)

	return {"row": box, "slider": slider, "label": val_lbl}


func _sync_from_manager() -> void:
	_master_slider.value = SettingsManager.master_volume
	_master_val.text = "%d%%" % int(round(SettingsManager.master_volume))
	_ambient_slider.value = SettingsManager.ambient_volume
	_ambient_val.text = "%d%%" % int(round(SettingsManager.ambient_volume))
	_ui_slider.value = SettingsManager.ui_volume
	_ui_val.text = "%d%%" % int(round(SettingsManager.ui_volume))

	_haptic_check.button_pressed = SettingsManager.haptic_feedback
	_grid_check.button_pressed = SettingsManager.show_terrain_grid

	_pan_slider.value = SettingsManager.pan_sensitivity
	_pan_val.text = "%.1fx" % SettingsManager.pan_sensitivity
	_zoom_slider.value = SettingsManager.zoom_speed
	_zoom_val.text = "%.1fx" % SettingsManager.zoom_speed
	_refresh_scale_buttons()


func _refresh_scale_buttons() -> void:
	UiTheme.style_button(_scale_normal_btn, "amber" if SettingsManager.ui_scale_mode == 0 else "secondary", 52, 20)
	UiTheme.style_button(_scale_large_btn, "amber" if SettingsManager.ui_scale_mode == 1 else "secondary", 52, 20)


func _apply_live_ui_scale() -> void:
	SettingsManager.master_volume = _master_slider.value
	SettingsManager.ambient_volume = _ambient_slider.value
	SettingsManager.ui_volume = _ui_slider.value
	SettingsManager.haptic_feedback = _haptic_check.button_pressed
	SettingsManager.show_terrain_grid = _grid_check.button_pressed
	SettingsManager.pan_sensitivity = _pan_slider.value
	SettingsManager.zoom_speed = _zoom_slider.value
	SettingsManager.save_settings()
	UiTheme.lock_fixed_mobile_scale()
	get_tree().reload_current_scene()


func _on_save_pressed() -> void:
	UiTheme.trigger_touch_feedback()
	SettingsManager.master_volume = _master_slider.value
	SettingsManager.ambient_volume = _ambient_slider.value
	SettingsManager.ui_volume = _ui_slider.value
	SettingsManager.haptic_feedback = _haptic_check.button_pressed
	SettingsManager.show_terrain_grid = _grid_check.button_pressed
	SettingsManager.pan_sensitivity = _pan_slider.value
	SettingsManager.zoom_speed = _zoom_slider.value
	SettingsManager.save_settings()
	UiTheme.lock_fixed_mobile_scale()
	_status_label.text = "Ajustes guardados correctamente."


func _on_reset_pressed() -> void:
	UiTheme.trigger_touch_feedback()
	SettingsManager.reset_defaults()
	UiTheme.lock_fixed_mobile_scale()
	get_tree().reload_current_scene()


func _on_back_pressed() -> void:
	UiTheme.trigger_touch_feedback()
	var target := GameSession.return_scene_path
	if target.is_empty():
		target = "res://scenes/ui/main_menu.tscn"
	get_tree().change_scene_to_file(target)
