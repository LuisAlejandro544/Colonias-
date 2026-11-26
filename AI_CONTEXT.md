# Contexto Técnico del Proyecto (`AI_CONTEXT.md`)

Este documento resume las decisiones arquitectónicas, versiones fijadas, restricciones del usuario y soluciones a errores reales ya resueltos para evitar regresiones.

---

## 1. Visión del producto y restricciones del usuario

- **Juego:** Colonia Centinela — simulación de colonias 2D en vista centinela (estilo *RimWorld*), ambientada **exclusivamente en la Tierra**, con progresión desde herramientas básicas de cobre hacia tecnología terrestre avanzada.
- **Líder de Colonia y cadena de mando:** Un colono aleatorio actúa como Líder de Colonia. Las órdenes marcadas por el jugador (talar, minar, almacén, cultivo, construcción) pasan por una reunión o trato personal del líder con los colonos antes de ejecutarse.
- **Generación aleatoria oculta:** La semilla, el bioma y el tamaño del terreno se generan aleatoriamente en `new_game_screen.gd` y se persisten únicamente de forma interna en el JSON de guardado. **Nunca** deben mostrarse en la interfaz de usuario (ni en Nueva Partida, Carga, Ranuras ni HUD).
- **Plataforma:** **Juego móvil Android** (NO web). Los APKs se distribuyen de forma directa (terceros / Uptodown), no orientados a Google Play `.aab`.
- **Entorno del creador:** Trabaja desde un teléfono móvil (sin PC), por lo que depende de la compilación automatizada en GitHub Actions.
- **Entorno web (`src/`):** Debe permanecer únicamente como un `"Hello World"` limpio que compile con `vite build` y `tsc --noEmit`.
- **Estilo visual:** Vectorial ilustrado suavizado estilo RimWorld (`.svg` con filtro lineal y MSAA 4x). **Prohibido pixel art** y **prohibida estética cyberpunk/neón/futurista** o minimalismo soso. La interfaz usa una paleta cálida de madera, latón y pergamino.

---

## 2. Decisiones técnicas críticas e invariantes

### A. Rust `1.100.0` + `godot = "=0.5.5"` (`api-4-7`)
- **Por qué está fijado `=0.5.5`:** En `godot-core 0.5.6` se introdujo `src/builtin/variant/rust_variant.rs` con `#[repr(C, align(8))]` fijo, lo cual provoca el error de compilación `E0080` (`static_assert_eq_size_align!(RustVariant, OpaqueVariant)`) al compilar para arquitecturas Android de 32 bits (`armv7-linux-androideabi`, donde `OpaqueVariant` tiene `align(4)`).
- La versión `=0.5.5` incluye soporte completo de `api-4-7` y compila limpiamente en **64 bits (`aarch64-linux-android`)**, **32 bits (`armv7-linux-androideabi`)** y **Linux x86_64**.

### B. Interoperabilidad JSON (Godot 4) ↔ Rust (`gdext`)
- `JSON.parse()` en GDScript interpreta los números JSON como `float`. Al pasar diccionarios restaurados a `ColonySimulation.restore_colony`, `colony_world.gd` normaliza explícitamente los campos enteros con `int(...)` (incluyendo `role_levels`), decimales con `float(...)` (`role_xp`, `tend_cooldown`), listas de códigos (`assigned_roles`) y arreglos a `PackedInt32Array` para evitar fallos de conversión estricta (`try_to::<i32>()`) en `gdext`.

### C. Godot `4.7.2-stable` y exportación Android
- `godot/project.godot` exige `textures/vram_compression/import_etc2_astc=true` para exportar texturas en Android.
- `godot/export_presets.cfg` usa `gradle_build/use_gradle_build=false` y `gradle_build/export_format=0` (APK directo). Por ello **no** deben incluirse `gradle_build/min_sdk` ni `gradle_build/target_sdk` en `export_presets.cfg` (Godot los rechaza si `use_gradle_build` está en `false`).
- Existen dos presets separados (`Android-arm64-v8a` y `Android-armeabi-v7a`) para generar APKs individuales por arquitectura y excluir `bin/linux/*`.

### D. GitHub Actions (`.github/workflows/android-apk.yml`)
- **Activación manual obligatoria:** Solo `workflow_dispatch`.
- **Sin `android-actions/setup-android`:** Esa acción intenta instalar el paquete obsoleto `tools` eliminado por Google (`Failed to find package 'tools'`). El workflow usa directamente el `sdkmanager` preinstalado en el runner Ubuntu.
- **Validación estricta de firma:** Exige los 4 secretos (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`) al inicio y firma/verifica ambos APKs con `zipalign` + `apksigner`.

### E. Licencias, audio y derechos de autor
- Usar únicamente nombres originales (sin marcas registradas).
- No incorporar dependencias ni assets que obliguen a abrir el código fuente del proyecto o imponer atribuciones restrictivas (no GPL/AGPL).
- Los efectos de sonido en `godot/assets/audio/*.ogg` provienen de *Kenney Impact Sounds* bajo licencia **CC0 1.0 Universal (Dominio Público)**, disparados mediante `sfx_events` desde `ColonySimulation` hacia un pool de `AudioStreamPlayer` en `colony_world.gd`.

### F. Inventario de colonos y sueño en el suelo
- Cada colono usa `ColonistInventory` (`rust/src/colony_types.rs`) con `helmet_slot`, `armor_slot`, `back_tool` y `main_hand` (con intercambio automático al equipar o guardar herramientas) además de energía (`energy`) para acostarse en el suelo (`ColonistActivity::Sleeping`) con partículas `.svg` `sleep_zzz.svg`.

### G. Sistema de 10 Roles (máx. 5 por colono) y Niveles `1..=10`
- `rust/src/roles.rs` define los 10 roles independientes (`Leñador`, `Minero`, `Transportador`, `Constructor`, `Agricultor`, `Botánico`, `Explorador`, `Líder de Colonia`, `Carpintero` y `Masón`), limitados a un máximo de `5` roles asignados simultáneamente por colono (`MAX_ASSIGNED_ROLES = 5`).
- **Agricultor vs Botánico:** Se mantienen como oficios distintos; el **Agricultor** ara tierra nueva (`CropStage::None -> Tilled`) y cosecha cultivos maduros (`CropStage::Ready`, con mayor rendimiento por nivel), mientras el **Botánico** siembra semillas (`CropStage::Tilled -> Seeded`), cuida cultivos en crecimiento y riega parcelas con cubo (`WaterCrops`).
- **Progresión de Nivel 1 a 10:** La experiencia requerida aumenta progresivamente (`25 -> 800` XP) con desaceleración por nivel alto y fatiga (`energy < 35`), escalando velocidades reales de trabajo/caminata, capacidad de carga de `Transportador` (`2` a `16` uds/viaje), rendimiento de recursos, carpintería, mampostería/fundición, radio de `Explorador` y rapidez de reuniones del `Líder de Colonia`. La colonia conserva siempre 1 líder activo al transferir o quitar roles.

### H. Atributos de Colono (`INT`, `FUE`, `DES`, `RES`, `CAR`) y Carpintería
- `rust/src/stats.rs` define `ColonistStats` con valores `1..=20` y `stat_xp` por práctica para **Inteligencia (`INT`)**, **Fuerza (`FUE`)**, **Destreza (`DES`)**, **Resistencia (`RES`)** y **Carisma (`CAR`)**.
- **`INT` como puerta de complejidad:** Cada receta de fabricación declara un `required_intel`; un `Carpintero` solo puede tomar pedidos cuyo `required_intel <= stats.intelligence` (`Tablones de madera` `INT 1+`, `Cubo de madera` `INT 2+`, `Puerta de madera` `INT 3+`, `Arcón de madera` `INT 4+`). Al fundar una colonia se garantiza al menos un colono inicial con `INT >= 4`.
- **Puerta de madera (ítem vs instalada):** La receta fabrica `ItemKind::WoodDoorItem` (`item_wood_door.svg`) en el piso/almacén; luego el plano `StructureKind::WoodDoorBlueprint` consume `1` puerta fabricada para instalar `StructureKind::WoodDoor` (`door_wood.svg`), manteniendo `blocked = 0` para permitir el paso libre y conectando visualmente sus bordes con `WoodWall`.
- **Retrocompatibilidad de guardados:** `save_bridge.gd` y `colonist_sim/persistence.rs` normalizan `stats`, `stat_xp`, `max_health`, `benches`, `crop_kind`, `target_id` y `animals`, reconstruyendo valores deterministas y fauna inicial al cargar partidas anteriores.

### I. Fauna por bioma, Minerales superficiales, Temporada de Cultivos y Nuevos Cultivos
- **Fauna terrestre (`rust/src/animals.rs`):** `Pig`, `Chicken`, `Sheep` y `Cow` aparecen según `biome_code` interno (`temperate_forest`, `fertile_meadow`, `rocky_highlands`). Con el modo `"hunt"` (**Cazar**), `Scout` o `ColonyLeader` persiguen dinámicamente al animal marcado con `CopperSword` y generan cortes anatómicos específicos (`RawChickenBreast`, `RawChickenThigh`, `RawPorkBelly`, `RawPorkRibs`, `RawMuttonShoulder`, `RawMuttonLeg`, `RawBeefSirloin`, `RawBeefBrisket`) más `AnimalBones` (reservados para futura sopa de huesos).
- **Cobre y Carbón en rocas y montañas (`world_gen.rs` + `tasks.rs`):** `PropKind::CopperBoulder` (`7`) y `PropKind::CoalBoulder` (`8`) se generan en superficie junto a `GraniteBoulder` e `IronBoulder`, además de bloques naturales de montaña conectables casilla a casilla (`MountainStone = 9`, `MountainCopper = 10`, `MountainCoal = 11`). `ItemKind::CopperIngot` ahora se funde en el **Horno de Piedra y Ladrillo**.
- **Temporada de Cultivos y especies (`CropKind`):** Ciclo de 6 días (`((day_count - 1) % 6) < 3`). En temporada: 100% disposición agrícola, `1.15x` velocidad de trabajo y `1.65x` velocidad de crecimiento; fuera de temporada: aún se puede sembrar y labrar, pero Agricultor y Botánico tienen baja motivación (~20% determinista) y `0.72x` velocidad. Soporta `Carrot`, `Potato`, `Wheat` y `Beet` con etapas `.svg` propias.

### J. Sistema de Sed, Ríos, Vasos Vacíos vs Llenos, Riego y Pozos de Agua
- **Hidratación de colonos (`Colonist.hydration`):** Cada colono pierde hidratación con el tiempo (más rápido trabajando, moderado caminando y lento descansando/durmiendo). Con `hydration <= 36.0` busca beber entre tareas; con `hydration <= 18.0` interrumpe su actividad actual mediante `try_interrupt_for_drink`. Persistida en JSON (`hydration`, defecto `90.0`).
- **Ríos con vados (`GroundKind::River = 5`):** `world_gen.rs` talla un río sinuoso determinista fuera del claro inicial (`radio 7.2`) con `blocked = 1`, orillas fértiles y `2` vados de `GroundKind::Gravel` (`blocked = 0`). Si no existe ningún pozo (`WaterWell`), el colono bebe del río (`DrinkingRiver`), perdiendo ánimo (`-9` sin vaso, `-4` con vaso).
- **Vasos de madera (`ItemKind::WoodCup = 28`):** Fabricables en la **Mesa de Carpintero** (`1 WoodPlank -> 2 WoodCup`, `INT 1+`). En el piso/inventario usan `item_wood_cup_empty.svg` (vaso vacío) y cuando el colono bebe (`is_drinking`) muestra `item_wood_cup.svg` (lleno de agua).
- **Pozos de Agua (`StructureKind::WaterWell = 8`) y Riego con Cubos:** El pozo (`1 WoodBucket + 4 WoodLogs + 6 StoneChunk`) provee agua limpia prioritaria. Además, el **Botánico** usa un `WoodBucket` (sin consumirlo, llenando en el pozo primero o en el río) para regar parcelas `3×3` (`watered_timer = 65.0`): eleva el crecimiento fuera de temporada a `1.60x` y añade `1.45x` sobre el `1.65x` de temporada.

### K. El Masón (`ColonyRole::Mason`), Horno de Piedra y Ladrillo y Mampostería
- **Mesa de Masón (`StructureKind::MasonBench = 10`, modo `"build_mason_bench"`, `6 StoneChunk`):** El **Masón** fabrica **Ladrillos de piedra** (`2 StoneChunk -> 4 StoneBricks`, `INT 1+`, sprite `item_stone_bricks.svg` en piedra gris-cálida labrada; el diseño previo de ladrillos de arcilla terracota se conserva en `item_clay_bricks.svg` para futura arcilla), **Mortero de piedra** (`3 StoneChunk -> 1 StoneMortar`, `INT 2+`), **Piedra de molino** (`4 StoneChunk + 2 StoneBricks -> 1 StoneMillstone`, `INT 3+`) y **Estatuilla de piedra** (`4 StoneBricks + 1 StoneChunk -> 1 StoneStatuette`, `INT 4+`).
- **Horno de Piedra y Ladrillo (`StructureKind::StoneFurnace = 12`, modo `"build_furnace"`, `4 StoneChunk + 6 StoneBricks`):** Funde **Lingote de cobre** (`2 CopperOre + 1 Coal -> 1 CopperIngot`, `INT 2+`).
- **Estructuras de Piedra y Ladrillo:** **Pared de Piedra y Ladrillo** (`14`, conectable con muros y puertas, `1 piedra + 2 ladrillos`), **Suelo de Ladrillos** (`16`, transitable con `+18%` velocidad al caminar, `2 ladrillos`) y **Brasero de Piedra y Ladrillo** (`18`, `2 piedra + 3 ladrillos + 1 carbón`, iluminación cálida). La UI comparte `CarpenterBenchPopup` vía `get_workshop_at`.
