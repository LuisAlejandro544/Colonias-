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
│   │   ├── animals/                     # Fauna terrestre libre por bioma
│   │   │   ├── animal_pig.svg           # Cerdito
│   │   │   ├── animal_chicken.svg       # Pollo
│   │   │   ├── animal_sheep.svg         # Oveja
│   │   │   └── animal_cow.svg           # Vaca
│   │   ├── colonists/
│   │   │   ├── pawn_shadow.svg          # Sombra ovalada bajo el colono y fauna
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
│   │   │   ├── item_coal.svg            # Carbón mineral suelto
│   │   │   ├── item_copper_ingot.svg    # Lingote de cobre fundido en el Horno de Piedra y Ladrillo
│   │   │   ├── item_stone_bricks.svg    # Ladrillos de piedra tallados en la Mesa de Masón
│   │   │   ├── item_clay_bricks.svg     # Ladrillos de arcilla cocida (diseño reservado para futura mecánica de arcilla)
│   │   │   ├── item_stone_mortar.svg    # Mortero de piedra tallado
│   │   │   ├── item_stone_millstone.svg # Piedra de molino tallada
│   │   │   ├── item_stone_statuette.svg # Estatuilla ornamental de piedra
│   │   │   ├── item_carrot_seeds.svg    # Saquito de semillas de zanahoria
│   │   │   ├── item_carrot.svg          # Manojo de zanahorias cosechadas
│   │   │   ├── item_potato_seeds.svg    # Saquito de semillas de papa
│   │   │   ├── item_potato.svg          # Papas cosechadas
│   │   │   ├── item_wheat_seeds.svg     # Saquito de semillas de trigo
│   │   │   ├── item_wheat.svg           # Gavilla de trigo cosechada
│   │   │   ├── item_beet_seeds.svg      # Saquito de semillas de remolacha
│   │   │   ├── item_beet.svg            # Remolachas cosechadas
│   │   │   ├── item_raw_chicken_breast.svg  # Pechuga de pollo cruda
│   │   │   ├── item_raw_chicken_thigh.svg   # Muslo de pollo crudo
│   │   │   ├── item_raw_pork_belly.svg      # Panceta de cerdo cruda
│   │   │   ├── item_raw_pork_ribs.svg       # Costillar de cerdo crudo
│   │   │   ├── item_raw_mutton_shoulder.svg # Paletilla de oveja cruda
│   │   │   ├── item_raw_mutton_leg.svg      # Pierna de oveja cruda
│   │   │   ├── item_raw_beef_sirloin.svg    # Solomillo de res crudo
│   │   │   ├── item_raw_beef_brisket.svg    # Pecho de res crudo
│   │   │   ├── item_animal_bones.svg        # Huesos de animal (para futura sopa de huesos)
│   │   │   ├── item_wood_planks.svg     # Tablones de madera procesados
│   │   │   ├── item_wood_bucket.svg     # Cubo de madera artesanal (pozos y riego de cultivos)
│   │   │   ├── item_wood_cup_empty.svg  # Vaso de madera vacío (suelo e inventario)
│   │   │   ├── item_wood_cup.svg        # Vaso de madera con agua (al beber)
│   │   │   ├── item_wood_door.svg       # Puerta de madera embalada en el piso antes de instalarse
│   │   │   └── item_wood_crate.svg      # Arcón de madera reforzado
│   │   ├── crops/
│   │   │   ├── soil_tilled.svg              # Tierra arada en surcos lista para sembrar
│   │   │   ├── carrot_stage_1_seeded.svg .. 4_ready.svg # Etapas de zanahoria
│   │   │   ├── potato_stage_1_seeded.svg .. 4_ready.svg # Etapas de papa
│   │   │   ├── wheat_stage_1_seeded.svg .. 4_ready.svg  # Etapas de trigo
│   │   │   └── beet_stage_1_seeded.svg .. 4_ready.svg   # Etapas de remolacha
│   │   ├── structures/
│   │   │   ├── wall_wood_blueprint.svg .. wall_wood_front.svg # Muro de madera conectable
│   │   │   ├── door_wood_blueprint.svg / door_wood.svg        # Puerta de madera transitable
│   │   │   ├── carpenter_bench_blueprint.svg / carpenter_bench.svg # Mesa de Carpintero
│   │   │   ├── water_well_blueprint.svg / water_well.svg      # Pozo de Agua profunda
│   │   │   ├── mason_bench_blueprint.svg / mason_bench.svg    # Mesa de Masón
│   │   │   ├── stone_furnace_blueprint.svg / stone_furnace.svg # Horno de Piedra y Ladrillo
│   │   │   ├── wall_stone_blueprint.svg .. wall_stone_front.svg # Pared de Piedra y Ladrillo conectable
│   │   │   ├── floor_stone_blueprint.svg / floor_stone.svg    # Suelo de Ladrillos (+18% velocidad)
│   │   │   └── stone_brazier_blueprint.svg / stone_brazier.svg # Brasero de Piedra y Ladrillo
│   │   ├── ground/
│   │   │   ├── ground_grass.svg         # Pasto templado
│   │   │   ├── ground_fertile.svg       # Tierra fértil oscura
│   │   │   ├── ground_dirt.svg          # Tierra batida
│   │   │   ├── ground_gravel.svg        # Grava natural
│   │   │   ├── ground_stone.svg         # Suelo rocoso
│   │   │   └── ground_river.svg         # Cauce continuo de río natural (agua corriente)
│   │   └── props/
│   │       ├── tree_oak.svg             # Roble maduro con sombra
│   │       ├── tree_pine.svg            # Pino estratificado con sombra
│   │       ├── sapling.svg              # Arbolito joven
│   │       ├── boulder_granite.svg      # Roca grande de granito
│   │       ├── boulder_iron.svg         # Roca con vetas de hierro
│   │       ├── boulder_copper.svg       # Roca con vetas de cobre
│   │       ├── boulder_coal.svg         # Roca con vetas de carbón mineral
│   │       ├── stone_pebbles.svg        # Piedras sueltas en el suelo
│   │       ├── mountain_stone.svg / _crown.svg / _front.svg   # Macizos de piedra conectables
│   │       ├── mountain_copper.svg / _crown.svg / _front.svg  # Vetas de cobre en montaña conectable
│   │       └── mountain_coal.svg / _crown.svg / _front.svg    # Vetas de carbón en montaña conectable
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
│           ├── world_entities_renderer.gd # Montañas conectables, props asíncronos, muros/suelos/mesas/hornos/braseros/pozos, fauna, ítems y colonos
│           ├── world_sfx.gd             # Pool de audio espacial `.ogg` con enfriamiento y volumen de ajustes
│           ├── save_bridge.gd           # Normalización estricta JSON ↔ `gdext` (`stats`, `hydration`, `watered_timer`, `benches`, `workshops`, `animals`, `crop_kind`, `target_id`)
│           ├── ground_canvas.gd         # Lienzo optimizado del suelo terrestre y ríos visible en cámara
│           ├── world_overlay.gd         # Capa de zonas, cultivos con indicador de riego húmedo, marcas y arrastre de selección
│           ├── colonist_visual.gd       # Ensamblado por capas: colono, corona, herramienta, vaso lleno al beber, cubo al regar y Zzz
│           ├── jobs_popup.gd            # Popup modular de Trabajos: hasta 5 roles entre los 10 disponibles y niveles 1 al 10
│           ├── world_hud.gd             # Coordinador del HUD: barra superior (incl. ladrillos, lingotes y morteros), barra inferior y popups
│           ├── build_menu_popup.gd      # Popup modal de Construcción y Zonas (almacén, 4 cultivos, muros, suelo, pozo, brasero, mesas, horno)
│           ├── colonist_inspector_panel.gd # Panel lateral rápido del colono seleccionado (roles, energía, hidratación, stats, seguir)
│           ├── colonist_inventory_popup.gd # Popup de ranuras personales del colono (casco, armadura, espalda, mano, carga)
│           ├── colonist_stats_popup.gd  # Popup de atributos del colono (INT, FUE, DES, RES, CAR) y nivel de fabricación
│           ├── carpenter_bench_popup.gd # Popup de cola de recetas para Mesa de Carpintero, Mesa de Masón y Horno de Piedra y Ladrillo
│           └── pause_menu_popup.gd      # Popup modal del Menú en Partida (guardar, ranuras, ajustes, salir)
├── rust/
│   ├── Cargo.toml                       # Crate cdylib `colonia_core` (godot =0.5.5, api-4-7)
│   ├── rust-toolchain.toml              # Fija Rust 1.100.0 + targets ARM64 y ARMv7
│   ├── .cargo/config.toml               # Flags de enlazado Android (page-size 16KB en ARM64, NEON en ARMv7)
│   └── src/
│       ├── lib.rs                       # Registro GDExtension y exportación de clases
│       ├── animals.rs                   # Fauna terrestre por bioma (cerdo, pollo, oveja, vaca), cortes de carne y huesos
│       ├── noise.rs                     # RNG determinista SplitMix64 y ruido fractal 2D (FBM)
│       ├── world_gen.rs                 # Generación procedural de piso, ríos con vados, árboles, rocas y montañas conectables
│       ├── tools.rs                     # Catálogo de herramientas de cobre (hacha, pico, pala, azada, espada)
│       ├── stats.rs                     # Atributos INT, FUE, DES, RES, CAR (1..=20), XP por práctica y puertas por complejidad
│       ├── roles.rs                     # Los 10 roles (incl. Carpintero y Masón), límite de 5 roles por colono y perfil de habilidades
│       ├── roles/
│       │   └── balance.rs               # Curva de XP (niveles 1..=10), bonificadores reales (incl. mampostería) y resúmenes en español
│       ├── colony_types.rs              # Modelos de ítems, talleres (`WorkshopKind`), recetas (`MasonryRecipe`), riego, órdenes e inventario
│       ├── pathfinding.rs               # Pathfinding A* acotado sobre obstáculos, montañas, ríos y paredes dinámicas
│       ├── colonist_sim.rs              # Puente GDExtension `ColonySimulation` hacia Godot (incl. `get_workshop_at`)
│       └── colonist_sim/
│           ├── core.rs                  # Estado `SimulationCore`, desgaste de sed/energía, Temporada de Cultivos, fauna y tests
│           ├── designations.rs          # Designación rectangular (incl. nuevas estructuras y colas de talleres/hornos) e inventario
│           ├── leader.rs                # Líder de Colonia, reuniones de órdenes y alta/baja de roles
│           ├── farming.rs               # Maduración estacional, riego con cubo (`watered_timer`) y tareas de Agricultor/Botánico
│           ├── tasks.rs                 # Asignación de tareas (beber, regar, minar montañas, masón/horno, caza, suelo +18%), A* y ejecución
│           └── persistence.rs           # Serialización/restauración de snapshots `VarDictionary` (incl. `workshops`, `watered_timer` e ítems nuevos)
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

- **`WorldGenerator` (`rust/src/world_gen.rs`)**: recibe `(seed, width, height, biome_code)` generados aleatoriamente de forma interna y devuelve un `VarDictionary` con `PackedInt32Array` para `ground` (`GroundKind::River` con vados), `props` (incluyendo bloques de montaña conectables `9, 10, 11`) y `blocked`.
- **`ColonySimulation` (`rust/src/colonist_sim.rs`)**: recibe las órdenes por área (`designate_area`), gestiona sed/hidratación, riego con cubo, Temporada de Cultivos, fauna terrestre, 10 roles (`1..=10`), 5 atributos, colas de `CarpenterBench` / `MasonBench` / `StoneFurnace` (`get_workshop_at`), suelos pavimentados (`+18%` velocidad), Líder de Colonia, A* y colonos en `step_simulation(delta)`.
