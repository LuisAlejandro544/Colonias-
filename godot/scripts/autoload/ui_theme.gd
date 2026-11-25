# Módulo central de identidad visual cálida ("Madera de Expedición y Pergamino").
#
# Lógica: Define la paleta cromática orgánica sin estética cyberpunk ni minimalismo
# soso, fija un tamaño de interfaz independiente de la configuración de fuente del
# sistema operativo móvil y provee constructores reutilizables de paneles, botones,
# deslizadores, interruptores y campos de texto cómodos para dedos en Android.

extends Node

const BG_DARK := Color("#1b1611")
const BG_PANEL := Color("#2a221a")
const BG_PANEL_ALT := Color("#352b21")
const BG_SLOT := Color("#2f261d")
const BORDER_BRASS := Color("#8c6b43")
const BORDER_LIGHT := Color("#b8925a")
const TEXT_PARCHMENT := Color("#f2e8d4")
const TEXT_MUTED := Color("#c6b69b")
const TEXT_ACCENT := Color("#e0b15e")

const BTN_PRIMARY := Color("#4c6b42")
const BTN_PRIMARY_HOVER := Color("#5c8050")
const BTN_SECONDARY := Color("#4a3b2c")
const BTN_SECONDARY_HOVER := Color("#5e4b38")
const BTN_DANGER := Color("#873b2d")
const BTN_DANGER_HOVER := Color("#a34736")
const BTN_AMBER := Color("#9e6f2d")
const BTN_AMBER_HOVER := Color("#b88237")

var _slider_grabber_tex: Texture2D


func _ready() -> void:
	RenderingServer.set_default_clear_color(BG_DARK)
	lock_fixed_mobile_scale()


func lock_fixed_mobile_scale() -> void:
	# Garantiza que el tamaño de letra configurado en Android no modifique la escala del juego.
	var root := get_tree().root
	if root:
		root.content_scale_mode = Window.CONTENT_SCALE_MODE_CANVAS_ITEMS
		root.content_scale_aspect = Window.CONTENT_SCALE_ASPECT_EXPAND
		root.content_scale_factor = 1.0
		root.theme = _build_global_theme()


func scaled_px(base_px: int) -> int:
	return int(round(float(base_px) * SettingsManager.get_ui_scale_multiplier()))


func _build_global_theme() -> Theme:
	var t := Theme.new()
	t.default_font_size = scaled_px(19)
	t.set_font_size("font_size", "Label", scaled_px(19))
	t.set_font_size("font_size", "Button", scaled_px(21))
	t.set_font_size("font_size", "LineEdit", scaled_px(20))
	t.set_font_size("font_size", "CheckButton", scaled_px(19))
	return t


func apply_panel_style(panel: PanelContainer, bg_color: Color = BG_PANEL, border_color: Color = BORDER_BRASS, radius: int = 12, pad_h: int = 22, pad_v: int = 18) -> void:
	var sb := StyleBoxFlat.new()
	sb.bg_color = bg_color
	sb.border_color = border_color
	sb.set_border_width_all(2)
	sb.set_corner_radius_all(radius)
	sb.content_margin_left = pad_h
	sb.content_margin_right = pad_h
	sb.content_margin_top = pad_v
	sb.content_margin_bottom = pad_v
	sb.shadow_color = Color(0.0, 0.0, 0.0, 0.35)
	sb.shadow_size = 6
	sb.shadow_offset = Vector2(0, 3)
	panel.add_theme_stylebox_override("panel", sb)


func style_button(btn: Button, variant: String = "secondary", min_height: int = 56, font_px: int = 21) -> void:
	var base_col := BTN_SECONDARY
	var hover_col := BTN_SECONDARY_HOVER
	var border_col := BORDER_BRASS

	match variant:
		"primary":
			base_col = BTN_PRIMARY
			hover_col = BTN_PRIMARY_HOVER
			border_col = Color("#7da16d")
		"danger":
			base_col = BTN_DANGER
			hover_col = BTN_DANGER_HOVER
			border_col = Color("#b86454")
		"amber":
			base_col = BTN_AMBER
			hover_col = BTN_AMBER_HOVER
			border_col = BORDER_LIGHT

	var scale_mult := SettingsManager.get_ui_scale_multiplier()
	btn.custom_minimum_size.y = int(round(float(min_height) * scale_mult))
	btn.add_theme_color_override("font_color", TEXT_PARCHMENT)
	btn.add_theme_color_override("font_hover_color", Color("#fff9ec"))
	btn.add_theme_color_override("font_pressed_color", Color("#fff4db"))
	btn.add_theme_color_override("font_disabled_color", Color("#857968"))
	btn.add_theme_font_size_override("font_size", int(round(float(font_px) * scale_mult)))

	var normal := StyleBoxFlat.new()
	normal.bg_color = base_col
	normal.border_color = border_col
	normal.set_border_width_all(2)
	normal.set_corner_radius_all(10)
	normal.content_margin_left = 22
	normal.content_margin_right = 22
	normal.content_margin_top = 12
	normal.content_margin_bottom = 12

	var hover := normal.duplicate() as StyleBoxFlat
	hover.bg_color = hover_col
	hover.border_color = BORDER_LIGHT

	var pressed := normal.duplicate() as StyleBoxFlat
	pressed.bg_color = base_col.darkened(0.18)

	var disabled := normal.duplicate() as StyleBoxFlat
	disabled.bg_color = Color("#2e2720")
	disabled.border_color = Color("#4d4133")

	btn.add_theme_stylebox_override("normal", normal)
	btn.add_theme_stylebox_override("hover", hover)
	btn.add_theme_stylebox_override("pressed", pressed)
	btn.add_theme_stylebox_override("disabled", disabled)


func style_title(label: Label, size_px: int = 38) -> void:
	label.add_theme_color_override("font_color", TEXT_PARCHMENT)
	label.add_theme_font_size_override("font_size", scaled_px(size_px))


func style_subtitle(label: Label, size_px: int = 19) -> void:
	label.add_theme_color_override("font_color", TEXT_MUTED)
	label.add_theme_font_size_override("font_size", scaled_px(size_px))


func style_section_label(label: Label, size_px: int = 20) -> void:
	label.add_theme_color_override("font_color", TEXT_ACCENT)
	label.add_theme_font_size_override("font_size", scaled_px(size_px))


func style_line_edit(edit: LineEdit, min_height: int = 54, font_px: int = 20) -> void:
	var scale_mult := SettingsManager.get_ui_scale_multiplier()
	edit.custom_minimum_size.y = int(round(float(min_height) * scale_mult))
	edit.add_theme_color_override("font_color", TEXT_PARCHMENT)
	edit.add_theme_color_override("font_placeholder_color", TEXT_MUTED)
	edit.add_theme_color_override("caret_color", TEXT_ACCENT)
	edit.add_theme_font_size_override("font_size", int(round(float(font_px) * scale_mult)))

	var box := StyleBoxFlat.new()
	box.bg_color = Color("#1e1812")
	box.border_color = BORDER_BRASS
	box.set_border_width_all(2)
	box.set_corner_radius_all(8)
	box.content_margin_left = 14
	box.content_margin_right = 14
	box.content_margin_top = 8
	box.content_margin_bottom = 8
	edit.add_theme_stylebox_override("normal", box)

	var focus_box := box.duplicate() as StyleBoxFlat
	focus_box.border_color = TEXT_ACCENT
	edit.add_theme_stylebox_override("focus", focus_box)


func style_check_button(check: CheckButton, font_px: int = 19) -> void:
	var scale_mult := SettingsManager.get_ui_scale_multiplier()
	check.custom_minimum_size.y = int(round(48.0 * scale_mult))
	check.add_theme_color_override("font_color", TEXT_PARCHMENT)
	check.add_theme_color_override("font_hover_color", Color("#fff9ec"))
	check.add_theme_color_override("font_pressed_color", TEXT_PARCHMENT)
	check.add_theme_color_override("font_focus_color", TEXT_PARCHMENT)
	check.add_theme_font_size_override("font_size", int(round(float(font_px) * scale_mult)))


func style_slider(slider: HSlider) -> void:
	var scale_mult := SettingsManager.get_ui_scale_multiplier()
	slider.custom_minimum_size.y = int(round(38.0 * scale_mult))

	var track := StyleBoxFlat.new()
	track.bg_color = Color("#4a3f34")
	track.set_corner_radius_all(6)
	track.content_margin_top = 6
	track.content_margin_bottom = 6
	slider.add_theme_stylebox_override("slider", track)

	var fill := StyleBoxFlat.new()
	fill.bg_color = TEXT_ACCENT
	fill.set_corner_radius_all(6)
	fill.content_margin_top = 6
	fill.content_margin_bottom = 6
	slider.add_theme_stylebox_override("grabber_area", fill)
	slider.add_theme_stylebox_override("grabber_area_highlight", fill)

	var grabber := _get_slider_grabber_texture()
	slider.add_theme_icon_override("grabber", grabber)
	slider.add_theme_icon_override("grabber_highlight", grabber)


func _get_slider_grabber_texture() -> Texture2D:
	if _slider_grabber_tex != null:
		return _slider_grabber_tex
	var size := 28
	var img := Image.create(size, size, false, Image.FORMAT_RGBA8)
	var center := Vector2(size * 0.5 - 0.5, size * 0.5 - 0.5)
	for y in range(size):
		for x in range(size):
			var d := Vector2(x, y).distance_to(center)
			if d <= 12.5:
				var col := Color("#f5ebd6") if d <= 9.5 else Color("#8c6b43")
				img.set_pixel(x, y, col)
			elif d <= 13.5:
				var alpha := clampf(13.5 - d, 0.0, 1.0)
				img.set_pixel(x, y, Color(0.55, 0.42, 0.26, alpha))
			else:
				img.set_pixel(x, y, Color(0, 0, 0, 0))
	_slider_grabber_tex = ImageTexture.create_from_image(img)
	return _slider_grabber_tex


func trigger_touch_feedback() -> void:
	if SettingsManager.haptic_feedback and OS.has_feature("mobile"):
		Input.vibrate_handheld(18)


func make_scroll_pass(node: Control) -> void:
	if node == null:
		return
	if node is PanelContainer or node is BoxContainer or node is MarginContainer or node is Label or node is HSeparator:
		node.mouse_filter = Control.MOUSE_FILTER_PASS
	for child in node.get_children():
		if child is Control:
			make_scroll_pass(child as Control)


func enable_touch_scroll(scroll: ScrollContainer) -> void:
	if scroll == null:
		return
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
	scroll.follow_focus = true

	var vbar := scroll.get_v_scroll_bar()
	if vbar:
		vbar.custom_minimum_size.x = 14
		var bar_bg := StyleBoxFlat.new()
		bar_bg.bg_color = Color(0.12, 0.09, 0.07, 0.55)
		bar_bg.set_corner_radius_all(6)
		vbar.add_theme_stylebox_override("scroll", bar_bg)

		var grabber := StyleBoxFlat.new()
		grabber.bg_color = BORDER_BRASS
		grabber.set_corner_radius_all(6)
		grabber.content_margin_left = 4
		grabber.content_margin_right = 4
		vbar.add_theme_stylebox_override("grabber", grabber)

		var grabber_hi := grabber.duplicate() as StyleBoxFlat
		grabber_hi.bg_color = BORDER_LIGHT
		vbar.add_theme_stylebox_override("grabber_highlight", grabber_hi)
		vbar.add_theme_stylebox_override("grabber_pressed", grabber_hi)

	for child in scroll.get_children():
		if child is Control:
			make_scroll_pass(child as Control)

	var handler := _TouchScrollHandler.new(scroll)
	scroll.add_child(handler)


class _TouchScrollHandler extends Node:
	var _scroll: ScrollContainer
	var _active_touch: int = -1
	var _mouse_down: bool = false
	var _start_pos: Vector2 = Vector2.ZERO
	var _started_on_slider: bool = false
	var _dragging: bool = false
	var _velocity_y: float = 0.0
	var _scroll_accum: float = 0.0
	var _suppressed_controls: Array[Control] = []

	func _init(scroll_node: ScrollContainer) -> void:
		_scroll = scroll_node

	func _process(delta: float) -> void:
		if _scroll == null or not _scroll.is_visible_in_tree():
			return
		if not _dragging and absf(_velocity_y) > 8.0:
			_apply_scroll_delta(-_velocity_y * delta)
			_velocity_y = lerpf(_velocity_y, 0.0, minf(1.0, delta * 9.0))
			if absf(_velocity_y) <= 8.0:
				_velocity_y = 0.0

	func _input(event: InputEvent) -> void:
		if _scroll == null or not _scroll.is_visible_in_tree():
			return

		if event is InputEventScreenTouch:
			var t := event as InputEventScreenTouch
			if t.pressed:
				if _active_touch == -1 and _scroll.get_global_rect().has_point(t.position):
					_active_touch = t.index
					_start_pos = t.position
					_started_on_slider = _has_slider_at(t.position)
					_dragging = false
					_velocity_y = 0.0
					_scroll_accum = 0.0
			elif t.index == _active_touch:
				var was_dragging := _dragging
				_active_touch = -1
				_dragging = false
				_started_on_slider = false
				call_deferred("_restore_suppressed")
				if was_dragging:
					get_viewport().set_input_as_handled()

		elif event is InputEventScreenDrag:
			var d := event as InputEventScreenDrag
			if d.index != _active_touch:
				return
			_handle_drag(d.position, d.relative)

		elif event is InputEventMouseButton:
			var mb := event as InputEventMouseButton
			if mb.device == InputEvent.DEVICE_ID_EMULATION:
				return
			if mb.button_index == MOUSE_BUTTON_LEFT:
				if mb.pressed:
					if _scroll.get_global_rect().has_point(mb.position):
						_mouse_down = true
						_start_pos = mb.position
						_started_on_slider = _has_slider_at(mb.position)
						_dragging = false
						_velocity_y = 0.0
						_scroll_accum = 0.0
				elif _mouse_down:
					var was_dragging := _dragging
					_mouse_down = false
					_dragging = false
					_started_on_slider = false
					call_deferred("_restore_suppressed")
					if was_dragging:
						get_viewport().set_input_as_handled()

		elif event is InputEventMouseMotion:
			var mm := event as InputEventMouseMotion
			if mm.device == InputEvent.DEVICE_ID_EMULATION:
				if _dragging:
					get_viewport().set_input_as_handled()
				return
			if _mouse_down:
				_handle_drag(mm.position, mm.relative)

	func _handle_drag(pos: Vector2, rel: Vector2) -> void:
		if not _dragging:
			var delta_total := pos - _start_pos
			var min_y := 22.0 if _started_on_slider else 12.0
			var ratio := 1.4 if _started_on_slider else 0.8
			if absf(delta_total.y) > min_y and absf(delta_total.y) >= absf(delta_total.x) * ratio:
				_dragging = true
				_suppress_controls_at(_start_pos)
			else:
				return

		_apply_scroll_delta(-rel.y)
		_velocity_y = rel.y * 28.0
		get_viewport().set_input_as_handled()

	func _apply_scroll_delta(dy: float) -> void:
		_scroll_accum += dy
		var step := int(truncf(_scroll_accum))
		if step == 0:
			return
		_scroll_accum -= float(step)
		var vbar := _scroll.get_v_scroll_bar()
		var max_val := int(maxf(0.0, vbar.max_value - vbar.page)) if vbar else 2000
		var prev := _scroll.scroll_vertical
		_scroll.scroll_vertical = clampi(prev + step, 0, max_val)
		if _scroll.scroll_vertical == prev:
			_scroll_accum = 0.0

	func _has_slider_at(pos: Vector2) -> bool:
		return _find_slider_recursive(_scroll, pos)

	func _find_slider_recursive(node: Node, pos: Vector2) -> bool:
		for child in node.get_children():
			if child is Control:
				var c := child as Control
				if not c.visible:
					continue
				if c is HSlider and c.get_global_rect().grow(10.0).has_point(pos):
					return true
				if _find_slider_recursive(c, pos):
					return true
		return false

	func _suppress_controls_at(pos: Vector2) -> void:
		_suppressed_controls.clear()
		_collect_hit_controls(_scroll, pos)
		for ctrl in _suppressed_controls:
			ctrl.mouse_filter = Control.MOUSE_FILTER_IGNORE
			if ctrl is BaseButton:
				var b := ctrl as BaseButton
				if not b.disabled:
					b.disabled = true
					b.disabled = false

	func _collect_hit_controls(node: Node, pos: Vector2) -> void:
		for child in node.get_children():
			if child is Control:
				var c := child as Control
				if not c.visible:
					continue
				if (c is BaseButton or c is HSlider or c is LineEdit) and c.get_global_rect().has_point(pos):
					if c.mouse_filter != Control.MOUSE_FILTER_IGNORE:
						_suppressed_controls.append(c)
					if c is LineEdit and c.has_focus():
						c.release_focus()
				_collect_hit_controls(c, pos)

	func _restore_suppressed() -> void:
		for ctrl in _suppressed_controls:
			if is_instance_valid(ctrl):
				ctrl.mouse_filter = Control.MOUSE_FILTER_STOP
		_suppressed_controls.clear()
