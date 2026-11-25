# Pantalla de transición y carga del mundo 2D terrestre.
#
# Lógica: Muestra inmediatamente una interfaz cálida con el resumen de la colonia,
# barra de progreso y estado mientras carga `colony_world.tscn` y verifica que la
# escena y su script se instancien correctamente antes de entregar el control,
# evitando quedar atrapado en una pantalla oscura vacía si ocurre algún fallo.

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
	_progress_bar.value = 18.0
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
	if _finished or _load_started:
		return

	_frames_waited += 1
	if _frames_waited < 3:
		_progress_bar.value = 20.0 + _frames_waited * 10.0
		return

	_load_started = true
	_load_world_safely()


func _load_world_safely() -> void:
	_progress_bar.value = 45.0
	_status_label.text = "Abriendo escena del mundo..."
	await get_tree().process_frame

	if not ResourceLoader.exists(WORLD_SCENE_PATH):
		_show_load_error("No se encontró `colony_world.tscn` dentro del paquete.")
		return

	var packed := ResourceLoader.load(WORLD_SCENE_PATH) as PackedScene
	if packed == null or not packed.can_instantiate():
		_show_load_error("No se pudo cargar o instanciar `colony_world.tscn`.")
		return

	_progress_bar.value = 65.0
	_status_label.text = "Verificando controlador del mundo..."
	await get_tree().process_frame

	var world_node := packed.instantiate()
	if world_node == null:
		_show_load_error("Error interno al instanciar el nodo raíz del mundo.")
		return

	if world_node.get_script() == null or not world_node.has_method("start_colony_world"):
		world_node.queue_free()
		_show_load_error(
			"El script del mundo (`colony_world.gd`) no pudo compilarse en este dispositivo.\n" +
			"Revisa los scripts en `godot/scripts/world/`."
		)
		return

	_progress_bar.value = 80.0
	_status_label.text = "Entrando al mapa terrestre..."
	await get_tree().process_frame

	_finished = true
	var root := get_tree().root
	var old_scene := get_tree().current_scene
	root.add_child(world_node)
	get_tree().current_scene = world_node
	if old_scene and is_instance_valid(old_scene):
		old_scene.queue_free()


func _show_load_error(msg: String) -> void:
	_finished = true
	_status_label.text = msg
	_status_label.add_theme_color_override("font_color", Color("#e58a76"))
	_back_btn.visible = true
