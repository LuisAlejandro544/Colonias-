# Estado de sesión activa compartido entre las pantallas de menús y el mundo 2D.
#
# Lógica: Almacena los parámetros de la colonia actual (ranura, nombre, semilla,
# bioma, tamaño, colonos y estado completo guardado del mundo) y recuerda a qué
# pantalla regresar al salir de Ajustes o Ranuras.

extends Node

var active_slot: int = 1
var colony_name: String = "Valle del Roble"
var seed_value: int = 24819
var biome_code: String = "temperate_forest"
var map_size: int = 64
var initial_colonists: int = 4
var day_count: int = 1
var saved_colonists: Array = []
var saved_world_state: Dictionary = {}

var return_scene_path: String = "res://scenes/ui/main_menu.tscn"
var in_world_mode: bool = false


func start_new_colony(slot: int, name_str: String, seed_num: int, biome: String, size_val: int, colonist_num: int) -> void:
	active_slot = slot
	colony_name = name_str.strip_edges() if not name_str.strip_edges().is_empty() else "Valle del Roble"
	seed_value = seed_num
	biome_code = biome
	map_size = size_val
	initial_colonists = colonist_num
	day_count = 1
	saved_colonists = []
	saved_world_state = {}
	in_world_mode = true


func load_from_slot_data(slot: int, data: Dictionary) -> void:
	active_slot = slot
	colony_name = str(data.get("colony_name", "Valle del Roble"))
	seed_value = int(data.get("seed", 24819))
	biome_code = str(data.get("biome_code", "temperate_forest"))
	map_size = int(data.get("map_size", 64))
	day_count = int(data.get("day", 1))
	var cols = data.get("colonists", [])
	saved_colonists = cols if cols is Array else []
	saved_world_state = data.duplicate(true)
	initial_colonists = max(1, saved_colonists.size())
	in_world_mode = true
