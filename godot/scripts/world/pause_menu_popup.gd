# Popup modal modular del Menú en Partida de la colonia.
#
# Lógica: Ofrece acciones rápidas durante la partida sin salir al menú sin guardar:
# continuar jugando, guardar en la ranura activa, abrir Ranuras de Partidas,
# abrir Ajustes del Juego o volver al Menú Principal.

class_name PauseMenuPopup
extends ColorRect

signal save_requested

var _save_btn: Button


func _ready() -> void:
	visible = false
	color = Color(0.05, 0.04, 0.03, 0.72)
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_build_ui()


func _build_ui() -> void:
	var menu_popup := PanelContainer.new()
	menu_popup.set_anchors_and_offsets_preset(Control.PRESET_CENTER)
	menu_popup.custom_minimum_size = Vector2(450, 0)
	UiTheme.apply_panel_style(menu_popup, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 28, 24)
	add_child(menu_popup)

	var mv := VBoxContainer.new()
	mv.add_theme_constant_override("separation", 14)
	menu_popup.add_child(mv)

	var mtitle := Label.new()
	mtitle.text = "Menú de Colonia"
	mtitle.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	UiTheme.style_title(mtitle, 28)
	mv.add_child(mtitle)

	var resume_btn := Button.new()
	resume_btn.text = "Continuar jugando"
	UiTheme.style_button(resume_btn, "primary", 56, 20)
	resume_btn.pressed.connect(func(): close_popup())
	mv.add_child(resume_btn)

	_save_btn = Button.new()
	_save_btn.text = "Guardar en Ranura %d" % GameSession.active_slot
	UiTheme.style_button(_save_btn, "amber", 56, 20)
	_save_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		save_requested.emit()
		close_popup()
	)
	mv.add_child(_save_btn)

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


func open_popup() -> void:
	if _save_btn:
		_save_btn.text = "Guardar en Ranura %d" % GameSession.active_slot
	visible = true


func close_popup() -> void:
	visible = false
