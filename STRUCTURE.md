# Estructura del Proyecto (`STRUCTURE.md`)

Arquitectura modular separada en núcleo nativo (**Rust**), motor de escenas/UI (**Godot 4.7.2**), automatización Android y marcador web mínimo.

```text
.
├── .github/
│   └── workflows/
│       └── android-apk.yml              # CI manual: descarga dependencias + Godot 4.7.2, compila 32/64 bits y firma APKs Debug
├── godot/
│   ├── project.godot                    # Configuración Godot 4.7.2 móvil (Linear, MSAA 4x, ETC2/ASTC, AutoLoads)
│   ├── export_presets.cfg               # Presets separados: Android-arm64-v8a (64 bits) y Android-armeabi-v7a (32 bits)
│   ├── addons/
│   │   └── colonia_core/
│   │       └── colonia_core.gdextension # Mapa de librerías .so para Android (arm64, armeabi-v7a) y Linux anfitrión
│   ├── assets/                          # Gráficos vectoriales .svg estilo RimWorld (sin pixel art)
│   │   ├── colonists/
│   │   │   ├── pawn_shadow.svg          # Sombra ovalada bajo el colono
│   │   │   ├── pawn_body_tunic.svg      # Torso túnica (tinte via modulate)
│   │   │   ├── pawn_body_parka.svg      # Torso abrigo (tinte via modulate)
│   │   │   ├── pawn_head.svg            # Cabeza redonda (piel via modulate)
│   │   │   ├── pawn_face.svg            # Ojos expresivos oscuros
│   │   │   ├── pawn_hair_short.svg      # Peinado corto
│   │   │   ├── pawn_hair_bob.svg        # Peinado melena
│   │   │   ├── pawn_hair_ponytail.svg   # Peinado coleta
│   │   │   ├── leader_crown.svg         # Corona/insignia dorada del Líder de Colonia
│   │   │   ├── meeting_bubble.svg       # Globo de diálogo cuando el líder reúne y da órdenes
│   │   │   └── sleep_zzz.svg            # Partícula vectorial animada "Zzz" al dormir en el suelo
│   │   ├── audio/                       # Efectos de sonido .ogg (Kenney Impact Sounds, CC0 1.0)
│   │   │   ├── chop_01.ogg .. 03.ogg    # Impactos de hacha al talar madera
│   │   │   ├── mine_01.ogg .. 03.ogg    # Golpes de pico sobre roca y vetas minerales
│   │   │   └── build_01.ogg .. 03.ogg   # Golpes de construcción de paredes de madera
│   │   ├── tools/
│   │   │   ├── tool_copper_axe.svg      # Hacha de cobre
│   │   │   ├── tool_copper_pickaxe.svg  # Pico de cobre
│   │   │   ├── tool_copper_shovel.svg   # Pala de cobre
│   │   │   ├── tool_copper_hoe.svg      # Azada de cobre
│   │   │   └── tool_copper_sword.svg    # Espada de cobre
│   │   ├── items/
│   │   │   ├── item_wood_logs.svg       # Troncos de madera sueltos en el piso
│   │   │   ├── item_stone_chunk.svg     # Bloques/trozos de piedra sueltos
│   │   │   ├── item_iron_ore.svg        # Mena de hierro suelta
│   │   │   ├── item_copper_ore.svg      # Mena de cobre suelta
│   │   │   ├── item_carrot_seeds.svg    # Saquito de semillas de zanahoria
│   │   │   ├── item_carrot.svg          # Manojo de zanahorias cosechadas
│   │   │   ├── item_wood_planks.svg     # Tablones de madera procesados
│   │   │   ├── item_wood_bucket.svg     # Cubo de madera artesanal (en piso/almacén)
│   │   │   ├── item_wood_door.svg       # Puerta de madera embalada en el piso antes de instalarse
│   │   │   └── item_wood_crate.svg      # Arcón de madera reforzado
│   │   ├── crops/
│   │   │   ├── soil_tilled.svg          # Tierra arada en surcos lista para sembrar
│   │   │   ├── carrot_stage_1_seeded.svg  # Semillas recién sembradas en el surco
│   │   │   ├── carrot_stage_2_sprout.svg  # Brotes verdes jóvenes
│   │   │   ├── carrot_stage_3_growing.svg # Follaje frondoso en crecimiento
│   │   │   └── carrot_stage_4_ready.svg   # Zanahorias maduras listas para cosechar
│   │   ├── structures/
│   │   │   ├── wall_wood_blueprint.svg        # Plano guía de bloque de pared de madera
│   │   │   ├── wall_wood.svg                  # Bloque sólido de pared/casa de madera
│   │   │   ├── door_wood_blueprint.svg        # Plano guía de puerta de madera
│   │   │   ├── door_wood.svg                  # Puerta de madera instalada entre paredes (transitable)
│   │   │   ├── carpenter_bench_blueprint.svg  # Plano guía de mesa de carpintero
│   │   │   └── carpenter_bench.svg            # Mesa de carpintero instalada con prensa y virutas
│   │   ├── ground/
│   │   │   ├── ground_grass.svg         # Pasto templado
│   │   │   ├── ground_fertile.svg       # Tierra fértil oscura
│   │   │   ├── ground_dirt.svg          # Tierra batida
│   │   │   ├── ground_gravel.svg        # Grava natural
│   │   │   └── ground_stone.svg         # Suelo rocoso
│   │   └── props/
│   │       ├── tree_oak.svg             # Roble maduro con sombra
│   │       ├── tree_pine.svg            # Pino estratificado con sombra
│   │       ├── sapling.svg              # Arbolito joven
│   │       ├── boulder_granite.svg      # Roca grande de granito
│   │       ├── boulder_iron.svg         # Roca con vetas de hierro
│   │       └── stone_pebbles.svg        # Piedras sueltas en el suelo
│   ├── scenes/
│   │   ├── main.tscn                    # Escena bootstrap mínima de prueba
│   │   ├── ui/
│   │   │   ├── main_menu.tscn           # Pantalla 1: Menú Principal
│   │   │   ├── new_game_screen.tscn     # Pantalla 2: Nueva Partida
│   │   │   ├── save_slots_screen.tscn   # Pantalla 3: Ranuras Guardadas
│   │   │   ├── settings_screen.tscn     # Pantalla 4: Ajustes del Juego
│   │   │   └── loading_screen.tscn      # Pantalla de transición y carga del mundo
│   │   └── world/
│   │       └── colony_world.tscn        # Pantalla 5: Mundo 2D en vista centinela
│   └── scripts/
│       ├── main.gd                      # Arranque bootstrap de verificación GDExtension
│       ├── autoload/
│       │   ├── ui_theme.gd              # Paleta cálida Madera/Pergamino y estilos táctiles fijos
│       │   ├── touch_scroll.gd          # Desplazamiento táctil con inercia y filtro anti-toques accidentales
│       │   ├── settings_manager.gd      # Persistencia de ajustes en user://settings.cfg
│       │   ├── save_manager.gd          # Gestión de 3 ranuras JSON en user://saves/
│       │   └── game_session.gd          # Estado activo compartido entre menús y mundo
│       ├── ui/
│       │   ├── main_menu.gd             # Portada con diorama vivo y botones de navegación
│       │   ├── new_game_screen.gd       # Nombre de colonia, cantidad de colonos y ranura (semilla/terreno aleatorios ocultos)
│       │   ├── save_slots_screen.gd     # Tarjetas de ranuras + popup de confirmación al borrar
│       │   ├── settings_screen.gd       # Sliders de audio, escala táctil y sensibilidad de cámara
│       │   └── loading_screen.gd        # Carga asíncrona con progreso hacia el mundo 2D
│       └── world/
│           ├── colony_world.gd          # Orquestador de la escena jugable y enlace con Rust (`WorldGenerator`/`ColonySimulation`)
│           ├── world_camera_input.gd    # Paneo táctil, pinch-zoom, toques y selección rectangular de áreas
│           ├── world_entities_renderer.gd # Props asíncronos, muros/puertas/mesas conectables, ítems sueltos y nodos de colonos
│           ├── world_sfx.gd             # Pool de audio espacial `.ogg` con enfriamiento y volumen de ajustes
│           ├── save_bridge.gd           # Normalización estricta JSON ↔ `gdext` (incluye `stats`, `stat_xp` y `benches`)
│           ├── ground_canvas.gd         # Lienzo optimizado del suelo terrestre visible en cámara
│           ├── world_overlay.gd         # Capa de zonas, cultivos de zanahoria, marcas y arrastre de selección
│           ├── colonist_visual.gd       # Ensamblado por capas: colono, corona, herramienta en mano/espalda, carga y Zzz
│           ├── jobs_popup.gd            # Popup modular de Trabajos: hasta 5 roles entre los 9 disponibles y niveles 1 al 10
│           ├── world_hud.gd             # Coordinador del HUD: barra superior, barra inferior, toasts y popups
│           ├── build_menu_popup.gd      # Popup modal de Construcción y Zonas (almacén, cultivo, pared, puerta, mesa)
│           ├── colonist_inspector_panel.gd # Panel lateral rápido del colono seleccionado (roles, stats, inventario y seguir)
│           ├── colonist_inventory_popup.gd # Popup de ranuras personales del colono (casco, armadura, espalda, mano, carga)
│           ├── colonist_stats_popup.gd  # Popup de atributos del colono (INT, FUE, DES, RES, CAR) y nivel de fabricación
│           ├── carpenter_bench_popup.gd # Popup de cola de recetas y requisitos INT en la Mesa de Carpintero
│           └── pause_menu_popup.gd      # Popup modal del Menú en Partida (guardar, ranuras, ajustes, salir)
├── rust/
│   ├── Cargo.toml                       # Crate cdylib `colonia_core` (godot =0.5.5, api-4-7)
│   ├── rust-toolchain.toml              # Fija Rust 1.100.0 + targets ARM64 y ARMv7
│   ├── .cargo/config.toml               # Flags de enlazado Android (page-size 16KB en ARM64, NEON en ARMv7)
│   └── src/
│       ├── lib.rs                       # Registro GDExtension y exportación de clases
│       ├── noise.rs                     # RNG determinista SplitMix64 y ruido fractal 2D (FBM)
│       ├── world_gen.rs                 # Generación procedural de piso, árboles, arbolitos, rocas y piedras
│       ├── tools.rs                     # Catálogo de herramientas de cobre (hacha, pico, pala, azada, espada)
│       ├── stats.rs                     # Atributos INT, FUE, DES, RES, CAR (1..=20), XP por práctica y puertas por complejidad
│       ├── roles.rs                     # Los 9 roles (incl. Carpintero), límite de 5 roles por colono y perfil de habilidades
│       ├── roles/
│       │   └── balance.rs               # Curva de XP (niveles 1..=10), bonificadores reales y resúmenes en español
│       ├── colony_types.rs              # Modelos de ítems, zonas, cultivos, paredes/puertas/mesas, órdenes e inventario
│       ├── pathfinding.rs               # Pathfinding A* acotado sobre obstáculos y paredes dinámicas
│       ├── colonist_sim.rs              # Puente GDExtension `ColonySimulation` hacia Godot (incl. API de Mesa de Carpintero)
│       └── colonist_sim/
│           ├── core.rs                  # Estado `SimulationCore`, inicialización, bucle `step` y tests de integración
│           ├── designations.rs          # Designación/cancelación rectangular, cola de carpintería, almacén e inventario
│           ├── leader.rs                # Líder de Colonia, reuniones de órdenes y alta/baja de roles
│           ├── farming.rs               # Maduración de zanahorias y tareas de Agricultor y Botánico
│           ├── tasks.rs                 # Asignación de tareas, rutas A* y ejecución de tala, minería, construcción, carpintería y acarreo
│           └── persistence.rs           # Serialización/restauración de snapshots `VarDictionary` para guardado JSON
├── scripts/
│   └── build_android.sh                 # Compila Rust (ARM64 + ARMv7 + host Linux) y exporta APKs en Godot
├── src/                                 # Marcador web mínimo ("Hello World") para el entorno de vista previa
│   ├── App.tsx
│   ├── main.tsx
│   ├── index.css
│   └── utils/
│       ├── greeting.ts
│       └── greeting.test.ts
├── README.md                            # Documentación general
├── ROADMAP.md                           # Plan de fases
├── STRUCTURE.md                         # Este archivo
├── AI_CONTEXT.md                        # Contexto técnico e invariantes
├── AGENTS.md                            # Reglas de desarrollo para agentes
└── README_AI.md                         # Guía rápida para asistentes IA
```

## Frontera de datos Rust ↔ Godot

- **`WorldGenerator` (`rust/src/world_gen.rs`)**: recibe `(seed, width, height, biome_code)` generados aleatoriamente de forma interna y devuelve un `VarDictionary` con `PackedInt32Array` para `ground`, `props` y `blocked`.
- **`ColonySimulation` (`rust/src/colonist_sim.rs`)**: recibe las órdenes por área (`designate_area`), gestiona los 9 roles y niveles `1..=10` (`get_colonist_roles_detail`, `set_colonist_role_enabled`), los 5 atributos `INT`/`FUE`/`DES`/`RES`/`CAR`, las colas de fabricación de `CarpenterBench` (`get_carpentry_bench_at`, `queue_carpentry_recipe`, `cancel_carpentry_queue_item`), el Líder de Colonia y sus reuniones, cultivos, energía/sueño, ranuras de inventario (`ColonistInventory`), pathfinding A* y colonos en `step_simulation(delta)`, entregando `get_colonists_snapshot()`, `get_world_snapshot(force_full)` (incluyendo `benches` y `sfx_events`) y `export_save_state()` para sincronizar visuales, HUD y JSON en `SaveManager`.
