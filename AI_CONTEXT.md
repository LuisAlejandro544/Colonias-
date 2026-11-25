# Contexto Técnico del Proyecto (`AI_CONTEXT.md`)

Este documento resume las decisiones arquitectónicas, versiones fijadas, restricciones del usuario y soluciones a errores reales ya resueltos para evitar regresiones.

---

## 1. Visión del producto y restricciones del usuario

- **Juego:** Colonia Centinela — simulación de colonias 2D en vista centinela (estilo *RimWorld*), ambientada **exclusivamente en la Tierra**, con progresión desde herramientas básicas hacia tecnología terrestre avanzada.
- **Plataforma:** **Juego móvil Android** (NO web). Los APKs se distribuyen de forma directa (terceros / Uptodown), no orientados a Google Play `.aab`.
- **Entorno del creador:** Trabaja desde un teléfono móvil (sin PC), por lo que depende de la compilación automatizada en GitHub Actions.
- **Entorno web (`src/`):** Debe permanecer únicamente como un `"Hello World"` limpio que compile con `vite build` y `tsc --noEmit`.
- **Estilo visual:** Vectorial ilustrado suavizado estilo RimWorld (`.svg` con filtro lineal y MSAA 4x). **Prohibido pixel art** y **prohibida estética cyberpunk/neón/futurista** o minimalismo soso. La interfaz usa una paleta cálida de madera, latón y pergamino.

---

## 2. Decisiones técnicas críticas e invariantes

### A. Rust `1.100.0` + `godot = "=0.5.5"` (`api-4-7`)
- **Por qué está fijado `=0.5.5`:** En `godot-core 0.5.6` se introdujo `src/builtin/variant/rust_variant.rs` con `#[repr(C, align(8))]` fijo, lo cual provoca el error de compilación `E0080` (`static_assert_eq_size_align!(RustVariant, OpaqueVariant)`) al compilar para arquitecturas Android de 32 bits (`armv7-linux-androideabi`, donde `OpaqueVariant` tiene `align(4)`).
- La versión `=0.5.5` incluye soporte completo de `api-4-7` y compila limpiamente en **64 bits (`aarch64-linux-android`)**, **32 bits (`armv7-linux-androideabi`)** y **Linux x86_64**.

### B. Godot `4.7.2-stable` y exportación Android
- `godot/project.godot` exige `textures/vram_compression/import_etc2_astc=true` para exportar texturas en Android.
- `godot/export_presets.cfg` usa `gradle_build/use_gradle_build=false` y `gradle_build/export_format=0` (APK directo). Por ello **no** deben incluirse `gradle_build/min_sdk` ni `gradle_build/target_sdk` en `export_presets.cfg` (Godot los rechaza si `use_gradle_build` está en `false`).
- Existen dos presets separados (`Android-arm64-v8a` y `Android-armeabi-v7a`) para generar APKs individuales por arquitectura y excluir `bin/linux/*`.

### C. GitHub Actions (`.github/workflows/android-apk.yml`)
- **Activación manual obligatoria:** Solo `workflow_dispatch`.
- **Sin `android-actions/setup-android`:** Esa acción intenta instalar el paquete obsoleto `tools` eliminado por Google (`Failed to find package 'tools'`). El workflow usa directamente el `sdkmanager` preinstalado en el runner Ubuntu.
- **Validación estricta de firma:** Exige los 4 secretos (`ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`) al inicio y firma/verifica ambos APKs con `zipalign` + `apksigner`.

### D. Licencias y derechos de autor
- Usar únicamente nombres originales (sin marcas registradas).
- No incorporar dependencias ni assets que obliguen a abrir el código fuente del proyecto o imponer atribuciones restrictivas (no GPL/AGPL).
