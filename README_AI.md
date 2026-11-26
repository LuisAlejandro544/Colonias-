# Guía Rápida para Asistentes IA (`README_AI.md`)

Si acabas de abrir este repositorio como asistente técnico, lee esto primero para ubicarte en menos de un minuto.

---

## 1. Qué es este repo (y qué NO es)

- **El juego real está en `/rust` + `/godot`:** es un juego móvil 2D para Android en vista centinela sobre colonias terrestres llamado **Colonia Centinela**.
- **`/src` NO es el juego:** es solo un `"Hello World"` en React/Vite requerido para que el entorno de vista previa web compile sin fallos. No añadas mecánicas del juego en `/src` salvo que el usuario lo pida expresamente.

## 2. Cómo fluye la arquitectura

```text
Menús Godot (scripts/ui/*.gd)
   │  configuran GameSession y leen/escriben SaveManager / SettingsManager
   ▼
Escena del Mundo (godot/scripts/world/colony_world.gd)
   ├── Llama a WorldGenerator (rust/src/world_gen.rs) -> terreno + obstáculos (semilla/bioma aleatorios ocultos)
   ├── Llama a ColonySimulation (rust/src/colonist_sim.rs + roles.rs + stats.rs) -> 9 roles (máx. 5), niveles 1..=10, 5 atributos (INT/FUE/DES/RES/CAR), líder, reuniones, A*, cultivos, muros/puertas/mesas, carpintería, ítems, energía/sueño, inventario y sfx_events
   ├── WorldOverlay (world_overlay.gd) -> zonas, etapas de zanahorias, marcas y selección táctil
   ├── WorldHud (world_hud.gd) + popups -> barra superior, selector inferior, menú de construcción, Trabajos, Atributos, Inventario y Mesa de Carpintero
   └── SaveBridge (save_bridge.gd) + renderizadores -> suelo SVG, props/muros/puertas/mesas/ítems Y-sorted, ColonistVisual y SFX .ogg
```

## 3. Cómo añadir nuevas mecánicas limpiamente

- **Nuevo bioma, cultivo o estructura:**
  1. Agrega las variantes en `rust/src/world_gen.rs` o `rust/src/colony_types.rs` y la lógica en `rust/src/colonist_sim/`.
  2. Crea los `.svg` correspondientes en `godot/assets/` (sin pixel art, contorno `#1e1813`) y revísalos renderizándolos a `.png`.
  3. Registra las texturas y el mapeo en `godot/scripts/world/world_entities_renderer.gd`, `world_hud.gd` o `world_overlay.gd` y mantén la conversión de guardado en `save_bridge.gd` (incluyendo `assigned_roles`, `role_levels`, `role_xp`, `stats`, `stat_xp`, `max_health`, `benches` y `tend_cooldown`, sin exponer semilla ni parámetros de terreno al jugador).
- **Nuevas herramientas, roles, atributos, recetas, armaduras/cascos o necesidades:**
  1. Extiende `rust/src/tools.rs`, `rust/src/roles.rs`, `rust/src/stats.rs`, `ColonistInventory` en `rust/src/colony_types.rs` o crea un submódulo dedicado en `rust/src/` y expón métodos limpios en `ColonySimulation`.
  2. Añade pruebas unitarias `#[cfg(test)]` en Rust para la lógica pura.
  3. Mantén la serialización compatible con JSON en `SaveManager` (`user://saves/slot_N.json`), convirtiendo enteros y floats parseados por `JSON.parse()` en `save_bridge.gd` antes de pasarlos a `gdext`.

## 4. Checklist antes de terminar cualquier cambio

1. Revisar que `rust/Cargo.toml` mantiene `godot = { version = "=0.5.5", features = ["api-4-7"] }`.
2. Verificar que no haya warnings ni código muerto en Rust (`cargo` compila en CI tanto para `aarch64-linux-android`, `armv7-linux-androideabi` y host Linux).
3. Si se crearon o editaron archivos `.svg`, renderizarlos a `.png` con Chromium para verificar visualmente alineación y contornos.
4. Ejecutar `npm run build`, `npm run lint` y los tests de `src/utils/greeting.test.ts` para garantizar que el contenedor web siga sano.
