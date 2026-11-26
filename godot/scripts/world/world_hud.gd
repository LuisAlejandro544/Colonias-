# Coordinador modular de la interfaz superpuesta (HUD) del mundo 2D en vista centinela.
#
# Lógica: Muestra la barra superior (colonia, día, estado del Líder de Colonia,
# inventario global incluyendo carpintería, velocidad y zoom), la barra inferior
# de acciones (Observar, Talar, Minar, Construir, Trabajos y Cancelar) y las
# notificaciones toast, delegando los paneles y popups complejos a módulos
# dedicados: `BuildMenuPopup`, `ColonistInspectorPanel`, `ColonistInventoryPopup`,
# `ColonistStatsPopup`, `CarpenterBenchPopup`, `JobsPopup` y `PauseMenuPopup`.

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
const BuildMenuPopupScript := preload("res://scripts/world/build_menu_popup.gd")
const ColonistInventoryPopupScript := preload("res://scripts/world/colonist_inventory_popup.gd")
const ColonistStatsPopupScript := preload("res://scripts/world/colonist_stats_popup.gd")
const CarpenterBenchPopupScript := preload("res://scripts/world/carpenter_bench_popup.gd")
const ColonistInspectorPanelScript := preload("res://scripts/world/colonist_inspector_panel.gd")
const PauseMenuPopupScript := preload("res://scripts/world/pause_menu_popup.gd")

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

var _build_popup: ColorRect
var _inspector_panel: PanelContainer
var _inv_popup: ColorRect
var _stats_popup: ColorRect
var _bench_popup: ColorRect
var _jobs_popup: ColorRect
var _pause_popup: ColorRect

var _inspected_colonist: Node = null
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
		"wood_plank": load("res://assets/items/item_wood_planks.svg") as Texture2D,
		"wood_bucket": load("res://assets/items/item_wood_bucket.svg") as Texture2D,
		"wood_door_item": load("res://assets/items/item_wood_door.svg") as Texture2D,
		"wood_crate": load("res://assets/items/item_wood_crate.svg") as Texture2D,
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

	_build_top_bar(root)
	_build_bottom_bar(root)

	_inspector_panel = ColonistInspectorPanelScript.new() as PanelContainer
	root.add_child(_inspector_panel)
	_inspector_panel.connect("inventory_requested", func():
		if _inv_popup and _inspected_colonist != null:
			_inv_popup.call("open_for_colonist", _inspected_colonist, _tool_icons, _item_icons)
	)
	_inspector_panel.connect("stats_requested", func():
		if _stats_popup and _inspected_colonist != null:
			_stats_popup.call("open_for_colonist", _inspected_colonist)
	)
	_inspector_panel.connect("jobs_requested", func(): jobs_opened.emit())
	_inspector_panel.connect("follow_toggled", func(enabled: bool): follow_toggled.emit(enabled))

	_toast_label = Label.new()
	_toast_label.visible = false
	_toast_label.set_anchors_and_offsets_preset(Control.PRESET_CENTER_TOP)
	_toast_label.grow_horizontal = Control.GROW_DIRECTION_BOTH
	_toast_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_toast_label.offset_top = 128
	_toast_label.add_theme_color_override("font_color", Color("#f6eccf"))
	_toast_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(19))
	root.add_child(_toast_label)

	_build_popup = BuildMenuPopupScript.new() as ColorRect
	root.add_child(_build_popup)
	_build_popup.connect("build_mode_selected", func(code: String): _set_active_tool_mode(code))

	_inv_popup = ColonistInventoryPopupScript.new() as ColorRect
	root.add_child(_inv_popup)

	_stats_popup = ColonistStatsPopupScript.new() as ColorRect
	root.add_child(_stats_popup)

	_bench_popup = CarpenterBenchPopupScript.new() as ColorRect
	root.add_child(_bench_popup)

	_jobs_popup = JobsPopupScript.new() as ColorRect
	root.add_child(_jobs_popup)
	_jobs_popup.connect("role_toggled", func(cid: int, r_code: String, enabled: bool):
		role_toggle_requested.emit(cid, r_code, enabled)
	)

	_pause_popup = PauseMenuPopupScript.new() as ColorRect
	root.add_child(_pause_popup)
	_pause_popup.connect("save_requested", func(): save_requested.emit())

	_set_active_tool_mode("observe")


func _build_top_bar(root: Control) -> void:
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
	_inventory_label.text = "Madera: 0 • Piedra: 0 • Hierro: 0 • Cobre: 0 • Semillas: 0 • Zanahorias: 0\nTablones: 0 • Cubos: 0 • Puertas: 0 • Arcones: 0"
	_inventory_label.add_theme_color_override("font_color", Color("#e5d7be"))
	_inventory_label.add_theme_font_size_override("font_size", UiTheme.scaled_px(14))
	info_v.add_child(_inventory_label)

	var spacer := Control.new()
	spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	spacer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	top_row.add_child(spacer)

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
		if _pause_popup:
			_pause_popup.call("open_popup")
	)
	ctrl_row.add_child(menu_btn)


func _build_bottom_bar(root: Control) -> void:
	var bottom_margin := MarginContainer.new()
	bottom_margin.anchor_left = 0.0
	bottom_margin.anchor_top = 1.0
	bottom_margin.anchor_right = 1.0
	bottom_margin.anchor_bottom = 1.0
	bottom_margin.offset_left = 0.0
	bottom_margin.offset_top = 0.0
	bottom_margin.offset_right = 0.0
	bottom_margin.offset_bottom = 0.0
	bottom_margin.grow_horizontal = Control.GROW_DIRECTION_BOTH
	bottom_margin.grow_vertical = Control.GROW_DIRECTION_BEGIN
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
		if _build_popup:
			_build_popup.call("open_popup")
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

	var is_build_mode: bool = mode_code in ["storage", "farm", "build_wall", "build_door", "build_carpenter_bench"]
	if _build_menu_btn:
		UiTheme.style_button(_build_menu_btn, "amber" if is_build_mode else "primary", 48, 17)

	if _build_popup:
		_build_popup.call("update_selected_mode", mode_code)

	match mode_code:
		"chop":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Talar: arrastra sobre árboles • El líder reunirá al grupo para dar la orden"
		"mine":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Minar: arrastra sobre rocas o vetas de hierro • El líder comunicará la orden"
		"storage":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Construir Almacén: arrastra donde guardar madera, piedra, menas, piezas y cosechas"
		"farm":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Construir Cultivo (Zanahorias): arrastra tierra o pasto para arar y sembrar"
		"build_wall":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Construir Pared de Madera (2 troncos/bloque): arrastra para colocar planos"
		"build_door":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Instalar Puerta de Madera (1 puerta fabricada): arrastra o toca un hueco entre muros"
		"build_carpenter_bench":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Mesa de Carpintero (6 troncos): toca una casilla para levantar el taller"
		"cancel":
			_mode_hint_panel.visible = true
			_mode_hint_label.text = "Cancelar: arrastra sobre áreas, planos o marcas para limpiarlas"
		_:
			_mode_hint_panel.visible = false

	tool_mode_changed.emit(mode_code)


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
		_inventory_label.text = "Madera: %d • Piedra: %d • Hierro: %d • Cobre: %d • Semillas: %d • Zanahorias: %d\nTablones: %d • Cubos: %d • Puertas: %d • Arcones: %d" % [
			int(inv.get("wood", 0)),
			int(inv.get("stone", 0)),
			int(inv.get("iron", 0)),
			int(inv.get("copper", 0)),
			int(inv.get("carrot_seeds", 0)),
			int(inv.get("carrots", 0)),
			int(inv.get("wood_planks", 0)),
			int(inv.get("wood_buckets", 0)),
			int(inv.get("wood_doors", 0)),
			int(inv.get("wood_crates", 0)),
		]


func show_colonist_info(colonist: Node) -> void:
	_inspected_colonist = colonist
	if _inspector_panel:
		_inspector_panel.call("show_colonist", colonist)
	if colonist == null:
		if _inv_popup:
			_inv_popup.call("close_popup")
		if _stats_popup:
			_stats_popup.call("close_popup")
		return
	if _inv_popup:
		_inv_popup.call("refresh_if_visible", colonist, _tool_icons, _item_icons)
	if _stats_popup:
		_stats_popup.call("refresh_if_visible", colonist)


func open_carpenter_bench(tile_x: int, tile_y: int, colony_sim: Node) -> bool:
	if _bench_popup == null or colony_sim == null:
		return false
	var bench_var: Variant = colony_sim.call("get_carpentry_bench_at", tile_x, tile_y)
	if bench_var is Dictionary and bool((bench_var as Dictionary).get("found", false)):
		_bench_popup.call("open_for_bench", bench_var as Dictionary, colony_sim, _item_icons)
		return true
	return false


func refresh_carpenter_bench_if_visible() -> void:
	if _bench_popup:
		_bench_popup.call("refresh_if_visible")


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
