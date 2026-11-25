# Gestor de ranuras de partidas guardadas en JSON dentro de `user://saves/`.
#
# Lógica: Administra 3 ranuras persistentes para guardar el estado completo de la
# colonia (nombre, semilla, bioma terrestre, tamaño, día y colonos), listar su
# resumen en pantalla, cargarlas o eliminarlas de forma segura.

extends Node

const TOTAL_SLOTS := 3
const SAVES_DIR := "user://saves"
const SLOT_TEMPLATE := "user://saves/slot_%d.json"


func _ready() -> void:
	if not DirAccess.dir_exists_absolute(SAVES_DIR):
		DirAccess.make_dir_recursive_absolute(SAVES_DIR)


func get_slot_path(slot_index: int) -> String:
	return SLOT_TEMPLATE % clampi(slot_index, 1, TOTAL_SLOTS)


func biome_label_es(biome_code: String) -> String:
	match biome_code:
		"fertile_meadow":
			return "Pradera Fértil"
		"rocky_highlands":
			return "Sierra Rocosa"
		_:
			return "Bosque Templado"


func list_slots() -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for i in range(1, TOTAL_SLOTS + 1):
		var path := get_slot_path(i)
		if FileAccess.file_exists(path):
			var data := load_game(i)
			if not data.is_empty():
				var colonists_arr = data.get("colonists", [])
				result.append({
					"slot": i,
					"exists": true,
					"colony_name": str(data.get("colony_name", "Colonia sin nombre")),
					"biome_code": str(data.get("biome_code", "temperate_forest")),
					"biome_label": biome_label_es(str(data.get("biome_code", "temperate_forest"))),
					"seed": int(data.get("seed", 1)),
					"map_size": int(data.get("map_size", 64)),
					"colonist_count": colonists_arr.size() if colonists_arr is Array else int(data.get("colonist_count", 3)),
					"day": int(data.get("day", 1)),
					"saved_at": str(data.get("saved_at", "Reciente")),
					"saved_unix": int(data.get("saved_unix", 0)),
				})
				continue
		result.append({
			"slot": i,
			"exists": false,
			"colony_name": "",
			"biome_code": "temperate_forest",
			"biome_label": "",
			"seed": 0,
			"map_size": 64,
			"colonist_count": 0,
			"day": 0,
			"saved_at": "",
			"saved_unix": 0,
		})
	return result


func get_latest_occupied_slot() -> int:
	var best_slot := -1
	var best_unix := -1
	for info in list_slots():
		if info.get("exists", false):
			var ts := int(info.get("saved_unix", 0))
			if ts >= best_unix:
				best_unix = ts
				best_slot = int(info.get("slot", -1))
	return best_slot


func save_game(slot_index: int, payload: Dictionary) -> bool:
	if not DirAccess.dir_exists_absolute(SAVES_DIR):
		DirAccess.make_dir_recursive_absolute(SAVES_DIR)

	var data := payload.duplicate(true)
	data["version"] = 1
	data["saved_at"] = Time.get_datetime_string_from_system(false, true).replace("T", " ")
	data["saved_unix"] = int(Time.get_unix_time_from_system())

	var file := FileAccess.open(get_slot_path(slot_index), FileAccess.WRITE)
	if file == null:
		return false
	file.store_string(JSON.stringify(data, "\t"))
	return true


func load_game(slot_index: int) -> Dictionary:
	var path := get_slot_path(slot_index)
	if not FileAccess.file_exists(path):
		return {}
	var file := FileAccess.open(path, FileAccess.READ)
	if file == null:
		return {}
	var json := JSON.new()
	if json.parse(file.get_as_text()) != OK:
		return {}
	var parsed = json.get_data()
	return parsed if parsed is Dictionary else {}


func delete_slot(slot_index: int) -> bool:
	var path := get_slot_path(slot_index)
	if not FileAccess.file_exists(path):
		return false
	return DirAccess.remove_absolute(path) == OK
