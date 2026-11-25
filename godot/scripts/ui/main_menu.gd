# Pantalla 1: Menú Principal de Colonia Centinela.
#
# Lógica: Construye una portada cálida con estética artesanal de madera y pergamino
# acompañada de una escena viva de campamento terrestre en el fondo (árboles, rocas
# y colonos estilo RimWorld), ofreciendo acceso separado a Continuar, Nueva Partida,
# Ranuras de Partidas Guardadas y Ajustes del Juego.

extends Control

var _continue_btn: Button
var _latest_slot: int = -1


func _ready() -> void:
	GameSession.in_world_mode = false
	_build_diorama_background()
	_build_menu_ui()


func _build_diorama_background() -> void:
	var bg := ColorRect.new()
	bg.color = Color("#3e4c2c")
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(bg)

	# Mosaico sutil del terreno terrestre al fondo de la portada.
	var grass_tex := load("res://assets/ground/ground_grass.svg") as Texture2D
	var fertile_tex := load("res://assets/ground/ground_fertile.svg") as Texture2D
	var dirt_tex := load("res://assets/ground/ground_dirt.svg") as Texture2D
	var tiles_holder := Node2D.new()
	tiles_holder.modulate = Color(0.88, 0.85, 0.78, 1.0)
	add_child(tiles_holder)

	for y in range(12):
		for x in range(22):
			var sp := Sprite2D.new()
			if (x + y) % 7 == 0 and fertile_tex:
				sp.texture = fertile_tex
			elif (x * 3 + y) % 9 == 0 and dirt_tex:
				sp.texture = dirt_tex
			else:
				sp.texture = grass_tex
			sp.position = Vector2(x * 72 + 36, y * 72 + 36)
			sp.scale = Vector2(0.57, 0.57)
			tiles_holder.add_child(sp)

	# Elementos naturales decorativos del campamento en el lado derecho.
	var props_layer := Node2D.new()
	props_layer.y_sort_enabled = true
	add_child(props_layer)

	var decor_items := [
		{"path": "res://assets/props/tree_oak.svg", "pos": Vector2(820, 180), "scale": 0.92},
		{"path": "res://assets/props/tree_pine.svg", "pos": Vector2(1080, 210), "scale": 0.95},
		{"path": "res://assets/props/boulder_granite.svg", "pos": Vector2(980, 490), "scale": 0.78},
		{"path": "res://assets/props/boulder_iron.svg", "pos": Vector2(720, 560), "scale": 0.70},
		{"path": "res://assets/props/sapling.svg", "pos": Vector2(750, 340), "scale": 0.65},
		{"path": "res://assets/props/sapling.svg", "pos": Vector2(1140, 420), "scale": 0.62},
		{"path": "res://assets/props/stone_pebbles.svg", "pos": Vector2(870, 580), "scale": 0.65},
		{"path": "res://assets/props/tree_oak.svg", "pos": Vector2(1160, 610), "scale": 0.85},
	]
	for item in decor_items:
		var tex := load(item["path"]) as Texture2D
		if tex:
			var sprite := Sprite2D.new()
			sprite.texture = tex
			sprite.position = item["pos"]
			var s: float = item["scale"]
			sprite.scale = Vector2(s, s)
			props_layer.add_child(sprite)

	# Colonos de muestra en el claro derecho para dar vida a la portada
	var demo_pawns := [
		{"id": 1, "name": "Mateo", "role": "Leñador", "x": 14.2, "y": 5.4, "body_style": 0, "hair_style": 0, "skin_color": "#e4b892", "hair_color": "#5b3b28", "outfit_color": "#4f6b47"},
		{"id": 2, "name": "Lucía", "role": "Exploradora", "x": 16.0, "y": 6.1, "body_style": 1, "hair_style": 2, "skin_color": "#d19b6c", "hair_color": "#2c221e", "outfit_color": "#8f4e37"},
	]
	for dp in demo_pawns:
		var cv := ColonistVisual.new()
		cv.setup_from_snapshot(dp)
		props_layer.add_child(cv)

	# Viñeta sutil para mantener la escena luminosa y cálida sin perder contraste
	var shade := ColorRect.new()
	shade.color = Color(0.08, 0.06, 0.04, 0.25)
	shade.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(shade)


func _build_menu_ui() -> void:
	var margin := MarginContainer.new()
	margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	margin.add_theme_constant_override("margin_left", 42)
	margin.add_theme_constant_override("margin_right", 42)
	margin.add_theme_constant_override("margin_top", 34)
	margin.add_theme_constant_override("margin_bottom", 34)
	add_child(margin)

	var layout := HBoxContainer.new()
	layout.add_theme_constant_override("separation", 36)
	margin.add_child(layout)

	# Tarjeta principal del menú (lado izquierdo)
	var menu_panel := PanelContainer.new()
	menu_panel.custom_minimum_size = Vector2(500, 0)
	menu_panel.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	UiTheme.apply_panel_style(menu_panel, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 30, 26)
	layout.add_child(menu_panel)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 14)
	menu_panel.add_child(vbox)

	var emblem_label := Label.new()
	emblem_label.text = "CRÓNICAS TERRESTRES"
	emblem_label.add_theme_color_override("font_color", UiTheme.TEXT_ACCENT)
	emblem_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(16))
	vbox.add_child(emblem_label)

	var title := Label.new()
	title.text = "Colonia Centinela"
	UiTheme.style_title(title, 42)
	vbox.add_child(title)

	var subtitle := Label.new()
	subtitle.text = "Gestiona asentamientos humanos en la Tierra desde el primer campamento hasta tecnología avanzada."
	subtitle.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(subtitle, 19)
	vbox.add_child(subtitle)

	var sep := HSeparator.new()
	sep.add_theme_constant_override("separation", 10)
	vbox.add_child(sep)

	_latest_slot = SaveManager.get_latest_occupied_slot()
	if _latest_slot > 0:
		var slot_data := SaveManager.load_game(_latest_slot)
		var col_name := str(slot_data.get("colony_name", "Colonia"))
		_continue_btn = Button.new()
		_continue_btn.text = "Continuar: %s (Ranura %d)" % [col_name, _latest_slot]
		UiTheme.style_button(_continue_btn, "amber", 60, 21)
		_continue_btn.pressed.connect(_on_continue_pressed)
		vbox.add_child(_continue_btn)

	var new_game_btn := Button.new()
	new_game_btn.text = "Nueva Partida"
	UiTheme.style_button(new_game_btn, "primary", 60, 22)
	new_game_btn.pressed.connect(_on_new_game_pressed)
	vbox.add_child(new_game_btn)

	var saves_btn := Button.new()
	saves_btn.text = "Partidas Guardadas"
	UiTheme.style_button(saves_btn, "secondary", 58, 21)
	saves_btn.pressed.connect(_on_saves_pressed)
	vbox.add_child(saves_btn)

	var settings_btn := Button.new()
	settings_btn.text = "Ajustes del Juego"
	UiTheme.style_button(settings_btn, "secondary", 58, 21)
	settings_btn.pressed.connect(_on_settings_pressed)
	vbox.add_child(settings_btn)

	var quit_btn := Button.new()
	quit_btn.text = "Salir"
	UiTheme.style_button(quit_btn, "danger", 54, 20)
	quit_btn.pressed.connect(_on_quit_pressed)
	vbox.add_child(quit_btn)

	# Tarjeta informativa cálida a la derecha (resumen de versión y estado del terreno)
	var right_spacer := Control.new()
	right_spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	layout.add_child(right_spacer)

	var card := PanelContainer.new()
	card.custom_minimum_size = Vector2(400, 0)
	card.size_flags_vertical = Control.SIZE_SHRINK_END
	UiTheme.apply_panel_style(card, Color(0.15, 0.12, 0.09, 0.92), UiTheme.BORDER_BRASS, 14, 24, 20)
	layout.add_child(card)

	var card_v := VBoxContainer.new()
	card_v.add_theme_constant_override("separation", 8)
	card.add_child(card_v)

	var card_head := Label.new()
	card_head.text = "Edición Móvil • Vista Centinela"
	card_head.add_theme_color_override("font_color", UiTheme.TEXT_ACCENT)
	card_head.add_theme_font_size_override("font_size", UiTheme.scaled_px(18))
	card_v.add_child(card_head)

	var card_body := Label.new()
	card_body.text = "Mundos terrestres procedurales, biomas vivos y colonos autónomos simulados en núcleo nativo Rust + Godot 4.7."
	card_body.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(card_body, 17)
	card_v.add_child(card_body)


func _on_continue_pressed() -> void:
	UiTheme.trigger_touch_feedback()
	if _latest_slot <= 0:
		return
	var data := SaveManager.load_game(_latest_slot)
	if data.is_empty():
		return
	GameSession.load_from_slot_data(_latest_slot, data)
	get_tree().change_scene_to_file("res://scenes/ui/loading_screen.tscn")


func _on_new_game_pressed() -> void:
	UiTheme.trigger_touch_feedback()
	GameSession.return_scene_path = "res://scenes/ui/main_menu.tscn"
	get_tree().change_scene_to_file("res://scenes/ui/new_game_screen.tscn")


func _on_saves_pressed() -> void:
	UiTheme.trigger_touch_feedback()
	GameSession.return_scene_path = "res://scenes/ui/main_menu.tscn"
	get_tree().change_scene_to_file("res://scenes/ui/save_slots_screen.tscn")


func _on_settings_pressed() -> void:
	UiTheme.trigger_touch_feedback()
	GameSession.return_scene_path = "res://scenes/ui/main_menu.tscn"
	get_tree().change_scene_to_file("res://scenes/ui/settings_screen.tscn")


func _on_quit_pressed() -> void:
	UiTheme.trigger_touch_feedback()
	get_tree().quit()
