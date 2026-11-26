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

### G. Sistema de 9 Roles (máx. 5 por colono) y Niveles `1..=10`
- `rust/src/roles.rs` define los 9 roles independientes (`Leñador`, `Minero`, `Transportador`, `Constructor`, `Agricultor`, `Botánico`, `Explorador`, `Líder de Colonia` y `Carpintero`), limitados a un máximo de `5` roles asignados simultáneamente por colono (`MAX_ASSIGNED_ROLES = 5`).
- **Agricultor vs Botánico:** Se mantienen como oficios distintos; el **Agricultor** ara tierra nueva (`CropStage::None -> Tilled`) y cosecha zanahorias maduras (`CropStage::Ready`, con mayor rendimiento por nivel), mientras el **Botánico** siembra semillas (`CropStage::Tilled -> Seeded`) y cuida cultivos en crecimiento (impulsando `crop.progress` según su nivel).
- **Progresión de Nivel 1 a 10:** La experiencia requerida aumenta progresivamente (`25 -> 800` XP) con desaceleración por nivel alto y fatiga (`energy < 35`), escalando velocidades reales de trabajo/caminata, capacidad de carga de `Transportador` (`2` a `16` uds/viaje), rendimiento de recursos, velocidad de carpintería, radio de `Explorador` y rapidez de reuniones del `Líder de Colonia`. La colonia conserva siempre 1 líder activo al transferir o quitar roles.

### H. Atributos de Colono (`INT`, `FUE`, `DES`, `RES`, `CAR`) y Carpintería
- `rust/src/stats.rs` define `ColonistStats` con valores `1..=20` y `stat_xp` por práctica para **Inteligencia (`INT`)**, **Fuerza (`FUE`)**, **Destreza (`DES`)**, **Resistencia (`RES`)** y **Carisma (`CAR`)**.
- **`INT` como puerta de complejidad:** Cada receta de fabricación declara un `required_intel`; un `Carpintero` solo puede tomar pedidos cuyo `required_intel <= stats.intelligence` (`Tablones de madera` `INT 1+`, `Cubo de madera` `INT 2+`, `Puerta de madera` `INT 3+`, `Arcón de madera` `INT 4+`). Al fundar una colonia se garantiza al menos un colono inicial con `INT >= 4`.
- **Puerta de madera (ítem vs instalada):** La receta fabrica `ItemKind::WoodDoorItem` (`item_wood_door.svg`) en el piso/almacén; luego el plano `StructureKind::WoodDoorBlueprint` consume `1` puerta fabricada para instalar `StructureKind::WoodDoor` (`door_wood.svg`), manteniendo `blocked = 0` para permitir el paso libre y conectando visualmente sus bordes con `WoodWall`.
- **Retrocompatibilidad de guardados:** `save_bridge.gd` y `colonist_sim/persistence.rs` normalizan `stats`, `stat_xp`, `max_health` y `benches`, reconstruyendo valores deterministas y mesas faltantes al cargar partidas anteriores.
