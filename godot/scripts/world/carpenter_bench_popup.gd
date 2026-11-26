# Popup modular de fabricación en Talleres (Mesa de Carpintero, Mesa de Masón y Horno).
#
# Lógica: Se abre al tocar una estación construida en el mapa. Permite revisar
# especialistas disponibles y su Inteligencia (`INT`), encolar recetas con uno o
# dos ingredientes según `INT` mínima, y cancelar piezas pendientes en la cola.

class_name CarpenterBenchPopup
extends ColorRect

signal queue_changed

var _bench_id: int = -1
var _bench_x: int = 0
var _bench_y: int = 0
var _station_kind: String = "carpenter"
var _colony_sim: Node = null
var _item_icons: Dictionary = {}

var _title_label: Label
var _sub_label: Label
var _rec_title_label: Label
var _status_label: Label
var _feedback_label: Label
var _recipes_box: VBoxContainer
var _queue_box: VBoxContainer


func _ready() -> void:
	visible = false
	color = Color(0.05, 0.04, 0.03, 0.74)
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_build_ui()


func _build_ui() -> void:
	var center := CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	add_child(center)

	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(960, 560)
	UiTheme.apply_panel_style(panel, UiTheme.BG_PANEL, UiTheme.BORDER_LIGHT, 16, 24, 20)
	center.add_child(panel)

	var root_v := VBoxContainer.new()
	root_v.add_theme_constant_override("separation", 12)
	panel.add_child(root_v)

	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 12)
	root_v.add_child(head)

	var head_txt := VBoxContainer.new()
	head_txt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	head_txt.add_theme_constant_override("separation", 3)
	head.add_child(head_txt)

	_title_label = Label.new()
	_title_label.text = "Mesa de Carpintero"
	UiTheme.style_title(_title_label, 27)
	head_txt.add_child(_title_label)

	_sub_label = Label.new()
	_sub_label.text = "Procesa troncos en tablones y fabrica vasos, cubos, puertas y arcones según la Inteligencia del carpintero."
	_sub_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	UiTheme.style_subtitle(_sub_label, 15)
	head_txt.add_child(_sub_label)

	var close_btn := Button.new()
	close_btn.text = "Cerrar"
	UiTheme.style_button(close_btn, "secondary", 48, 18)
	close_btn.pressed.connect(func():
		UiTheme.trigger_touch_feedback()
		close_popup()
	)
	head.add_child(close_btn)

	var info_bar := PanelContainer.new()
	UiTheme.apply_panel_style(info_bar, Color("#1e1812"), UiTheme.BORDER_BRASS, 10, 14, 9)
	root_v.add_child(info_bar)

	var info_row := HBoxContainer.new()
	info_row.add_theme_constant_override("separation", 10)
	info_bar.add_child(info_row)

	_status_label = Label.new()
	_status_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_status_label.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
	_status_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(15))
	info_row.add_child(_status_label)

	_feedback_label = Label.new()
	_feedback_label.add_theme_color_override("font_color", UiTheme.TEXT_ACCENT)
	_feedback_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(14))
	info_row.add_child(_feedback_label)

	var columns := HBoxContainer.new()
	columns.size_flags_vertical = Control.SIZE_EXPAND_FILL
	columns.add_theme_constant_override("separation", 14)
	root_v.add_child(columns)

	# Columna izquierda: Recetas
	var left_v := VBoxContainer.new()
	left_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left_v.size_flags_stretch_ratio = 1.6
	left_v.add_theme_constant_override("separation", 8)
	columns.add_child(left_v)

	_rec_title_label = Label.new()
	_rec_title_label.text = "Recetas de Carpintería"
	UiTheme.style_title(_rec_title_label, 19)
	left_v.add_child(_rec_title_label)

	var rec_scroll := ScrollContainer.new()
	rec_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	rec_scroll.custom_minimum_size = Vector2(0, 320)
	left_v.add_child(rec_scroll)

	_recipes_box = VBoxContainer.new()
	_recipes_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_recipes_box.add_theme_constant_override("separation", 10)
	rec_scroll.add_child(_recipes_box)
	UiTheme.enable_touch_scroll(rec_scroll)

	# Columna derecha: Cola activa
	var right_v := VBoxContainer.new()
	right_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_v.size_flags_stretch_ratio = 1.0
	right_v.add_theme_constant_override("separation", 8)
	columns.add_child(right_v)

	var q_title := Label.new()
	q_title.text = "Cola de Fabricación"
	UiTheme.style_title(q_title, 19)
	right_v.add_child(q_title)

	var q_scroll := ScrollContainer.new()
	q_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	q_scroll.custom_minimum_size = Vector2(0, 320)
	right_v.add_child(q_scroll)

	_queue_box = VBoxContainer.new()
	_queue_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_queue_box.add_theme_constant_override("separation", 8)
	q_scroll.add_child(_queue_box)
	UiTheme.enable_touch_scroll(q_scroll)


func open_for_bench(bench_snap: Dictionary, colony_sim: Node, item_icons: Dictionary) -> void:
	if not bool(bench_snap.get("found", false)):
		return
	_colony_sim = colony_sim
	_item_icons = item_icons
	_bench_id = int(bench_snap.get("id", -1))
	_bench_x = int(bench_snap.get("x", 0))
	_bench_y = int(bench_snap.get("y", 0))
	_station_kind = str(bench_snap.get("station_kind", "carpenter"))
	_feedback_label.text = ""
	visible = true
	_populate(bench_snap)


func refresh_if_visible() -> void:
	if not visible or _colony_sim == null:
		return
	var snap_var: Variant = _colony_sim.call("get_workshop_at", _bench_x, _bench_y)
	if snap_var is Dictionary and bool((snap_var as Dictionary).get("found", false)):
		_populate(snap_var as Dictionary)


func close_popup() -> void:
	visible = false


func _populate(snap: Dictionary) -> void:
	_station_kind = str(snap.get("station_kind", _station_kind))
	var station_label := str(snap.get("station_label", "Mesa de Carpintero"))
	_title_label.text = "%s #%d (%d, %d)" % [
		station_label,
		int(snap.get("id", _bench_id)),
		_bench_x,
		_bench_y,
	]

	var role_plural := "Carpinteros"
	var role_singular := "Carpintero"
	if _station_kind == "mason":
		role_plural = "Masones"
		role_singular = "Masón"
		_sub_label.text = "Labra bloques de piedra para fabricar ladrillos, morteros, ruedas de molino y estatuillas."
		_rec_title_label.text = "Recetas de Cantería"
	elif _station_kind == "furnace":
		role_plural = "Masones"
		role_singular = "Masón"
		_sub_label.text = "Alimenta el horno de piedra y ladrillo con carbón para fundir menas de cobre en lingotes."
		_rec_title_label.text = "Recetas de Fundición"
	else:
		_sub_label.text = "Procesa troncos en tablones y fabrica vasos, cubos, puertas y arcones según la Inteligencia del carpintero."
		_rec_title_label.text = "Recetas de Carpintería"

	var inv: Dictionary = snap.get("inventory", {})
	var specialists: Array = snap.get("carpenters", [])
	var max_int := int(snap.get("max_carpenter_int", 0))
	var spec_summary := "Sin %s asignados (asígnalo en Trabajos)" % role_plural.to_lower()
	if not specialists.is_empty():
		var parts: Array[String] = []
		for c_var in specialists:
			if c_var is Dictionary:
				var cd: Dictionary = c_var
				parts.append("%s (INT %d • Nv.%d)" % [
					str(cd.get("name", "")),
					int(cd.get("intelligence", 1)),
					int(cd.get("level", 1)),
				])
		spec_summary = "%s: %s (Máx INT %d)" % [role_plural, ", ".join(parts), max_int]

	if _station_kind == "mason" or _station_kind == "furnace":
		_status_label.text = "%s\nDisponible: %d piedra • %d ladrillos • %d cobre • %d carbón • %d lingotes • %d morteros • %d molinos • %d estatuillas" % [
			spec_summary,
			int(inv.get("stone", 0)),
			int(inv.get("stone_bricks", 0)),
			int(inv.get("copper", 0)),
			int(inv.get("coal", 0)),
			int(inv.get("copper_ingots", 0)),
			int(inv.get("stone_mortars", 0)),
			int(inv.get("stone_millstones", 0)),
			int(inv.get("stone_statuettes", 0)),
		]
	else:
		_status_label.text = "%s\nDisponible: %d troncos • %d tablones • %d vasos • %d cubos • %d puertas • %d arcones" % [
			spec_summary,
			int(inv.get("wood", 0)),
			int(inv.get("wood_planks", 0)),
			int(inv.get("wood_cups", 0)),
			int(inv.get("wood_buckets", 0)),
			int(inv.get("wood_doors", 0)),
			int(inv.get("wood_crates", 0)),
		]

	for ch in _recipes_box.get_children():
		_recipes_box.remove_child(ch)
		ch.queue_free()

	var recipes: Array = snap.get("recipes", [])
	for r_var in recipes:
		if not (r_var is Dictionary):
			continue
		var r: Dictionary = r_var
		var code := str(r.get("code", ""))
		var label_txt := str(r.get("label", ""))
		var desc_txt := str(r.get("description", ""))
		var min_int := int(r.get("min_intelligence", 1))
		var in_label := str(r.get("input_label", ""))
		var in_amt := int(r.get("input_amount", 1))
		var in2_label := str(r.get("input2_label", ""))
		var in2_amt := int(r.get("input2_amount", 0))
		var avail_in2 := int(r.get("available_input2", 0))
		var out_code := str(r.get("output_code", ""))
		var out_amt := int(r.get("output_amount", 1))
		var avail_in := int(r.get("available_input", 0))
		var int_ok := bool(r.get("can_craft_by_colony", false))

		var card := PanelContainer.new()
		card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		UiTheme.apply_panel_style(card, UiTheme.BG_PANEL_ALT, UiTheme.BORDER_BRASS, 12, 12, 10)
		_recipes_box.add_child(card)

		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 12)
		card.add_child(row)

		var icon_box := PanelContainer.new()
		icon_box.custom_minimum_size = Vector2(58, 58)
		icon_box.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		UiTheme.apply_panel_style(icon_box, Color("#1b1510"), UiTheme.BORDER_BRASS, 10, 6, 6)
		row.add_child(icon_box)

		var icon := TextureRect.new()
		icon.texture = _item_icons.get(out_code, null)
		icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
		icon.custom_minimum_size = Vector2(42, 42)
		icon_box.add_child(icon)

		var txt_v := VBoxContainer.new()
		txt_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		txt_v.add_theme_constant_override("separation", 3)
		row.add_child(txt_v)

		var title_r := Label.new()
		title_r.text = "%s (x%d)" % [label_txt, out_amt]
		UiTheme.style_title(title_r, 18)
		txt_v.add_child(title_r)

		var desc_r := Label.new()
		desc_r.text = desc_txt
		desc_r.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		UiTheme.style_subtitle(desc_r, 14)
		txt_v.add_child(desc_r)

		var req_lbl := Label.new()
		var int_status := "Habilitado (INT %d+)" % min_int if int_ok else "Falta %s con INT %d+" % [role_singular, min_int]
		var cost_txt := "Costo: %d %s (Hay: %d)" % [in_amt, in_label, avail_in]
		if in2_amt > 0:
			cost_txt += " + %d %s (Hay: %d)" % [in2_amt, in2_label, avail_in2]
		req_lbl.text = "%s  •  %s" % [cost_txt, int_status]
		req_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		req_lbl.add_theme_color_override(
			"font_color",
			Color("#d9be82") if int_ok else Color("#e58a76")
		)
		req_lbl.add_theme_font_size_override("font_size", UiTheme.scaled_px(13))
		txt_v.add_child(req_lbl)

		var add_btn := Button.new()
		add_btn.text = "Fabricar"
		add_btn.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		UiTheme.style_button(add_btn, "primary" if int_ok else "secondary", 44, 16)
		add_btn.pressed.connect(func():
			UiTheme.trigger_touch_feedback()
			if _colony_sim == null:
				return
			var res_var: Variant = _colony_sim.call("queue_workshop_recipe", _station_kind, _bench_id, code)
			if res_var is Dictionary:
				var res: Dictionary = res_var
				var ok := bool(res.get("ok", false))
				_feedback_label.text = str(res.get("message", ""))
				_feedback_label.add_theme_color_override(
					"font_color",
					Color("#a7d489") if ok else Color("#e58a76")
				)
			refresh_if_visible()
			queue_changed.emit()
		)
		row.add_child(add_btn)

	UiTheme.make_scroll_pass(_recipes_box)

	for ch in _queue_box.get_children():
		_queue_box.remove_child(ch)
		ch.queue_free()

	var queue_arr: Array = snap.get("queue", [])
	if queue_arr.is_empty():
		var empty_lbl := Label.new()
		empty_lbl.text = "La cola está vacía.\nElige una pieza a la izquierda para que tus colonos la procesen."
		empty_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		UiTheme.style_subtitle(empty_lbl, 15)
		_queue_box.add_child(empty_lbl)
	else:
		for q_var in queue_arr:
			if not (q_var is Dictionary):
				continue
			var q: Dictionary = q_var
			var q_idx := int(q.get("index", 0))
			var q_label := str(q.get("label", ""))
			var q_int := int(q.get("min_intelligence", 1))

			var q_card := PanelContainer.new()
			UiTheme.apply_panel_style(q_card, UiTheme.BG_PANEL_ALT, UiTheme.BORDER_BRASS, 10, 12, 8)
			_queue_box.add_child(q_card)

			var q_row := HBoxContainer.new()
			q_row.add_theme_constant_override("separation", 8)
			q_card.add_child(q_row)

			var q_txt := Label.new()
			q_txt.text = "#%d  %s (INT %d+)" % [q_idx + 1, q_label, q_int]
			q_txt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			q_txt.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
			q_txt.add_theme_color_override("font_color", UiTheme.TEXT_PARCHMENT)
			q_txt.add_theme_font_size_override("font_size", UiTheme.scaled_px(15))
			q_row.add_child(q_txt)

			var cancel_btn := Button.new()
			cancel_btn.text = "Quitar"
			UiTheme.style_button(cancel_btn, "danger", 38, 14)
			cancel_btn.pressed.connect(func():
				UiTheme.trigger_touch_feedback()
				if _colony_sim != null:
					_colony_sim.call("cancel_workshop_queue_item", _station_kind, _bench_id, q_idx)
				refresh_if_visible()
				queue_changed.emit()
			)
			q_row.add_child(cancel_btn)

	UiTheme.make_scroll_pass(_queue_box)
