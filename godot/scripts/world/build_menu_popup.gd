# Popup modular del Menú de Construcción y Zonas de la colonia.
#
# Lógica: Presenta las tarjetas desplazables de construcción y designación de
# áreas (Almacén, Cultivos con riego, Muros de madera o piedra/ladrillo, Puerta,
# Suelo de ladrillos, Pozo, Brasero, Mesa de Carpintero, Mesa de Masón y Horno)
# con su icono, descripción y costo, emitiendo el modo seleccionado hacia `WorldHud`.

class_name BuildMenuPopup
extends ColorRect

signal build_mode_selected(mode_code: String)

var _option_buttons: Dictionary = {}


func _ready() -> void:
	visible = false
	color = Color(0.05, 0.04, 0.03, 0.68)
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_build_ui()


func _build_ui() -> void:
	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(center)

	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(760, 520)
	UiTheme.apply_panel_style(panel, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 26, 22)
	center.add_child(panel)

	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 12)
	panel.add_child(v)

	var head_row := HBoxContainer.new()
	head_row.add_theme_constant_override("separation", 12)
	v.add_child(head_row)

	var title_v := VBoxContainer.new()
	title_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title_v.add_theme_constant_override("separation", 4)
	head_row.add_child(title_v)

	var title := Label.new()
	title.text = "Menú de Construcción y Zonas"
	UiTheme.style_title(title, 28)
	title_v.add_child(title)

	var sub := Label.new()
	sub.text = "Elige zonas de acopio o cultivo, muros, suelos, pozos, braseros, mesas de trabajo y hornos."
	sub.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(sub, 16)
	title_v.add_child(sub)

	var close_btn := Button.new()
	close_btn.text = "Cerrar"
	UiTheme.style_button(close_btn, "secondary", 48, 18)
	close_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		close_popup()
	)
	head_row.add_child(close_btn)

	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.custom_minimum_size = Vector2(0, 380)
	v.add_child(scroll)

	var cards_v := VBoxContainer.new()
	cards_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cards_v.add_theme_constant_override("separation", 10)
	scroll.add_child(cards_v)
	UiTheme.enable_touch_scroll(scroll)

	var build_items := [
		{
			"code": "storage",
			"title": "Zona de Almacenamiento",
			"desc": "Punto de acopio para troncos, tablones, piedra, menas, objetos y cosechas.",
			"cost": "Sin costo de materiales",
			"icon": "res://assets/items/item_wood_logs.svg",
		},
		{
			"code": "farm",
			"title": "Zona de Cultivo (Zanahorias)",
			"desc": "Área agrícola para arar con azada de cobre, sembrar zanahorias y cosechar.",
			"cost": "Usa semillas de zanahoria al sembrar (mejor en Temporada)",
			"icon": "res://assets/items/item_carrot.svg",
		},
		{
			"code": "farm_potato",
			"title": "Zona de Cultivo (Papas)",
			"desc": "Área agrícola para arar tierra, sembrar semillas de papa y cosechar tubérculos.",
			"cost": "Usa semillas de papa al sembrar (mejor en Temporada)",
			"icon": "res://assets/items/item_potato.svg",
		},
		{
			"code": "farm_wheat",
			"title": "Zona de Cultivo (Trigo)",
			"desc": "Área agrícola para cultivar espigas de trigo dorado en suelo preparado.",
			"cost": "Usa semillas de trigo al sembrar (mejor en Temporada)",
			"icon": "res://assets/items/item_wheat.svg",
		},
		{
			"code": "farm_beet",
			"title": "Zona de Cultivo (Remolacha)",
			"desc": "Área agrícola para sembrar y cosechar remolachas frescas en el huerto.",
			"cost": "Usa semillas de remolacha al sembrar (mejor en Temporada)",
			"icon": "res://assets/items/item_beet.svg",
		},
		{
			"code": "build_wall",
			"title": "Pared de Madera",
			"desc": "Bloques sólidos para refugios y muros que protegen y cierran espacios.",
			"cost": "Costo: 2 troncos de madera por bloque",
			"icon": "res://assets/structures/wall_wood.svg",
		},
		{
			"code": "build_door",
			"title": "Puerta de Madera",
			"desc": "Acceso instalable entre muros de madera que permite el paso de los colonos.",
			"cost": "Costo: 1 puerta de madera (Mesa de Carpintero • INT 3+)",
			"icon": "res://assets/structures/door_wood.svg",
		},
		{
			"code": "build_carpenter_bench",
			"title": "Mesa de Carpintero",
			"desc": "Banco de trabajo para aserrar tablones y fabricar vasos, cubos, puertas y arcones según Inteligencia.",
			"cost": "Costo: 6 troncos de madera",
			"icon": "res://assets/structures/carpenter_bench.svg",
		},
		{
			"code": "build_well",
			"title": "Pozo de Agua",
			"desc": "Extrae agua subterránea profunda; los colonos beben de él y llenan cubos para regar cultivos.",
			"cost": "Costo: 1 cubo de madera, 4 troncos y 6 piedras",
			"icon": "res://assets/structures/water_well.svg",
		},
		{
			"code": "build_mason_bench",
			"title": "Mesa de Masón",
			"desc": "Taller de cantería para labrar piedra en ladrillos, morteros, ruedas de molino y estatuillas.",
			"cost": "Costo: 6 bloques de piedra",
			"icon": "res://assets/structures/mason_bench.svg",
		},
		{
			"code": "build_furnace",
			"title": "Horno de Piedra y Ladrillo",
			"desc": "Horno cerrado de piedra y ladrillos de piedra que funde menas de cobre con carbón en lingotes.",
			"cost": "Costo: 4 piedras y 6 ladrillos de piedra",
			"icon": "res://assets/structures/stone_furnace.svg",
		},
		{
			"code": "build_stone_wall",
			"title": "Pared de Piedra y Ladrillo",
			"desc": "Muro sólido conectable de mampostería que refuerza y protege las estancias de la colonia.",
			"cost": "Costo: 1 piedra y 2 ladrillos de piedra por bloque",
			"icon": "res://assets/structures/wall_stone.svg",
		},
		{
			"code": "build_stone_floor",
			"title": "Suelo de Ladrillos",
			"desc": "Calzada firme de ladrillos de piedra que acelera el paso de los colonos un +18% al caminar encima.",
			"cost": "Costo: 2 ladrillos de piedra por casilla",
			"icon": "res://assets/structures/floor_stone.svg",
		},
		{
			"code": "build_stone_brazier",
			"title": "Brasero de Piedra y Ladrillo",
			"desc": "Luminaria cálida de piedra, ladrillos y carbón que aporta abrigo y luz en los caminos y talleres.",
			"cost": "Costo: 2 piedras, 3 ladrillos de piedra y 1 carbón",
			"icon": "res://assets/structures/stone_brazier.svg",
		},
	]

	for item in build_items:
		var card := PanelContainer.new()
		UiTheme.apply_panel_style(card, UiTheme.BG_PANEL_ALT, UiTheme.BORDER_BRASS, 12, 16, 12)
		cards_v.add_child(card)

		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 14)
		card.add_child(row)

		var icon_box := PanelContainer.new()
		icon_box.custom_minimum_size = Vector2(64, 64)
		icon_box.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
		icon_box.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		UiTheme.apply_panel_style(icon_box, Color("#1b1510"), UiTheme.BORDER_BRASS, 10, 8, 8)
		row.add_child(icon_box)

		var icon := TextureRect.new()
		icon.texture = load(item["icon"]) as Texture2D
		icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
		icon.custom_minimum_size = Vector2(46, 46)
		icon_box.add_child(icon)

		var txt_v := VBoxContainer.new()
		txt_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		txt_v.add_theme_constant_override("separation", 3)
		row.add_child(txt_v)

		var lbl_t := Label.new()
		lbl_t.text = item["title"]
		UiTheme.style_title(lbl_t, 20)
		txt_v.add_child(lbl_t)

		var lbl_d := Label.new()
		lbl_d.text = item["desc"]
		lbl_d.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		UiTheme.style_subtitle(lbl_d, 15)
		txt_v.add_child(lbl_d)

		var lbl_c := Label.new()
		lbl_c.text = item["cost"]
		lbl_c.add_theme_color_override("font_color", UiTheme.TEXT_ACCENT)
		lbl_c.add_theme_font_size_override("font_size", UiTheme.scaled_px(14))
		txt_v.add_child(lbl_c)

		var code: String = item["code"]
		var pick_btn := Button.new()
		pick_btn.text = "Seleccionar"
		pick_btn.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		pick_btn.pressed.connect(func():
			UiTheme.trigger_touch_feedback()
			build_mode_selected.emit(code)
			close_popup()
		)
		row.add_child(pick_btn)
		_option_buttons[code] = pick_btn

	UiTheme.make_scroll_pass(cards_v)


func open_popup() -> void:
	visible = true


func close_popup() -> void:
	visible = false


func update_selected_mode(mode_code: String) -> void:
	for b_code in _option_buttons.keys():
		var b_code_str: String = str(b_code)
		var b_btn: Button = _option_buttons[b_code]
		var selected: bool = (b_code_str == mode_code)
		b_btn.text = "Activo" if selected else "Seleccionar"
		UiTheme.style_button(b_btn, "amber" if selected else "primary", 46, 17)
