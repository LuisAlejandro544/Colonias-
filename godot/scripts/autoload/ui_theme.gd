# Módulo central de identidad visual cálida ("Madera de Expedición y Pergamino").
#
# Lógica: Define la paleta cromática orgánica sin estética cyberpunk ni minimalismo
# soso, y provee constructores reutilizables de paneles, tarjetas y botones
# adaptados a pantallas táctiles móviles.

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


func apply_panel_style(panel: PanelContainer, bg_color: Color = BG_PANEL, border_color: Color = BORDER_BRASS, radius: int = 12, pad_h: int = 20, pad_v: int = 16) -> void:
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


func style_button(btn: Button, variant: String = "secondary", min_height: int = 48) -> void:
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
	btn.custom_minimum_size.y = int(min_height * scale_mult)
	btn.add_theme_color_override("font_color", TEXT_PARCHMENT)
	btn.add_theme_color_override("font_hover_color", Color("#fff9ec"))
	btn.add_theme_color_override("font_pressed_color", Color("#fff4db"))
	btn.add_theme_color_override("font_disabled_color", Color("#857968"))
	btn.add_theme_font_size_override("font_size", int(18 * scale_mult))

	var normal := StyleBoxFlat.new()
	normal.bg_color = base_col
	normal.border_color = border_col
	normal.set_border_width_all(2)
	normal.set_corner_radius_all(8)
	normal.content_margin_left = 18
	normal.content_margin_right = 18
	normal.content_margin_top = 10
	normal.content_margin_bottom = 10

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


func style_title(label: Label, size_px: int = 34) -> void:
	var scale_mult := SettingsManager.get_ui_scale_multiplier()
	label.add_theme_color_override("font_color", TEXT_PARCHMENT)
	label.add_theme_font_size_override("font_size", int(size_px * scale_mult))


func style_subtitle(label: Label, size_px: int = 16) -> void:
	var scale_mult := SettingsManager.get_ui_scale_multiplier()
	label.add_theme_color_override("font_color", TEXT_MUTED)
	label.add_theme_font_size_override("font_size", int(size_px * scale_mult))


func trigger_touch_feedback() -> void:
	if SettingsManager.haptic_feedback and OS.has_feature("mobile"):
		Input.vibrate_handheld(18)
