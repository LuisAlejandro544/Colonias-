# Gestor persistente de ajustes del juego almacenados en `user://settings.cfg`.
#
# Lógica: Carga, modifica, restablece y guarda las preferencias de audio,
# escala táctil móvil, vibración háptica, cuadrícula del terreno y sensibilidad
# de la cámara centinela.

extends Node

const SETTINGS_PATH := "user://settings.cfg"

var master_volume: float = 85.0
var ambient_volume: float = 75.0
var ui_volume: float = 80.0
var ui_scale_mode: int = 0 # 0 = Normal, 1 = Grande (dedos)
var haptic_feedback: bool = true
var show_terrain_grid: bool = false
var pan_sensitivity: float = 1.0
var zoom_speed: float = 1.0


func _ready() -> void:
	load_settings()


func get_ui_scale_multiplier() -> float:
	return 1.18 if ui_scale_mode == 1 else 1.0


func load_settings() -> void:
	var cfg := ConfigFile.new()
	if cfg.load(SETTINGS_PATH) != OK:
		return
	master_volume = float(cfg.get_value("audio", "master_volume", 85.0))
	ambient_volume = float(cfg.get_value("audio", "ambient_volume", 75.0))
	ui_volume = float(cfg.get_value("audio", "ui_volume", 80.0))
	ui_scale_mode = int(cfg.get_value("display", "ui_scale_mode", 0))
	haptic_feedback = bool(cfg.get_value("display", "haptic_feedback", true))
	show_terrain_grid = bool(cfg.get_value("display", "show_terrain_grid", false))
	pan_sensitivity = float(cfg.get_value("camera", "pan_sensitivity", 1.0))
	zoom_speed = float(cfg.get_value("camera", "zoom_speed", 1.0))


func save_settings() -> void:
	var cfg := ConfigFile.new()
	cfg.set_value("audio", "master_volume", master_volume)
	cfg.set_value("audio", "ambient_volume", ambient_volume)
	cfg.set_value("audio", "ui_volume", ui_volume)
	cfg.set_value("display", "ui_scale_mode", ui_scale_mode)
	cfg.set_value("display", "haptic_feedback", haptic_feedback)
	cfg.set_value("display", "show_terrain_grid", show_terrain_grid)
	cfg.set_value("camera", "pan_sensitivity", pan_sensitivity)
	cfg.set_value("camera", "zoom_speed", zoom_speed)
	cfg.save(SETTINGS_PATH)


func reset_defaults() -> void:
	master_volume = 85.0
	ambient_volume = 75.0
	ui_volume = 80.0
	ui_scale_mode = 0
	haptic_feedback = true
	show_terrain_grid = false
	pan_sensitivity = 1.0
	zoom_speed = 1.0
	save_settings()
