# Reglas para Agentes y Asistentes (`AGENTS.md`)

Cualquier agente o desarrollador que modifique este repositorio debe cumplir estrictamente estas reglas:

1. **No romper la compatibilidad de 32 y 64 bits:**
   - Mantén `godot = { version = "=0.5.5", features = ["api-4-7"] }` salvo que verifiques explícitamente que una versión posterior compila sin errores tanto en `aarch64-linux-android` como en `armv7-linux-androideabi`.
   - Toda lógica determinista en Rust debe evitar suposiciones de tamaño de puntero (`usize` en serialización o hashes).

2. **Separación estricta Rust vs Godot (sin fallbacks engañosos):**
   - La simulación, generación procedural, pathfinding, inventarios y reglas del juego viven en **Rust** (`rust/src/`).
   - La presentación visual, audio, persistencia local en `user://`, cámara táctil e interfaz multipantalla viven en **Godot** (`godot/`).
   - No reemplaces módulos solicitados en Rust con implementaciones falsas o duplicadas en GDScript.

3. **Diseño modular y documentación en cada archivo:**
   - Todo archivo de código nuevo o modificado debe incluir en su cabecera una explicación clara en español de para qué sirve su lógica.
   - Mantén pantallas separadas para apartados distintos (Menú Principal, Nueva Partida, Ranuras, Ajustes, Mundo) y usa popups modales solo para acciones puntuales (confirmar borrado, menú en partida).

4. **Consistencia visual (Estilo RimWorld cálido):**
   - Todos los sprites van en formato `.svg` vectorial limpio con contornos oscuros `#1e1813` y filtro lineal (`default_texture_filter=1`). No introducir pixel art.
   - La UI conserva la paleta cálida definida en `godot/scripts/autoload/ui_theme.gd`. No introducir colores neón, estética cyberpunk ni cambiar iconos/diseño existente sin petición explícita.

5. **Seguridad, licencias y compilación:**
   - Nunca guardes contraseñas ni keystores dentro de `godot/export_presets.cfg` ni los subas al repositorio.
   - No recomiendes ni agregues crates/dependencias con licencias copyleft virales (GPL/AGPL) o que exijan hacer público el juego.
   - Mantén el marcador `"Hello World"` en `src/` compilando (`npm run build` y `npm run lint`) y verifica visualmente cualquier `.svg` nuevo renderizándolo a `.png` antes de darlo por terminado.
