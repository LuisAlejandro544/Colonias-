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
│   │   │   └── meeting_bubble.svg       # Globo de diálogo cuando el líder reúne y da órdenes
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
│   │   │   └── item_carrot.svg          # Manojo de zanahorias cosechadas
│   │   ├── crops/
│   │   │   ├── soil_tilled.svg          # Tierra arada en surcos lista para sembrar
│   │   │   ├── carrot_stage_1_seeded.svg  # Semillas recién sembradas en el surco
│   │   │   ├── carrot_stage_2_sprout.svg  # Brotes verdes jóvenes
│   │   │   ├── carrot_stage_3_growing.svg # Follaje frondoso en crecimiento
│   │   │   └── carrot_stage_4_ready.svg   # Zanahorias maduras listas para cosechar
│   │   ├── structures/
│   │   │   ├── wall_wood_blueprint.svg  # Plano guía de bloque de pared de madera
│   │   │   └── wall_wood.svg            # Bloque sólido de pared/casa de madera
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
│       │   ├── settings_manager.gd      # Persistencia de ajustes en user://settings.cfg
│       │   ├── save_manager.gd          # Gestión de 3 ranuras JSON en user://saves/
│       │   └── game_session.gd          # Estado activo compartido entre menús y mundo
│       ├── ui/
│       │   ├── main_menu.gd             # Portada con diorama vivo y botones de navegación
│       │   ├── new_game_screen.gd       # Selector de nombre, semilla, bioma, mapa, colonos y ranura
│       │   ├── save_slots_screen.gd     # Tarjetas de ranuras + popup de confirmación al borrar
│       │   ├── settings_screen.gd       # Sliders de audio, escala táctil y sensibilidad de cámara
│       │   └── loading_screen.gd        # Carga asíncrona con progreso hacia el mundo 2D
│       └── world/
│           ├── colony_world.gd          # Orquestador del mapa, props, muros, ítems, cámara y selección táctil
│           ├── ground_canvas.gd         # Lienzo optimizado del suelo terrestre visible en cámara
│           ├── world_overlay.gd         # Capa de zonas, cultivos de zanahoria, marcas y arrastre de selección
│           ├── colonist_visual.gd       # Ensamblado por capas: colono, corona de líder, herramientas y carga
│           └── world_hud.gd             # Barra superior (líder e inventario), selector inferior, inspector y menú
├── rust/
│   ├── Cargo.toml                       # Crate cdylib `colonia_core` (godot =0.5.5, api-4-7)
│   ├── rust-toolchain.toml              # Fija Rust 1.100.0 + targets ARM64 y ARMv7
│   ├── .cargo/config.toml               # Flags de enlazado Android (page-size 16KB en ARM64, NEON en ARMv7)
│   └── src/
│       ├── lib.rs                       # Registro GDExtension y exportación de clases
│       ├── noise.rs                     # RNG determinista SplitMix64 y ruido fractal 2D (FBM)
│       ├── world_gen.rs                 # Generación procedural de piso, árboles, arbolitos, rocas y piedras
│       ├── tools.rs                     # Catálogo de herramientas de cobre (hacha, pico, pala, azada, espada)
│       ├── colony_types.rs              # Modelos puros de ítems, zonas, cultivos, paredes, órdenes e inventario
│       ├── pathfinding.rs               # Pathfinding A* acotado sobre obstáculos y paredes dinámicas
│       └── colonist_sim.rs              # Simulación de colonos, líder aleatorio, reuniones, tareas y guardado
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

- **`WorldGenerator` (`rust/src/world_gen.rs`)**: recibe `(seed, width, height, biome_code)` y devuelve un `VarDictionary` con `PackedInt32Array` para `ground`, `props` y `blocked`.
- **`ColonySimulation` (`rust/src/colonist_sim.rs`)**: recibe las órdenes por área (`designate_area`), gestiona al Líder de Colonia y sus reuniones, avanza cultivos, pathfinding A* y colonos en `step_simulation(delta)` y entrega `get_colonists_snapshot()`, `get_world_snapshot(force_full)` y `export_save_state()` para sincronizar visuales, HUD y JSON en `SaveManager`.
