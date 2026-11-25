# Colonia Centinela

Juego 2D móvil en vista centinela sobre gestión y supervivencia de colonias humanas en la Tierra, con progresión desde herramientas básicas hasta tecnología avanzada. Construido con **Godot Engine 4.7.2** para renderizado/interfaz táctil y **Rust 1.100.0** (`gdext`) como núcleo nativo de simulación y generación procedural.

> **Nota sobre el entorno web:** La carpeta `src/` contiene únicamente un marcador mínimo (`"Hello World"`) para mantener compilando la vista previa web de desarrollo. El juego real está pensado exclusivamente para **Android (APK)** en `godot/` y `rust/`.

---

## Características implementadas (v0.1.0)

- **Menú principal y navegación multipantalla:** Portada cálida con diorama vivo, pantallas independientes para **Nueva Partida**, **Ranuras de Partidas Guardadas** (3 ranuras JSON con confirmación de borrado) y **Ajustes del Juego** persistentes en `user://settings.cfg`.
- **Estética ilustrada tipo RimWorld (sin pixel art):** Recursos vectoriales `.svg` con filtro lineal y MSAA 4x, contornos gruesos y paleta orgánica de madera y pergamino.
- **Generación procedural terrestre en Rust:** Ruido determinista por semilla (`SplitMix64` + FBM) que crea biomas de la Tierra (*Bosque Templado*, *Pradera Fértil*, *Sierra Rocosa*) con pasto, tierra fértil, tierra batida, grava, piedra, robles, pinos, arbolitos, rocas de granito, vetas de hierro y piedras sueltas.
- **Colonos autónomos por capas:** Peones compuestos por sombra, túnica/abrigo, cabeza, rostro y peinados independientes que recorren el mapa evitando obstáculos con cámara centinela táctil (arrastre, zoom con pellizco/botones e inspector inferior al tocar un colono).
- **Compilación Android de 32 y 64 bits:** Exportación separada de APKs Debug firmados para `arm64-v8a` y `armeabi-v7a` mediante GitHub Actions.

---

## Stack técnico

| Componente | Versión / Configuración |
| :--- | :--- |
| Motor de juego | Godot Engine `4.7.2-stable` (renderizador `Mobile`, ETC2/ASTC) |
| Lenguaje de núcleo | Rust `1.100.0` (Edition `2024`) |
| Binding GDExtension | `godot` (`gdext`) `=0.5.5` con feature `api-4-7` |
| Scripts de escena/UI | GDScript (Godot 4.7) |
| Distribución objetivo | Android APK directo (32 bits `armeabi-v7a` y 64 bits `arm64-v8a`, SDK mínimo 24) |

---

## Compilación de APKs Debug firmados (GitHub Actions)

El flujo `.github/workflows/android-apk.yml` se ejecuta **únicamente de forma manual** (`workflow_dispatch`) y genera dos APKs separados y firmados (`colonia_centinela-arm64-v8a-debug.apk` y `colonia_centinela-armeabi-v7a-debug.apk`).

### Secretos obligatorios en GitHub

En **Settings → Secrets and variables → Actions** configura estos 4 secretos:

| Secreto | Descripción |
| :--- | :--- |
| `ANDROID_KEYSTORE_BASE64` | Archivo `.keystore` o `.jks` codificado en Base64 en una sola línea (`base64 -w 0 firma.jks`). |
| `ANDROID_KEYSTORE_PASSWORD` | Contraseña del archivo keystore. |
| `ANDROID_KEY_ALIAS` | Alias de la clave dentro del keystore. |
| `ANDROID_KEY_PASSWORD` | Contraseña de la clave/alias. |

### Compilación local (Linux + Android NDK)

```bash
export ANDROID_NDK_HOME=/ruta/a/android-ndk
BUILD_MODE=debug ./scripts/build_android.sh
```

---

## Documentación del proyecto

- [`STRUCTURE.md`](./STRUCTURE.md) — Árbol completo de archivos y responsabilidades de cada módulo.
- [`ROADMAP.md`](./ROADMAP.md) — Hoja de ruta de desarrollo por fases.
- [`AI_CONTEXT.md`](./AI_CONTEXT.md) — Decisiones técnicas, restricciones y contexto clave del proyecto.
- [`AGENTS.md`](./AGENTS.md) — Reglas obligatorias para agentes e IA que editen este repositorio.
- [`README_AI.md`](./README_AI.md) — Guía rápida de arranque para asistentes de programación.
