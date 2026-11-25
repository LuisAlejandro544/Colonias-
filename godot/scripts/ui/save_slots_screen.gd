# Pantalla 3: Ranuras de Partidas Guardadas.
#
# Lógica: Muestra las 3 ranuras persistentes de `SaveManager` con resumen de la
# colonia (nombre, bioma, semilla, colonos, día y fecha). Permite cargar una
# partida, crear una nueva en una ranura vacía o borrar con popup de confirmación.

extends Control

var _slots_box: VBoxContainer
var _confirm_overlay: PanelContainer
var _confirm_label: Label
var _pending_delete_slot: int = -1


func _ready() -> void:
	_build_ui()
	_refresh_slots()


func _build_ui() -> void:
	var bg := ColorRect.new()
	bg.color = UiTheme.BG_DARK
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(bg)

	var margin := MarginContainer.new()
	margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	margin.add_theme_constant_override("margin_left", 40)
	margin.add_theme_constant_override("margin_right", 40)
	margin.add_theme_constant_override("margin_top", 28)
	margin.add_theme_constant_override("margin_bottom", 28)
	add_child(margin)

	var root_v := VBoxContainer.new()
	root_v.add_theme_constant_override("separation", 18)
	margin.add_child(root_v)

	var header := HBoxContainer.new()
	header.add_theme_constant_override("separation", 16)
	root_v.add_child(header)

	var title := Label.new()
	title.text = "Ranuras de Partidas Guardadas"
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.style_title(title, 36)
	header.add_child(title)

	var back_btn := Button.new()
	back_btn.text = "Volver"
	UiTheme.style_button(back_btn, "secondary", 54, 20)
	back_btn.pressed.connect(_on_back_pressed)
	header.add_child(back_btn)

	var hint := Label.new()
	hint.text = "Selecciona una colonia guardada para continuar o administra tus espacios disponibles."
	hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(hint, 19)
	root_v.add_child(hint)

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root_v.add_child(scroll)

	_slots_box = VBoxContainer.new()
	_slots_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_slots_box.add_theme_constant_override("separation", 16)
	scroll.add_child(_slots_box)
	UiTheme.enable_touch_scroll(scroll)

	_build_confirm_popup()


func _build_confirm_popup() -> void:
	_confirm_overlay = PanelContainer.new()
	_confirm_overlay.visible = false
	_confirm_overlay.set_anchors_and_offsets_preset(Control.PRESET_CENTER)
	_confirm_overlay.custom_minimum_size = Vector2(540, 230)
	UiTheme.apply_panel_style(_confirm_overlay, UiTheme.BG_PANEL_ALT, Color("#b86454"), 16, 28, 24)
	add_child(_confirm_overlay)

	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 18)
	_confirm_overlay.add_child(v)

	var head := Label.new()
	head.text = "Confirmar eliminación"
	UiTheme.style_title(head, 28)
	v.add_child(head)

	_confirm_label = Label.new()
	_confirm_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(_confirm_label, 19)
	v.add_child(_confirm_label)

	var row := HBoxContainer.new()
	row.alignment = BoxContainer.ALIGNMENT_END
	row.add_theme_constant_override("separation", 14)
	v.add_child(row)

	var cancel_btn := Button.new()
	cancel_btn.text = "Cancelar"
	UiTheme.style_button(cancel_btn, "secondary", 54, 20)
	cancel_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_pending_delete_slot = -1
		_confirm_overlay.visible = false
	)
	row.add_child(cancel_btn)

	var ok_btn := Button.new()
	ok_btn.text = "Eliminar Partida"
	UiTheme.style_button(ok_btn, "danger", 54, 20)
	ok_btn.pressed.connect(_confirm_delete)
	row.add_child(ok_btn)


func _refresh_slots() -> void:
	for child in _slots_box.get_children():
		child.queue_free()

	for info in SaveManager.list_slots():
		var slot_idx := int(info["slot"])
		var card := PanelContainer.new()
		UiTheme.apply_panel_style(card, UiTheme.BG_SLOT, UiTheme.BORDER_BRASS, 14, 24, 20)
		_slots_box.add_child(card)

		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 18)
		card.add_child(row)

		var text_v := VBoxContainer.new()
		text_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		text_v.add_theme_constant_override("separation", 6)
		row.add_child(text_v)

		var title := Label.new()
		var sub := Label.new()
		sub.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART

		if info["exists"]:
			title.text = "Ranura %d — %s" % [slot_idx, info["colony_name"]]
			UiTheme.style_title(title, 25)

			sub.text = "Colonos: %d  •  Día %d\nGuardado: %s" % [
				info["colonist_count"],
				info["day"],
				info["saved_at"],
			]
			UiTheme.style_subtitle(sub, 17)
		else:
			title.text = "Ranura %d — Espacio vacío" % slot_idx
			title.add_theme_color_override("font_color", UiTheme.TEXT_MUTED)
			title.add_theme_font_size_override("font_size", UiTheme.scaled_px(24))

			sub.text = "Lista para fundar una nueva colonia terrestre."
			UiTheme.style_subtitle(sub, 17)

		text_v.add_child(title)
		text_v.add_child(sub)

		var actions := HBoxContainer.new()
		actions.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		actions.add_theme_constant_override("separation", 12)
		row.add_child(actions)

		if info["exists"]:
			var load_btn := Button.new()
			load_btn.text = "Cargar"
			UiTheme.style_button(load_btn, "primary", 56, 21)
			load_btn.pressed.connect(_on_load_slot.bind(slot_idx))
			actions.add_child(load_btn)

			var del_btn := Button.new()
			del_btn.text = "Borrar"
			UiTheme.style_button(del_btn, "danger", 56, 21)
			del_btn.pressed.connect(_ask_delete_slot.bind(slot_idx, str(info["colony_name"])))
			actions.add_child(del_btn)
		else:
			var create_btn := Button.new()
			create_btn.text = "Nueva Partida aquí"
			UiTheme.style_button(create_btn, "amber", 56, 20)
			create_btn.pressed.connect(func():
				UiTheme.trigger_touch_feedback()
				get_tree().change_scene_to_file("res://scenes/ui/new_game_screen.tscn")
			)
			actions.add_child(create_btn)

	UiTheme.make_scroll_pass(_slots_box)


func _on_load_slot(slot_idx: int) -> void:
	UiTheme.trigger_touch_feedback()
	var data := SaveManager.load_game(slot_idx)
	if data.is_empty():
		return
	GameSession.load_from_slot_data(slot_idx, data)
	get_tree().change_scene_to_file("res://scenes/ui/loading_screen.tscn")


func _ask_delete_slot(slot_idx: int, colony_name: String) -> void:
	UiTheme.trigger_touch_feedback()
	_pending_delete_slot = slot_idx
	_confirm_label.text = "¿Seguro que deseas eliminar permanentemente la colonia \"%s\" en la Ranura %d?" % [colony_name, slot_idx]
	_confirm_overlay.visible = true


func _confirm_delete() -> void:
	UiTheme.trigger_touch_feedback()
	if _pending_delete_slot > 0:
		SaveManager.delete_slot(_pending_delete_slot)
	_pending_delete_slot = -1
	_confirm_overlay.visible = false
	_refresh_slots()


func _on_back_pressed() -> void:
	UiTheme.trigger_touch_feedback()
	var target := GameSession.return_scene_path
	if target.is_empty():
		target = "res://scenes/ui/main_menu.tscn"
	get_tree().change_scene_to_file(target)
