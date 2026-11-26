# Puente de conversión y empaquetado JSON entre `SaveManager` y `ColonySimulation` (Rust).
#
# Lógica: Normaliza explícitamente los tipos numéricos parseados por `JSON.parse()`
# (`int(...)`, `float(...)`, `PackedInt32Array`) para cumplir las conversiones
# estrictas de `gdext` al restaurar una partida, y arma el diccionario completo
# de guardado para la ranura activa.

class_name SaveBridge
extends RefCounted


static func convert_save_dict_for_rust(raw: Dictionary) -> Dictionary:
	var out := raw.duplicate(true)
	out["day"] = int(out.get("day", 1))
	for key in ["props", "blocked", "zones", "structures"]:
		var val = out.get(key, [])
		if val is Array:
			var packed := PackedInt32Array()
			packed.resize(val.size())
			for i in range(val.size()):
				packed[i] = int(val[i])
			out[key] = packed

	var crops_raw = out.get("crops", [])
	if crops_raw is Array:
		var crops_clean: Array = []
		for entry in crops_raw:
			if entry is Dictionary:
				crops_clean.append({
					"x": int(entry.get("x", 0)),
					"y": int(entry.get("y", 0)),
					"stage": int(entry.get("stage", 0)),
					"progress": float(entry.get("progress", 0.0)),
					"tend_cooldown": float(entry.get("tend_cooldown", 0.0)),
				})
		out["crops"] = crops_clean

	var items_raw = out.get("items", [])
	if items_raw is Array:
		var items_clean: Array = []
		for entry in items_raw:
			if entry is Dictionary:
				items_clean.append({
					"kind": str(entry.get("kind", "")),
					"x": int(entry.get("x", 0)),
					"y": int(entry.get("y", 0)),
					"amount": int(entry.get("amount", 1)),
					"stored": bool(entry.get("stored", false)),
				})
		out["items"] = items_clean

	var orders_raw = out.get("orders", [])
	if orders_raw is Array:
		var orders_clean: Array = []
		for entry in orders_raw:
			if entry is Dictionary:
				orders_clean.append({
					"kind": int(entry.get("kind", 1)),
					"x": int(entry.get("x", 0)),
					"y": int(entry.get("y", 0)),
					"status": int(entry.get("status", 0)),
				})
		out["orders"] = orders_clean

	var benches_raw = out.get("benches", [])
	if benches_raw is Array:
		var benches_clean: Array = []
		for entry in benches_raw:
			if entry is Dictionary:
				var q_clean: Array = []
				var q_raw = entry.get("queue", [])
				if q_raw is Array:
					for r_code in q_raw:
						q_clean.append(str(r_code))
				benches_clean.append({
					"id": int(entry.get("id", 1)),
					"x": int(entry.get("x", 0)),
					"y": int(entry.get("y", 0)),
					"queue": q_clean,
				})
		out["benches"] = benches_clean

	var cols_raw = out.get("colonists", [])
	if cols_raw is Array:
		var cols_clean: Array = []
		for entry in cols_raw:
			if entry is Dictionary:
				var col_dict := {
					"id": int(entry.get("id", 1)),
					"name": str(entry.get("name", "Colono")),
					"role": str(entry.get("role", "Explorador")),
					"is_leader": bool(entry.get("is_leader", false)),
					"x": float(entry.get("x", 32.0)),
					"y": float(entry.get("y", 32.0)),
					"health": int(entry.get("health", 100)),
					"max_health": int(entry.get("max_health", 100)),
					"mood": int(entry.get("mood", 80)),
					"energy": float(entry.get("energy", 92.0)),
					"body_style": int(entry.get("body_style", 0)),
					"hair_style": int(entry.get("hair_style", 0)),
					"skin_color": str(entry.get("skin_color", "#e4b892")),
					"hair_color": str(entry.get("hair_color", "#2c221e")),
					"outfit_color": str(entry.get("outfit_color", "#4f6b47")),
					"helmet_slot": str(entry.get("helmet_slot", "")),
					"armor_slot": str(entry.get("armor_slot", "")),
					"back_tool": str(entry.get("back_tool", "")),
					"main_hand_tool": str(entry.get("main_hand_tool", entry.get("equipped_tool", ""))),
					"equipped_tool": str(entry.get("equipped_tool", "")),
				}
				if entry.has("stats") and entry.get("stats") is Dictionary:
					var stats_clean: Dictionary = {}
					var raw_stats: Dictionary = entry.get("stats")
					for sk in raw_stats.keys():
						stats_clean[str(sk)] = int(raw_stats[sk])
					col_dict["stats"] = stats_clean
				if entry.has("stat_xp") and entry.get("stat_xp") is Array:
					var sxp_clean: Array = []
					for xv in entry.get("stat_xp"):
						sxp_clean.append(float(xv))
					col_dict["stat_xp"] = sxp_clean
				if entry.has("assigned_roles") and entry.get("assigned_roles") is Array:
					var roles_clean: Array = []
					for r_code in entry.get("assigned_roles"):
						roles_clean.append(str(r_code))
					col_dict["assigned_roles"] = roles_clean
				if entry.has("role_levels") and entry.get("role_levels") is Dictionary:
					var lv_clean: Dictionary = {}
					var raw_lv: Dictionary = entry.get("role_levels")
					for k in raw_lv.keys():
						lv_clean[str(k)] = int(raw_lv[k])
					col_dict["role_levels"] = lv_clean
				if entry.has("role_xp") and entry.get("role_xp") is Dictionary:
					var xp_clean: Dictionary = {}
					var raw_xp: Dictionary = entry.get("role_xp")
					for k in raw_xp.keys():
						xp_clean[str(k)] = float(raw_xp[k])
					col_dict["role_xp"] = xp_clean
				cols_clean.append(col_dict)
		out["colonists"] = cols_clean

	return out


static func build_save_payload(state: Dictionary) -> Dictionary:
	return {
		"colony_name": GameSession.colony_name,
		"seed": GameSession.seed_value,
		"biome_code": GameSession.biome_code,
		"map_size": GameSession.map_size,
		"day": int(state.get("day", GameSession.day_count)),
		"colonists": state.get("colonists", []),
		"props": Array(state.get("props", PackedInt32Array())),
		"blocked": Array(state.get("blocked", PackedInt32Array())),
		"zones": Array(state.get("zones", PackedInt32Array())),
		"structures": Array(state.get("structures", PackedInt32Array())),
		"crops": state.get("crops", []),
		"items": state.get("items", []),
		"orders": state.get("orders", []),
		"benches": state.get("benches", []),
	}
