# Interfaz superpuesta (HUD) del mundo 2D en vista centinela para móvil.
#
# Lógica: Muestra la barra superior (colonia, día, estado del Líder de Colonia,
# inventario de recursos, velocidad y zoom), la barra inferior de acciones
# (Observar, Talar, Minar, Construir, Trabajos y Cancelar), el popup dedicado del
# Menú de Construcción (Almacén, Cultivo de Zanahorias y Pared de Madera), el
# popup modular de Trabajos (8 roles, máx. 5 por colono, niveles 1 a 10), el
# inspector del colono tocado con su popup de Inventario/Equipo y el menú modal.

class_name WorldHud
extends CanvasLayer

signal speed_changed(multiplier: float)
signal zoom_step_requested(delta_zoom: float)
signal save_requested
signal follow_toggled(enabled: bool)
signal tool_mode_changed(mode_code: String)
signal jobs_opened
signal role_toggle_requested(colonist_id: int, role_code: String, enabled: bool)

const JobsPopupScript := preload("res://scripts/world/jobs_popup.gd")

var _colony_label: Label
var _stats_label: Label
var _leader_label: Label
var _inventory_label: Label
var _speed_buttons: Dictionary = {}
var _tool_buttons: Dictionary = {}
var _build_menu_btn: Button
var _active_tool_mode: String = "observe"

var _mode_hint_panel: PanelContainer
var _mode_hint_label: Label

# Popup del Menú de Construcción y Zonas
var _build_backdrop: ColorRect
var _build_option_buttons: Dictionary = {}

# Inspector inferior izquierdo del colono
var _inspector_panel: PanelContainer
var _inspector_name: Label
var _inspector_meta: Label
var _follow_btn: Button
var _inv_open_btn: Button
var _jobs_open_btn: Button
var _following: bool = false
var _inspected_colonist: Node = null
var _jobs_popup: ColorRect

# Popup de Inventario y Ranuras de Equipo del colono
var _inv_backdrop: ColorRect
var _inv_title_label: Label
var _inv_helmet_val: Label
var _inv_armor_val: Label
var _inv_back_val: Label
var _inv_back_icon: TextureRect
var _inv_hand_val: Label
var _inv_hand_icon: TextureRect
var _inv_carry_val: Label
var _inv_carry_icon: TextureRect

# Popup modal del menú en partida
var _menu_backdrop: ColorRect
var _menu_popup: PanelContainer
var _toast_label: Label
var _toast_timer: float = 0.0

var _tool_icons: Dictionary = {}
var _item_icons: Dictionary = {}


func _ready() -> void:
	_preload_icons()
	_build_hud()


func _preload_icons() -> void:
	_tool_icons = {
		"copper_axe": load("res://assets/tools/tool_copper_axe.svg") as Texture2D,
		"copper_pickaxe": load("res://assets/tools/tool_copper_pickaxe.svg") as Texture2D,
		"copper_shovel": load("res://assets/tools/tool_copper_shovel.svg") as Texture2D,
		"copper_hoe": load("res://assets/tools/tool_copper_hoe.svg") as Texture2D,
		"copper_sword": load("res://assets/tools/tool_copper_sword.svg") as Texture2D,
	}
	_item_icons = {
		"wood_logs": load("res://assets/items/item_wood_logs.svg") as Texture2D,
		"stone_chunk": load("res://assets/items/item_stone_chunk.svg") as Texture2D,
		"iron_ore": load("res://assets/items/item_iron_ore.svg") as Texture2D,
		"copper_ore": load("res://assets/items/item_copper_ore.svg") as Texture2D,
		"carrot_seed": load("res://assets/items/item_carrot_seeds.svg") as Texture2D,
		"carrot": load("res://assets/items/item_carrot.svg") as Texture2D,
	}


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
	_stats_label.text = "Día %d • %d colonos" % [
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

	# Barra inferior principal (Observar, Talar, Minar, Construir, Cancelar)
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
	tool_row.add_theme_constant_override("separation", 10)
	tool_panel.add_child(tool_row)

	for td in [
		{"code": "observe", "label": "Observar"},
		{"code": "chop", "label": "Talar"},
		{"code": "mine", "label": "Minar"},
	]:
		var code: String = td["code"]
		var btn := Button.new()
		btn.text = td["label"]
		btn.pressed.connect(func():
			UiTheme.trigger_touch_feedback()
			_set_active_tool_mode(code)
		)
		tool_row.add_child(btn)
		_tool_buttons[code] = btn

	_build_menu_btn = Button.new()
	_build_menu_btn.text = "Construir"
	_build_menu_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_toggle_build_menu(true)
	)
	tool_row.add_child(_build_menu_btn)

	var jobs_btn := Button.new()
	jobs_btn.text = "Trabajos"
	UiTheme.style_button(jobs_btn, "secondary", 48, 17)
	jobs_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		jobs_opened.emit()
	)
	tool_row.add_child(jobs_btn)

	var cancel_btn := Button.new()
	cancel_btn.text = "Cancelar"
	cancel_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_set_active_tool_mode("cancel")
	)
	tool_row.add_child(cancel_btn)
	_tool_buttons["cancel"] = cancel_btn

	# Inspector inferior izquierdo del colono tocado
	_inspector_panel = PanelContainer.new()
	_inspector_panel.visible = false
	_inspector_panel.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_LEFT)
	_inspector_panel.offset_left = 18
	_inspector_panel.offset_bottom = -96
	_inspector_panel.offset_right = 590
	_inspector_panel.offset_top = -304
	UiTheme.apply_panel_style(_inspector_panel, Color(0.16, 0.13, 0.10, 0.95), UiTheme.BORDER_LIGHT, 14, 18, 14)
	root.add_child(_inspector_panel)

	var insp_row := HBoxContainer.new()
	insp_row.add_theme_constant_override("separation", 12)
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
	_inspector_meta.add_theme_font_size_override("font_size", UiTheme.scaled_px(15))
	insp_v.add_child(_inspector_meta)

	var insp_btns := VBoxContainer.new()
	insp_btns.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	insp_btns.add_theme_constant_override("separation", 8)
	insp_row.add_child(insp_btns)

	_inv_open_btn = Button.new()
	_inv_open_btn.text = "Inventario"
	UiTheme.style_button(_inv_open_btn, "primary", 46, 17)
	_inv_open_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_toggle_colonist_inventory(true)
	)
	insp_btns.add_child(_inv_open_btn)

	_jobs_open_btn = Button.new()
	_jobs_open_btn.text = "Trabajos"
	UiTheme.style_button(_jobs_open_btn, "amber", 46, 17)
	_jobs_open_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		jobs_opened.emit()
	)
	insp_btns.add_child(_jobs_open_btn)

	_follow_btn = Button.new()
	_follow_btn.text = "Seguir"
	UiTheme.style_button(_follow_btn, "secondary", 46, 17)
	_follow_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_following = not _following
		UiTheme.style_button(_follow_btn, "amber" if _following else "secondary", 46, 17)
		follow_toggled.emit(_following)
	)
	insp_btns.add_child(_follow_btn)

	# Notificación flotante de acciones y guardado
	_toast_label = Label.new()
	_toast_label.visible = false
	_toast_label.set_anchors_and_offsets_preset(Control.PRESET_CENTER_TOP)
	_toast_label.offset_top = 112
	_toast_label.add_theme_color_override("font_color", Color("#f6eccf"))
	_toast_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(19))
	root.add_child(_toast_label)

	_build_construction_menu_popup(root)
	_build_colonist_inventory_popup(root)
	_jobs_popup = JobsPopupScript.new() as ColorRect
	root.add_child(_jobs_popup)
	_jobs_popup.connect("role_toggled", func(cid: int, r_code: String, enabled: bool):
		role_toggle_requested.emit(cid, r_code, enabled)
	)
	_build_pause_menu_popup(root)
	_set_active_tool_mode("observe")


func _build_construction_menu_popup(root: Control) -> void:
	_build_backdrop = ColorRect.new()
	_build_backdrop.visible = false
	_build_backdrop.color = Color(0.05, 0.04, 0.03, 0.68)
	_build_backdrop.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	root.add_child(_build_backdrop)

	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_build_backdrop.add_child(center)

	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(720, 0)
	UiTheme.apply_panel_style(panel, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 26, 22)
	center.add_child(panel)

	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 14)
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
	sub.text = "Elige dónde almacenar recursos, sembrar zanahorias o levantar paredes de madera."
	sub.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(sub, 16)
	title_v.add_child(sub)

	var close_btn := Button.new()
	close_btn.text = "Cerrar"
	UiTheme.style_button(close_btn, "secondary", 48, 18)
	close_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_toggle_build_menu(false)
	)
	head_row.add_child(close_btn)

	var build_items := [
		{
			"code": "storage",
			"title": "Zona de Almacenamiento",
			"desc": "Punto de acopio para madera, piedra, menas y cosechas recolectadas.",
			"cost": "Sin costo de materiales",
			"icon": "res://assets/items/item_wood_logs.svg",
		},
		{
			"code": "farm",
			"title": "Zona de Cultivo (Zanahorias)",
			"desc": "Área agrícola para arar con azada de cobre, sembrar semillas y cosechar.",
			"cost": "Usa semillas de zanahoria al sembrar",
			"icon": "res://assets/items/item_carrot.svg",
		},
		{
			"code": "build_wall",
			"title": "Pared de Madera",
			"desc": "Bloques sólidos para refugios y muros que protegen y cierran espacios.",
			"cost": "Costo: 2 troncos de madera por bloque",
			"icon": "res://assets/structures/wall_wood.svg",
		},
	]

	for item in build_items:
		var card := PanelContainer.new()
		UiTheme.apply_panel_style(card, UiTheme.BG_PANEL_ALT, UiTheme.BORDER_BRASS, 12, 16, 12)
		v.add_child(card)

		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 14)
		card.add_child(row)

		var icon_box := PanelContainer.new()
		icon_box.custom_minimum_size = Vector2(64, 64)
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
			_set_active_tool_mode(code)
			_toggle_build_menu(false)
		)
		row.add_child(pick_btn)
		_build_option_buttons[code] = pick_btn


func _build_colonist_inventory_popup(root: Control) -> void:
	_inv_backdrop = ColorRect.new()
	_inv_backdrop.visible = false
	_inv_backdrop.color = Color(0.05, 0.04, 0.03, 0.72)
	_inv_backdrop.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	root.add_child(_inv_backdrop)

	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_inv_backdrop.add_child(center)

	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(700, 0)
	UiTheme.apply_panel_style(panel, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 26, 22)
	center.add_child(panel)

	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 14)
	panel.add_child(v)

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 12)
	v.add_child(head)

	_inv_title_label = Label.new()
	_inv_title_label.text = "Inventario del Colono"
	_inv_title_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.style_title(_inv_title_label, 26)
	head.add_child(_inv_title_label)

	var close_btn := Button.new()
	close_btn.text = "Cerrar"
	UiTheme.style_button(close_btn, "secondary", 48, 18)
	close_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_toggle_colonist_inventory(false)
	)
	head.add_child(close_btn)

	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 12)
	grid.add_theme_constant_override("v_separation", 12)
	v.add_child(grid)

	var helmet_slot := _make_equipment_slot("Casco (Futuro)", "Ranura vacía")
	_inv_helmet_val = helmet_slot["val"]
	grid.add_child(helmet_slot["card"])

	var armor_slot := _make_equipment_slot("Armadura (Futura)", "Ranura vacía")
	_inv_armor_val = armor_slot["val"]
	grid.add_child(armor_slot["card"])

	var back_slot := _make_equipment_slot("Espalda", "Vacío")
	_inv_back_val = back_slot["val"]
	_inv_back_icon = back_slot["icon"]
	grid.add_child(back_slot["card"])

	var hand_slot := _make_equipment_slot("Mano Principal", "Vacío")
	_inv_hand_val = hand_slot["val"]
	_inv_hand_icon = hand_slot["icon"]
	grid.add_child(hand_slot["card"])

	var carry_slot := _make_equipment_slot("Mochila / Carga Actual", "Sin carga")
	_inv_carry_val = carry_slot["val"]
	_inv_carry_icon = carry_slot["icon"]
	v.add_child(carry_slot["card"])


func _make_equipment_slot(slot_title: String, default_text: String) -> Dictionary:
	var card := PanelContainer.new()
	card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.apply_panel_style(card, UiTheme.BG_PANEL_ALT, UiTheme.BORDER_BRASS, 12, 14, 10)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 12)
	card.add_child(row)

	var icon_box := PanelContainer.new()
	icon_box.custom_minimum_size = Vector2(54, 54)
	UiTheme.apply_panel_style(icon_box, Color("#1b1510"), UiTheme.BORDER_BRASS, 8, 6, 6)
	row.add_child(icon_box)

	var icon := TextureRect.new()
	icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	icon.custom_minimum_size = Vector2(40, 40)
	icon_box.add_child(icon)

	var txt_v := VBoxContainer.new()
	txt_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	txt_v.add_theme_constant_override("separation", 2)
	row.add_child(txt_v)

	var lbl_head := Label.new()
	lbl_head.text = slot_title
	UiTheme.style_section_label(lbl_head, 15)
	txt_v.add_child(lbl_head)

	var lbl_val := Label.new()
	lbl_val.text = default_text
	lbl_val.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	lbl_val.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
	lbl_val.add_theme_font_size_override("font_size", UiTheme.scaled_px(17))
	txt_v.add_child(lbl_val)

	return {"card": card, "val": lbl_val, "icon": icon}


func _build_pause_menu_popup(root: Control) -> void:
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

	var is_build_mode: bool = mode_code in ["storage", "farm", "build_wall"]
	if _build_menu_btn:
		UiTheme.style_button(_build_menu_btn, "amber" if is_build_mode else "primary", 48, 17)

	for b_code in _build_option_buttons.keys():
		var b_code_str: String = str(b_code)
		var b_btn: Button = _build_option_buttons[b_code]
		var selected: bool = (b_code_str == mode_code)
		b_btn.text = "Activo" if selected else "Seleccionar"
		UiTheme.style_button(b_btn, "amber" if selected else "primary", 46, 17)

	match mode_code:
		"chop":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Talar: arrastra sobre árboles • El líder reunirá al grupo para dar la orden"
		"mine":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Minar: arrastra sobre rocas o vetas de hierro • El líder comunicará la orden"
		"storage":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Construir Almacén: arrastra donde guardar madera, piedra, menas y cosechas"
		"farm":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Construir Cultivo (Zanahorias): arrastra tierra o pasto para arar y sembrar"
		"build_wall":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Construir Pared de Madera (2 madera/bloque): arrastra para colocar planos"
		"cancel":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Cancelar: arrastra sobre áreas, planos o marcas para limpiarlas"
		_:
			_mode_hint_panel.visible = false

	tool_mode_changed.emit(mode_code)


func _toggle_build_menu(open: bool) -> void:
	if _build_backdrop:
		_build_backdrop.visible = open


func _toggle_colonist_inventory(open: bool) -> void:
	if _inv_backdrop:
		_inv_backdrop.visible = open
		if open and _inspected_colonist != null:
			_refresh_inventory_popup(_inspected_colonist)


func _toggle_menu(open: bool) -> void:
	_menu_backdrop.visible = open


func update_world_summary(snap: Dictionary, colonist_count: int) -> void:
	var day := int(snap.get("day", GameSession.day_count))
	var leader_name := str(snap.get("leader_name", "..."))
	var meeting_active := bool(snap.get("meeting_active", false))
	var partner := str(snap.get("meeting_partner_name", ""))
	var pending := int(snap.get("pending_briefing_count", 0))

	GameSession.day_count = day
	_stats_label.text = "Día %d • %d colonos" % [day, colonist_count]

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
	_inspected_colonist = colonist
	if colonist == null:
		_inspector_panel.visible = false
		_following = false
		UiTheme.style_button(_follow_btn, "secondary", 46, 17)
		if _inv_backdrop:
			_inv_backdrop.visible = false
		return

	_inspector_panel.visible = true
	var leader_tag := " [Líder]" if bool(colonist.get("is_leader")) else ""
	_inspector_name.text = "%s%s — %s" % [
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

	_inspector_meta.text = "Roles (%d/5): %s\nActividad: %s\nMano: %s  •  Espalda: %s%s\nEnergía: %d%%  •  Salud: %d%%  •  Ánimo: %d%%" % [
		roles_count,
		roles_txt,
		str(colonist.get("current_activity")),
		hand_txt,
		back_txt,
		carry_line,
		int(colonist.get("energy_value")),
		int(colonist.get("health_value")),
		int(colonist.get("mood_value")),
	]

	if _inv_backdrop and _inv_backdrop.visible:
		_refresh_inventory_popup(colonist)


func _refresh_inventory_popup(colonist: Node) -> void:
	if colonist == null:
		return
	var leader_tag := " [Líder]" if bool(colonist.get("is_leader")) else ""
	_inv_title_label.text = "Inventario de %s%s" % [str(colonist.get("colonist_name")), leader_tag]

	var helmet := str(colonist.get("helmet_slot"))
	_inv_helmet_val.text = helmet if not helmet.is_empty() else "Vacío (Ranura lista para futuros cascos)"

	var armor := str(colonist.get("armor_slot"))
	_inv_armor_val.text = armor if not armor.is_empty() else "Vacío (Ranura lista para futuras armaduras)"

	var back_code := str(colonist.get("back_tool_code"))
	var back_label := str(colonist.get("back_tool_label"))
	_inv_back_val.text = back_label if back_code != "none" else "Vacío"
	_inv_back_icon.texture = _tool_icons.get(back_code, null)

	var hand_code := str(colonist.get("main_hand_tool_code"))
	var hand_label := str(colonist.get("main_hand_tool_label"))
	_inv_hand_val.text = hand_label if hand_code != "none" else "Manos libres"
	_inv_hand_icon.texture = _tool_icons.get(hand_code, null)

	var carry_kind := str(colonist.get("carrying_kind"))
	var carry_label := str(colonist.get("carrying_label"))
	var carry_amt := int(colonist.get("carrying_amount"))
	if not carry_kind.is_empty() and carry_amt > 0:
		_inv_carry_val.text = "%s (x%d)" % [carry_label if not carry_label.is_empty() else carry_kind, carry_amt]
		_inv_carry_icon.texture = _item_icons.get(carry_kind, null)
	else:
		_inv_carry_val.text = "Mochila vacía"
		_inv_carry_icon.texture = null


func open_jobs_popup(colonists: Array, focus_colonist_id: int, colony_sim: Node) -> void:
	if _jobs_popup:
		_jobs_popup.call("open_for_colonists", colonists, focus_colonist_id, colony_sim)


func refresh_jobs_popup(colonists: Array, colony_sim: Node, feedback_msg: String = "", is_error: bool = false) -> void:
	if _jobs_popup:
		_jobs_popup.call("refresh_from_sim", colonists, colony_sim)
		if not feedback_msg.is_empty():
			_jobs_popup.call("show_feedback", feedback_msg, is_error)


func show_toast(msg: String) -> void:
	_toast_label.text = msg
	_toast_label.visible = true
	_toast_timer = 2.8
