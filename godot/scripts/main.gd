# Script de arranque de la escena principal en Godot 4.7.2.
#
# Lógica: Conecta la escena 2D inicial con el nodo `ColoniaBootstrap` compilado
# en Rust y registra en consola la versión del núcleo nativo para confirmar
# que el puente GDExtension funciona correctamente en el dispositivo móvil.

extends Node2D

@onready var _core: ColoniaBootstrap = $ColoniaBootstrap


func _ready() -> void:
	if _core != null:
		print("Puente Godot <-> Rust activo. Núcleo: ", _core.get_core_version())
	else:
		push_warning("ColoniaBootstrap no está disponible; verifica los binarios .so de GDExtension.")
