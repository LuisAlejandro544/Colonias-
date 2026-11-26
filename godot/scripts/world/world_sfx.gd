# Módulo de audio espacial/ambiente de la colonia (pool de efectos SFX).
#
# Lógica: Precarga los sonidos `.ogg` (CC0) de tala, minería y construcción,
# administra un pool de `AudioStreamPlayer` para evitar cortes y aplica
# el volumen maestro/ambiente de `SettingsManager` más enfriamientos por evento.

class_name WorldSfx
extends Node

var _streams: Dictionary = {}
var _players: Array[AudioStreamPlayer] = []
var _cooldowns: Dictionary = {}
var _rng := RandomNumberGenerator.new()


func _ready() -> void:
	_preload_streams()
	_setup_pool()


func _preload_streams() -> void:
	_streams = {
		"chop": [
			load("res://assets/audio/chop_01.ogg") as AudioStream,
			load("res://assets/audio/chop_02.ogg") as AudioStream,
			load("res://assets/audio/chop_03.ogg") as AudioStream,
		],
		"mine": [
			load("res://assets/audio/mine_01.ogg") as AudioStream,
			load("res://assets/audio/mine_02.ogg") as AudioStream,
			load("res://assets/audio/mine_03.ogg") as AudioStream,
		],
		"build": [
			load("res://assets/audio/build_01.ogg") as AudioStream,
			load("res://assets/audio/build_02.ogg") as AudioStream,
			load("res://assets/audio/build_03.ogg") as AudioStream,
		],
	}


func _setup_pool() -> void:
	_rng.randomize()
	for i in range(8):
		var player := AudioStreamPlayer.new()
		player.max_polyphony = 1
		add_child(player)
		_players.append(player)


func play_events(events: Variant) -> void:
	if events is Array:
		for ev in events:
			play_event(str(ev))


func play_event(event_name: String) -> void:
	if not _streams.has(event_name) or _players.is_empty():
		return
	var master_gain := clampf(SettingsManager.master_volume / 100.0, 0.0, 1.0)
	var ambient_gain := clampf(SettingsManager.ambient_volume / 100.0, 0.0, 1.0)
	var gain := master_gain * ambient_gain
	if gain <= 0.01:
		return

	var now_sec := Time.get_ticks_msec() * 0.001
	var last_sec := float(_cooldowns.get(event_name, -10.0))
	if now_sec - last_sec < 0.14:
		return
	_cooldowns[event_name] = now_sec

	var variations: Array = _streams[event_name]
	if variations.is_empty():
		return
	var chosen: AudioStream = variations[_rng.randi_range(0, variations.size() - 1)]
	if chosen == null:
		return

	for p in _players:
		if not p.playing:
			p.stream = chosen
			p.volume_db = linear_to_db(gain * 0.72)
			p.pitch_scale = _rng.randf_range(0.92, 1.08)
			p.play()
			return
