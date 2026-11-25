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
   ├── Llama a WorldGenerator (rust/src/world_gen.rs) -> terreno + obstáculos
   ├── Llama a ColonySimulation (rust/src/colonist_sim.rs) -> posición/estado de colonos
   └── Dibuja suelo SVG, props Y-sorted, capas de ColonistVisual y HUD táctil
```

## 3. Cómo añadir nuevas mecánicas limpiamente

- **Nuevo bioma o recurso natural en el mapa:**
  1. Agrega la variante en `rust/src/world_gen.rs` (`EarthBiome`, `GroundKind` o `PropKind`).
  2. Crea el `.svg` correspondiente en `godot/assets/` (sin pixel art, contorno `#1e1813`) y revísalo renderizándolo a `.png`.
  3. Registra la textura y el mapeo en `godot/scripts/world/colony_world.gd` y `godot/scripts/autoload/save_manager.gd`.
- **Nuevas herramientas, tareas o necesidades de colonos:**
  1. Crea un submódulo dedicado en `rust/src/` (ej. `tools.rs`, `tasks.rs`) y expón métodos limpios desde `ColonySimulation` o una clase `GodotClass` nueva en `rust/src/lib.rs`.
  2. Añade pruebas unitarias `#[cfg(test)]` en Rust para la lógica pura.
  3. Mantén la serialización compatible con JSON en `SaveManager` (`user://saves/slot_N.json`).

## 4. Checklist antes de terminar cualquier cambio

1. Revisar que `rust/Cargo.toml` mantiene `godot = { version = "=0.5.5", features = ["api-4-7"] }`.
2. Verificar que no haya warnings ni código muerto en Rust (`cargo` compila en CI tanto para `aarch64-linux-android`, `armv7-linux-androideabi` y host Linux).
3. Si se crearon o editaron archivos `.svg`, renderizarlos a `.png` con Chromium para verificar visualmente alineación y contornos.
4. Ejecutar `npm run build`, `npm run lint` y los tests de `src/utils/greeting.test.ts` para garantizar que el contenedor web siga sano.
