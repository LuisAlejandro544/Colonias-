# Colonia Centinela

Juego 2D móvil en vista centinela sobre gestión y supervivencia de colonias humanas en la Tierra, con progresión desde herramientas básicas de cobre hasta tecnología avanzada. Construido con **Godot Engine 4.7.2** para renderizado/interfaz táctil y **Rust 1.100.0** (`gdext`) como núcleo nativo de simulación, pathfinding y generación procedural.

> **Nota sobre el entorno web:** La carpeta `src/` contiene únicamente un marcador mínimo (`"Hello World"`) para mantener compilando la vista previa web de desarrollo. El juego real está pensado exclusivamente para **Android (APK)** en `godot/` y `rust/`.

---

## Características implementadas (v0.3.0)

- **Líder de Colonia y reuniones de órdenes:** Al fundar una colonia se elige aleatoriamente a un **Líder de Colonia** (distinguido con corona dorada y espada de cobre). Las órdenes marcadas por el jugador no se ejecutan de forma mágica: el líder reúne o se acerca personalmente a los colonos para comunicar las instrucciones antes de que trabajen.
- **Menú de Construcción y Zonas:** Panel dedicado accesible desde el botón **Construir** en el HUD inferior para elegir entre **Zona de Almacenamiento**, **Zona de Cultivo (Zanahorias)** y planos de **Pared de Madera** (`2` troncos por bloque, bloqueando el paso y obligando al pathfinding A* a rodearlas).
- **Inventario personal y ranuras de colono:** Cada colono tiene ranuras propias en Rust y un panel visual de **Inventario** en el HUD con espacios reservados para **Casco (futuro)** y **Armadura (futura)**, ranura visible en la **Espalda** (pico, azada, hacha, pala o espada), **Mano principal** activa y **Mochila / carga actual**.
- **Sueño en el suelo y animación `Zzz` (`.svg`):** Los colonos gastan energía al trabajar o caminar y se acuestan en el suelo al agotarse, mostrando partículas vectoriales `.svg` animadas de `Zzz` hasta recuperar energía.
- **Tala, minería, cultivo de zanahorias y recursos en el piso:** Selección táctil por área para talar árboles, minar rocas de granito o hierro, arar tierra fértil, sembrar semillas de zanahoria (4 etapas visuales `.svg`), cosechar y almacenar automáticamente.
- **Generación aleatoria oculta del mundo:** La semilla, el bioma terrestre y las dimensiones del terreno se generan de forma aleatoria e invisible al crear una colonia, sin mostrarse en menús, pantalla de carga, ranuras ni HUD.
- **Efectos de sonido de trabajo (CC0):** Sonidos `.ogg` reales para talar, picar y construir sincronizados con cada golpe de herramienta en Rust y regulados por los ajustes de volumen.
- **Menú principal, multipantalla y guardado completo:** Portada con diorama vivo, pantallas independientes para **Nueva Partida**, **Ranuras de Partidas Guardadas** (3 ranuras JSON) y **Ajustes del Juego** persistentes en `user://settings.cfg`.
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
