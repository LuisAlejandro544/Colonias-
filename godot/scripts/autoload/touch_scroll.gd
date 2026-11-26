# Módulo reutilizable de desplazamiento táctil suave con inercia para Android.
#
# Lógica: Configura cualquier `ScrollContainer` para evitar clics accidentales al
# deslizar con el dedo sobre botones, deslizadores o tarjetas, gestiona arrastre
# táctil/ratón vertical con inercia y ajusta `mouse_filter` recursivamente.

class_name TouchScroll
extends RefCounted


static func make_scroll_pass(node: Control) -> void:
	if node == null:
		return
	if node is PanelContainer or node is BoxContainer or node is MarginContainer or node is Label or node is HSeparator:
		node.mouse_filter = Control.MOUSE_FILTER_PASS
	for child in node.get_children():
		if child is Control:
			make_scroll_pass(child as Control)


static func enable_touch_scroll(
	scroll: ScrollContainer,
	grabber_color: Color = Color("#8c6b43"),
	grabber_hi_color: Color = Color("#b8925a")
) -> void:
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
		grabber.bg_color = grabber_color
		grabber.set_corner_radius_all(6)
		grabber.content_margin_left = 4
		grabber.content_margin_right = 4
		vbar.add_theme_stylebox_override("grabber", grabber)

		var grabber_hi := grabber.duplicate() as StyleBoxFlat
		grabber_hi.bg_color = grabber_hi_color
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
		var step := int(_scroll_accum)
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
