# Capa de entidades ordenadas por profundidad Y (props, muros, ítems y colonos).
#
# Lógica: Administra la instanciación progresiva de árboles/rocas, la sincronización
# de planos y paredes de madera conectables por vecinos (`_WallTileVisual`), los
# montones de recursos sueltos en el piso y los nodos visuales de cada colono
# (`ColonistVisual`).

class_name WorldEntitiesRenderer
extends Node2D

const TILE_PX := 64.0
const PROPS_PER_FRAME := 240

const ColonistVisualScript := preload("res://scripts/world/colonist_visual.gd")

var _prop_sprites: Dictionary = {}
var _structure_sprites: Dictionary = {}
var _item_nodes: Dictionary = {}
var _colonist_nodes: Dictionary = {}

var _structure_textures: Dictionary = {}
var _item_textures: Dictionary = {}


func _ready() -> void:
	y_sort_enabled = true
	preload_textures()


func preload_textures() -> void:
	_structure_textures = {
		1: load("res://assets/structures/wall_wood_blueprint.svg") as Texture2D,
		2: load("res://assets/structures/wall_wood.svg") as Texture2D,
		3: load("res://assets/structures/door_wood_blueprint.svg") as Texture2D,
		4: load("res://assets/structures/door_wood.svg") as Texture2D,
		5: load("res://assets/structures/carpenter_bench_blueprint.svg") as Texture2D,
		6: load("res://assets/structures/carpenter_bench.svg") as Texture2D,
		201: load("res://assets/structures/wall_wood_crown.svg") as Texture2D,
		202: load("res://assets/structures/wall_wood_front.svg") as Texture2D,
	}
	_item_textures = {
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


func spawn_world_props_async(
	map_width: int,
	map_height: int,
	props: PackedInt32Array,
	seed_value: int,
	progress_cb: Callable
) -> void:
	var tex_oak := load("res://assets/props/tree_oak.svg") as Texture2D
	var tex_pine := load("res://assets/props/tree_pine.svg") as Texture2D
	var tex_sapling := load("res://assets/props/sapling.svg") as Texture2D
	var tex_granite := load("res://assets/props/boulder_granite.svg") as Texture2D
	var tex_iron := load("res://assets/props/boulder_iron.svg") as Texture2D
	var tex_stones := load("res://assets/props/stone_pebbles.svg") as Texture2D

	var rng := RandomNumberGenerator.new()
	rng.seed = seed_value ^ 0x4455

	var spawned_in_batch := 0
	var total_tiles := map_width * map_height

	for y in range(map_height):
		for x in range(map_width):
			var idx := y * map_width + x
			var p := props[idx]
			if p == 0:
				continue

			var sprite := Sprite2D.new()
			var base_pos := Vector2((x + 0.5) * TILE_PX, (y + 0.5) * TILE_PX)
			var jitter := Vector2(rng.randf_range(-8.0, 8.0), rng.randf_range(-6.0, 6.0))
			sprite.position = base_pos + jitter

			match p:
				1:
					sprite.texture = tex_oak
					var s := rng.randf_range(0.56, 0.72)
					sprite.scale = Vector2(s, s)
					sprite.offset = Vector2(0, -34)
				2:
					sprite.texture = tex_pine
					var s := rng.randf_range(0.58, 0.74)
					sprite.scale = Vector2(s, s)
					sprite.offset = Vector2(0, -38)
				3:
					sprite.texture = tex_sapling
					var s := rng.randf_range(0.42, 0.52)
					sprite.scale = Vector2(s, s)
					sprite.offset = Vector2(0, -18)
				4:
					sprite.texture = tex_granite
					var s := rng.randf_range(0.46, 0.58)
					sprite.scale = Vector2(s, s)
					sprite.flip_h = rng.randf() < 0.5
					sprite.offset = Vector2(0, -14)
				5:
					sprite.texture = tex_iron
					var s := rng.randf_range(0.46, 0.58)
					sprite.scale = Vector2(s, s)
					sprite.flip_h = rng.randf() < 0.5
					sprite.offset = Vector2(0, -14)
				6:
					sprite.texture = tex_stones
					var s := rng.randf_range(0.44, 0.56)
					sprite.scale = Vector2(s, s)
					sprite.flip_h = rng.randf() < 0.5

			add_child(sprite)
			_prop_sprites[idx] = sprite
			spawned_in_batch += 1
			if spawned_in_batch >= PROPS_PER_FRAME:
				spawned_in_batch = 0
				var pct := 45.0 + (float(idx) / maxf(1.0, float(total_tiles))) * 47.0
				if progress_cb.is_valid():
					progress_cb.call(pct)
				await get_tree().process_frame


func sync_world_entities(map_width: int, map_height: int, snap: Dictionary) -> void:
	# 1. Props (eliminar árboles talados o rocas picadas)
	var props: PackedInt32Array = snap.get("props", PackedInt32Array())
	if not props.is_empty():
		var to_remove: Array[int] = []
		for idx in _prop_sprites.keys():
			if idx < props.size() and props[idx] == 0:
				var sp := _prop_sprites[idx] as Node
				if is_instance_valid(sp):
					sp.queue_free()
				to_remove.append(idx)
		for idx in to_remove:
			_prop_sprites.erase(idx)

	# 2. Estructuras (planos y paredes de madera conectables por vecinos)
	var structures: PackedInt32Array = snap.get("structures", PackedInt32Array())
	if not structures.is_empty():
		for idx in range(structures.size()):
			var st := structures[idx]
			if st == 0:
				if _structure_sprites.has(idx):
					var old_node := _structure_sprites[idx] as Node
					if is_instance_valid(old_node):
						old_node.queue_free()
					_structure_sprites.erase(idx)
			else:
				var node: _WallTileVisual
				if _structure_sprites.has(idx):
					node = _structure_sprites[idx] as _WallTileVisual
				else:
					node = _WallTileVisual.new()
					var tx := idx % map_width
					var ty := int(idx / map_width)
					node.position = Vector2((tx + 0.5) * TILE_PX, (ty + 0.5) * TILE_PX)
					add_child(node)
					_structure_sprites[idx] = node

				if st == 1 or st == 3 or st == 4 or st == 5 or st == 6:
					node.configure(_structure_textures.get(st, null), true, true, true, true, false)
				else:
					var tx := idx % map_width
					var ty := int(idx / map_width)
					var n_st := structures[(ty - 1) * map_width + tx] if ty > 0 else 0
					var s_st := structures[(ty + 1) * map_width + tx] if ty + 1 < map_height else 0
					var w_st := structures[ty * map_width + (tx - 1)] if tx > 0 else 0
					var e_st := structures[ty * map_width + (tx + 1)] if tx + 1 < map_width else 0
					var has_n := (n_st == 2 or n_st == 4)
					var has_s := (s_st == 2 or s_st == 4)
					var has_w := (w_st == 2 or w_st == 4)
					var has_e := (e_st == 2 or e_st == 4)
					var isolated := not has_n and not has_s and not has_w and not has_e
					var tex: Texture2D
					if isolated:
						tex = _structure_textures.get(2, null)
					elif has_s:
						tex = _structure_textures.get(201, null)
					else:
						tex = _structure_textures.get(202, null)
					node.configure(tex, has_n, has_s, has_w, has_e, not isolated)

	# 3. Recursos sueltos en el piso
	var items_arr: Array = snap.get("items", [])
	var seen_ids: Dictionary = {}
	for it_var in items_arr:
		if not (it_var is Dictionary):
			continue
		var it: Dictionary = it_var
		var iid := int(it.get("id", 0))
		seen_ids[iid] = true
		var node: Node2D
		if _item_nodes.has(iid):
			node = _item_nodes[iid] as Node2D
		else:
			node = _create_ground_item_node()
			add_child(node)
			_item_nodes[iid] = node
		_update_ground_item_node(node, it)

	var dead_items: Array[int] = []
	for iid in _item_nodes.keys():
		if not seen_ids.has(iid):
			var n := _item_nodes[iid] as Node
			if is_instance_valid(n):
				n.queue_free()
			dead_items.append(iid)
	for iid in dead_items:
		_item_nodes.erase(iid)


func _create_ground_item_node() -> Node2D:
	var root := Node2D.new()
	var sp := Sprite2D.new()
	sp.name = "Icon"
	sp.scale = Vector2(0.34, 0.34)
	root.add_child(sp)

	var lbl := Label.new()
	lbl.name = "Qty"
	lbl.position = Vector2(4, 4)
	lbl.add_theme_color_override("font_color", Color("#fff5dc"))
	lbl.add_theme_color_override("font_outline_color", Color("#1b1510"))
	lbl.add_theme_constant_override("outline_size", 4)
	lbl.add_theme_font_size_override("font_size", 14)
	root.add_child(lbl)
	return root


func _update_ground_item_node(node: Node2D, it: Dictionary) -> void:
	var ix := int(it.get("x", 0))
	var iy := int(it.get("y", 0))
	node.position = Vector2((ix + 0.5) * TILE_PX, (iy + 0.54) * TILE_PX)
	var kind := str(it.get("kind", "wood_logs"))
	var amount := int(it.get("amount", 1))
	var sp := node.get_node("Icon") as Sprite2D
	if sp:
		sp.texture = _item_textures.get(kind, null)
	var lbl := node.get_node("Qty") as Label
	if lbl:
		lbl.text = "x%d" % amount


func sync_colonists(snapshots: Array) -> void:
	for snap in snapshots:
		if not (snap is Dictionary):
			continue
		var cid := int(snap.get("id", 0))
		var node: Node2D
		if _colonist_nodes.has(cid):
			node = _colonist_nodes[cid]
			node.call("apply_snapshot", snap)
		else:
			node = ColonistVisualScript.new() as Node2D
			node.call("setup_from_snapshot", snap)
			add_child(node)
			_colonist_nodes[cid] = node


func find_colonist_at_world_pos(world_pos: Vector2, max_dist: float = 54.0) -> int:
	var best_id := -1
	var best_dist := max_dist
	for cid in _colonist_nodes.keys():
		var c := _colonist_nodes[cid] as Node2D
		var d := (c.position + Vector2(0, -16)).distance_to(world_pos)
		if d < best_dist:
			best_dist = d
			best_id = cid
	return best_id


func set_selected_colonist(selected_id: int) -> Node2D:
	for cid in _colonist_nodes.keys():
		(_colonist_nodes[cid] as Node2D).call("set_selected", cid == selected_id)
	return get_colonist_node(selected_id)


func get_colonist_node(colonist_id: int) -> Node2D:
	if _colonist_nodes.has(colonist_id):
		return _colonist_nodes[colonist_id] as Node2D
	return null


func get_colonist_nodes_array() -> Array:
	return _colonist_nodes.values()


func get_colonist_count() -> int:
	return _colonist_nodes.size()


class _WallTileVisual extends Node2D:
	var _sprite: Sprite2D
	var _border: _WallBorderDrawer

	func _init() -> void:
		_sprite = Sprite2D.new()
		_sprite.scale = Vector2(0.50, 0.50)
		add_child(_sprite)
		_border = _WallBorderDrawer.new()
		add_child(_border)

	func configure(
		tex: Texture2D,
		has_n: bool,
		has_s: bool,
		has_w: bool,
		has_e: bool,
		draw_borders: bool
	) -> void:
		_sprite.texture = tex
		_border.visible = draw_borders
		_border.set_edges(not has_n, not has_s, not has_w, not has_e)


class _WallBorderDrawer extends Node2D:
	var _n: bool = false
	var _s: bool = false
	var _w: bool = false
	var _e: bool = false

	func set_edges(n: bool, s: bool, w: bool, e: bool) -> void:
		if _n == n and _s == s and _w == w and _e == e:
			return
		_n = n
		_s = s
		_w = w
		_e = e
		queue_redraw()

	func _draw() -> void:
		var outline := Color("#1e1813")
		var w := 2.4
		if _n:
			draw_line(Vector2(-32.0, -31.0), Vector2(32.0, -31.0), outline, w)
		if _s:
			draw_line(Vector2(-32.0, 31.0), Vector2(32.0, 31.0), outline, w)
		if _w:
			draw_line(Vector2(-31.0, -32.0), Vector2(-31.0, 32.0), outline, w)
		if _e:
			draw_line(Vector2(31.0, -32.0), Vector2(31.0, 32.0), outline, w)
