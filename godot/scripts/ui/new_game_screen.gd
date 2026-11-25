# Pantalla 2: Configuración de Nueva Partida (Colonia Terrestre).
#
# Lógica: Permite al jugador elegir el nombre de su colonia, una semilla
# procedural aleatoria o manual, el bioma de la Tierra (Bosque Templado,
# Pradera Fértil o Sierra Rocosa), el tamaño del mapa, el número de colonos
# iniciales y la ranura de guardado destino antes de lanzar la simulación.

extends Control

const COLONY_NAMES := [
	"Valle del Roble",
	"Paso del Cedro",
	"Ribera Clara",
	"Loma de Piedra",
	"Vega Serena",
	"Puerto Arroyo",
]

var _name_input: LineEdit
var _seed_input: LineEdit
var _biome_code: String = "temperate_forest"
var _map_size: int = 64
var _colonists_count: int = 4
var _target_slot: int = 1

var _biome_buttons: Dictionary = {}
var _size_buttons: Dictionary = {}
var _slot_buttons: Dictionary = {}
var _colonist_label: Label


func _ready() -> void:
	_target_slot = _pick_default_slot()
	_build_ui()


func _pick_default_slot() -> int:
	for info in SaveManager.list_slots():
		if not info.get("exists", false):
			return int(info.get("slot", 1))
	return 1


func _build_ui() -> void:
	var bg := ColorRect.new()
	bg.color = UiTheme.BG_DARK
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(bg)

	var margin := MarginContainer.new()
	margin.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	margin.add_theme_constant_override("margin_left", 36)
	margin.add_theme_constant_override("margin_right", 36)
	margin.add_theme_constant_override("margin_top", 26)
	margin.add_theme_constant_override("margin_bottom", 26)
	add_child(margin)

	var root_v := VBoxContainer.new()
	root_v.add_theme_constant_override("separation", 18)
	margin.add_child(root_v)

	# Cabecera
	var header := HBoxContainer.new()
	header.add_theme_constant_override("separation", 16)
	root_v.add_child(header)

	var title := Label.new()
	title.text = "Fundar Nueva Colonia"
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.style_title(title, 36)
	header.add_child(title)

	var back_btn := Button.new()
	back_btn.text = "Volver al Menú"
	UiTheme.style_button(back_btn, "secondary", 54, 20)
	back_btn.pressed.connect(_on_back_pressed)
	header.add_child(back_btn)

	# Contenido central en dos columnas limpias con scroll táctil vertical
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	root_v.add_child(scroll)

	var cols := HBoxContainer.new()
	cols.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cols.add_theme_constant_override("separation", 22)
	scroll.add_child(cols)

	# Columna izquierda: Identidad, Semilla y Bioma
	var left_panel := PanelContainer.new()
	left_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.apply_panel_style(left_panel, UiTheme.BG_PANEL, UiTheme.BORDER_BRASS, 14, 24, 22)
	cols.add_child(left_panel)

	var left_v := VBoxContainer.new()
	left_v.add_theme_constant_override("separation", 16)
	left_panel.add_child(left_v)

	left_v.add_child(_section_label("Nombre de la colonia"))
	var name_row := HBoxContainer.new()
	name_row.add_theme_constant_override("separation", 12)
	left_v.add_child(name_row)

	_name_input = LineEdit.new()
	_name_input.text = COLONY_NAMES[randi() % COLONY_NAMES.size()]
	_name_input.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.style_line_edit(_name_input, 54, 20)
	name_row.add_child(_name_input)

	var rand_name_btn := Button.new()
	rand_name_btn.text = "Aleatorio"
	UiTheme.style_button(rand_name_btn, "secondary", 54, 20)
	rand_name_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_name_input.text = COLONY_NAMES[randi() % COLONY_NAMES.size()]
	)
	name_row.add_child(rand_name_btn)

	left_v.add_child(_section_label("Semilla del mundo procedural"))
	var seed_row := HBoxContainer.new()
	seed_row.add_theme_constant_override("separation", 12)
	left_v.add_child(seed_row)

	_seed_input = LineEdit.new()
	_seed_input.text = str(randi_range(10000, 999999))
	_seed_input.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.style_line_edit(_seed_input, 54, 20)
	seed_row.add_child(_seed_input)

	var rand_seed_btn := Button.new()
	rand_seed_btn.text = "Nueva Semilla"
	UiTheme.style_button(rand_seed_btn, "secondary", 54, 20)
	rand_seed_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_seed_input.text = str(randi_range(10000, 999999))
	)
	seed_row.add_child(rand_seed_btn)

	left_v.add_child(_section_label("Bioma terrestre"))
	var biomes := [
		{"code": "temperate_forest", "title": "Bosque Templado", "desc": "Abundantes robles, pinos y suelo equilibrado."},
		{"code": "fertile_meadow", "title": "Pradera Fértil", "desc": "Amplios pastizales fértiles ideales para crecer."},
		{"code": "rocky_highlands", "title": "Sierra Rocosa", "desc": "Rocas de granito y vetas de hierro expuestas."},
	]
	for b in biomes:
		var btn := Button.new()
		btn.text = "%s — %s" % [b["title"], b["desc"]]
		btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
		btn.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		btn.pressed.connect(_select_biome.bind(b["code"]))
		left_v.add_child(btn)
		_biome_buttons[b["code"]] = btn

	# Columna derecha: Tamaño, Colonos y Ranura
	var right_panel := PanelContainer.new()
	right_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	UiTheme.apply_panel_style(right_panel, UiTheme.BG_PANEL, UiTheme.BORDER_BRASS, 14, 24, 22)
	cols.add_child(right_panel)

	var right_v := VBoxContainer.new()
	right_v.add_theme_constant_override("separation", 16)
	right_panel.add_child(right_v)

	right_v.add_child(_section_label("Tamaño del mapa"))
	var size_row := HBoxContainer.new()
	size_row.add_theme_constant_override("separation", 12)
	right_v.add_child(size_row)

	for sz in [64, 96, 128]:
		var btn := Button.new()
		btn.text = "%dx%d" % [sz, sz]
		btn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		btn.pressed.connect(_select_size.bind(sz))
		size_row.add_child(btn)
		_size_buttons[sz] = btn

	right_v.add_child(_section_label("Colonos iniciales"))
	var col_row := HBoxContainer.new()
	col_row.add_theme_constant_override("separation", 14)
	right_v.add_child(col_row)

	var minus_btn := Button.new()
	minus_btn.text = "  -  "
	UiTheme.style_button(minus_btn, "secondary", 54, 24)
	minus_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_colonists_count = maxi(3, _colonists_count - 1)
		_colonist_label.text = "%d colonos listos para descender" % _colonists_count
	)
	col_row.add_child(minus_btn)

	_colonist_label = Label.new()
	_colonist_label.text = "%d colonos listos para descender" % _colonists_count
	_colonist_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_colonist_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	UiTheme.style_subtitle(_colonist_label, 19)
	col_row.add_child(_colonist_label)

	var plus_btn := Button.new()
	plus_btn.text = "  +  "
	UiTheme.style_button(plus_btn, "secondary", 54, 24)
	plus_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		_colonists_count = mini(6, _colonists_count + 1)
		_colonist_label.text = "%d colonos listos para descender" % _colonists_count
	)
	col_row.add_child(plus_btn)

	right_v.add_child(_section_label("Ranura de guardado destino"))
	for slot_info in SaveManager.list_slots():
		var idx := int(slot_info["slot"])
		var btn := Button.new()
		btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
		btn.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		if slot_info["exists"]:
			btn.text = "Ranura %d (Sobrescribir: %s)" % [idx, slot_info["colony_name"]]
		else:
			btn.text = "Ranura %d (Vacía)" % idx
		btn.pressed.connect(_select_slot.bind(idx))
		right_v.add_child(btn)
		_slot_buttons[idx] = btn

	# Barra inferior con botón principal de inicio
	var footer := HBoxContainer.new()
	footer.alignment = BoxContainer.ALIGNMENT_END
	root_v.add_child(footer)

	var start_btn := Button.new()
	start_btn.text = "Fundar Colonia y Entrar al Mundo"
	UiTheme.style_button(start_btn, "primary", 64, 22)
	start_btn.pressed.connect(_on_start_pressed)
	footer.add_child(start_btn)

	_refresh_selection_styles()
	UiTheme.enable_touch_scroll(scroll)


func _section_label(txt: String) -> Label:
	var lbl := Label.new()
	lbl.text = txt
	UiTheme.style_section_label(lbl, 20)
	return lbl


func _select_biome(code: String) -> void:
	UiTheme.trigger_touch_feedback()
	_biome_code = code
	_refresh_selection_styles()


func _select_size(sz: int) -> void:
	UiTheme.trigger_touch_feedback()
	_map_size = sz
	_refresh_selection_styles()


func _select_slot(slot_idx: int) -> void:
	UiTheme.trigger_touch_feedback()
	_target_slot = slot_idx
	_refresh_selection_styles()


func _refresh_selection_styles() -> void:
	for code in _biome_buttons.keys():
		UiTheme.style_button(_biome_buttons[code], "amber" if code == _biome_code else "secondary", 58, 19)
	for sz in _size_buttons.keys():
		UiTheme.style_button(_size_buttons[sz], "amber" if sz == _map_size else "secondary", 54, 20)
	for idx in _slot_buttons.keys():
		UiTheme.style_button(_slot_buttons[idx], "amber" if idx == _target_slot else "secondary", 54, 19)


func _on_back_pressed() -> void:
	UiTheme.trigger_touch_feedback()
	get_tree().change_scene_to_file("res://scenes/ui/main_menu.tscn")


func _on_start_pressed() -> void:
	UiTheme.trigger_touch_feedback()
	var raw_seed := _seed_input.text.strip_edges()
	var parsed_seed := int(raw_seed.hash()) if not raw_seed.is_valid_int() else int(raw_seed)
	if parsed_seed == 0:
		parsed_seed = 24819
	GameSession.start_new_colony(
		_target_slot,
		_name_input.text,
		parsed_seed,
		_biome_code,
		_map_size,
		_colonists_count
	)
	get_tree().change_scene_to_file("res://scenes/ui/loading_screen.tscn")
