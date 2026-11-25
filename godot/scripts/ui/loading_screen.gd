# Pantalla de transición y carga del mundo 2D terrestre.
#
# Lógica: Muestra inmediatamente una interfaz cálida con el resumen de la colonia,
# barra de progreso y estado mientras carga en segundo plano la escena
# `colony_world.tscn` mediante `ResourceLoader.load_threaded_request`, evitando
# pantallas grises vacías o congelamientos al entrar al mundo.

extends Control

const WORLD_SCENE_PATH := "res://scenes/world/colony_world.tscn"

var _progress_bar: ProgressBar
var _status_label: Label
var _back_btn: Button
var _frames_waited: int = 0
var _load_started: bool = false
var _finished: bool = false


func _ready() -> void:
	_build_ui()


func _build_ui() -> void:
	var bg := ColorRect.new()
	bg.color = UiTheme.BG_DARK
	bg.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(bg)

	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(center)

	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(680, 0)
	UiTheme.apply_panel_style(panel, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 36, 32)
	center.add_child(panel)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 18)
	panel.add_child(vbox)

	var eyebrow := Label.new()
	eyebrow.text = "EXPEDICIÓN TERRESTRE EN MARCHA"
	eyebrow.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	eyebrow.add_theme_color_override("font_color", UiTheme.TEXT_ACCENT)
	eyebrow.add_theme_font_size_override("font_size", UiTheme.scaled_px(16))
	vbox.add_child(eyebrow)

	var title := Label.new()
	title.text = "Preparando: %s" % GameSession.colony_name
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_title(title, 34)
	vbox.add_child(title)

	var meta := Label.new()
	meta.text = "Bioma: %s  •  Mapa %dx%d  •  Semilla %d" % [
		SaveManager.biome_label_es(GameSession.biome_code),
		GameSession.map_size,
		GameSession.map_size,
		GameSession.seed_value,
	]
	meta.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	meta.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(meta, 19)
	vbox.add_child(meta)

	_progress_bar = ProgressBar.new()
	_progress_bar.min_value = 0.0
	_progress_bar.max_value = 100.0
	_progress_bar.value = 15.0
	_progress_bar.show_percentage = false
	_progress_bar.custom_minimum_size = Vector2(0, 30)

	var bar_bg := StyleBoxFlat.new()
	bar_bg.bg_color = Color("#1b1510")
	bar_bg.border_color = UiTheme.BORDER_BRASS
	bar_bg.set_border_width_all(2)
	bar_bg.set_corner_radius_all(8)
	_progress_bar.add_theme_stylebox_override("background", bar_bg)

	var bar_fill := StyleBoxFlat.new()
	bar_fill.bg_color = UiTheme.BTN_PRIMARY_HOVER
	bar_fill.set_corner_radius_all(6)
	_progress_bar.add_theme_stylebox_override("fill", bar_fill)
	vbox.add_child(_progress_bar)

	_status_label = Label.new()
	_status_label.text = "Cargando escena del mundo y texturas vectoriales..."
	_status_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(_status_label, 18)
	vbox.add_child(_status_label)

	_back_btn = Button.new()
	_back_btn.text = "Volver al Menú Principal"
	_back_btn.visible = false
	UiTheme.style_button(_back_btn, "secondary", 56, 20)
	_back_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		get_tree().change_scene_to_file("res://scenes/ui/main_menu.tscn")
	)
	vbox.add_child(_back_btn)


func _process(_delta: float) -> void:
	if _finished:
		return

	if not _load_started:
		_frames_waited += 1
		if _frames_waited >= 2:
			_load_started = true
			var err := ResourceLoader.load_threaded_request(WORLD_SCENE_PATH)
			if err != OK:
				_show_load_error("No se pudo iniciar la carga del mundo (código %d)." % err)
		return

	var progress_arr: Array = []
	var status := ResourceLoader.load_threaded_get_status(WORLD_SCENE_PATH, progress_arr)
	match status:
		ResourceLoader.THREAD_LOAD_IN_PROGRESS:
			var pct := float(progress_arr[0]) * 70.0 + 15.0 if progress_arr.size() > 0 else 45.0
			_progress_bar.value = clampf(pct, 15.0, 88.0)
			_status_label.text = "Cargando recursos del mapa terrestre..."
		ResourceLoader.THREAD_LOAD_LOADED:
			_finished = true
			_progress_bar.value = 95.0
			_status_label.text = "Iniciando generación procedural del terreno..."
			var res := ResourceLoader.load_threaded_get(WORLD_SCENE_PATH)
			if res is PackedScene:
				get_tree().change_scene_to_packed(res as PackedScene)
			else:
				_show_load_error("La escena del mundo no es válida.")
		ResourceLoader.THREAD_LOAD_FAILED, ResourceLoader.THREAD_LOAD_INVALID_RESOURCE:
			_show_load_error("Error al cargar la escena del mundo (`colony_world.tscn`).")


func _show_load_error(msg: String) -> void:
	_finished = true
	_status_label.text = msg
	_status_label.add_theme_color_override("font_color", Color("#e58a76"))
	_back_btn.visible = true
