# Colonia Centinela

Juego 2D móvil en vista centinela sobre gestión y supervivencia de colonias humanas en la Tierra, con progresión desde herramientas básicas de cobre hasta tecnología avanzada. Construido con **Godot Engine 4.7.2** para renderizado/interfaz táctil y **Rust 1.100.0** (`gdext`) como núcleo nativo de simulación, pathfinding y generación procedural.

> **Nota sobre el entorno web:** La carpeta `src/` contiene únicamente un marcador mínimo (`"Hello World"`) para mantener compilando la vista previa web de desarrollo. El juego real está pensado exclusivamente para **Android (APK)** en `godot/` y `rust/`.

---

## Características implementadas (v0.8.0)

- **Bloques Naturales de Montaña Conectables (Piedra, Cobre y Carbón):** Además de las rocas sueltas, el generador procedural forma macizos montañosos cuadrados conectables casilla a casilla (`mountain_stone*.svg`, `mountain_copper*.svg` y `mountain_coal*.svg`). Al minarlos con la orden **Minar**, entregan bloques de piedra, menas de cobre y carbón mineral, actualizando dinámicamente las caras y contornos de las casillas vecinas.
- **El Masón (`Masón`), Ladrillos de piedra, Horno y Objetos de Piedra y Ladrillo:** Décimo rol jugable (`ColonyRole::Mason`) con progresión de nivel `1..=10` y cola de recetas por Inteligencia (`INT`):
  - **Mesa de Masón** (`6` piedras): labra **Ladrillos de piedra** (`2 piedra → 4 ladrillos`, sprite `item_stone_bricks.svg` en piedra tallada; además se conserva `item_clay_bricks.svg` reservado para futura arcilla), **Mortero de piedra** (`3 piedra → 1 mortero`, `INT 2+`), **Piedra de molino** (`4 piedra + 2 ladrillos → 1 molino`, `INT 3+`) y **Estatuilla de piedra** (`4 ladrillos + 1 piedra → 1 estatuilla`, `INT 4+`).
  - **Horno de Piedra y Ladrillo** (`4` piedras + `6` ladrillos): funde **Lingotes de cobre** (`2 menas de cobre + 1 carbón → 1 lingote`, `INT 2+`).
  - **Estructuras de Mampostería:** **Pared de Piedra y Ladrillo** conectable (`1 piedra + 2 ladrillos`), **Suelo de Ladrillos** transitable (`2 ladrillos`, otorga `+18%` velocidad al caminar encima) y **Brasero de Piedra y Ladrillo** (`2 piedra + 3 ladrillos + 1 carbón`, iluminación cálida).
- **Riego de Cultivos con Cubos de Madera:** Los **Botánicos** llenan un **Cubo de madera** en el **Pozo de Agua** (o en la orilla del río) sin consumirlo y riegan parcelas `3×3` (`watered_timer`), mostrando suelo húmedo y gota de riego. Regar compensa las siembras **Fuera de Temporada** (`1.60x`) y potencia aún más el crecimiento en **Temporada de Cultivos** (`1.65x × 1.45x`).
- **Sistema de Sed, Ríos, Vasos Vacíos vs Llenos y Pozos de Agua:** Cada colono gestiona su propia **hidratación**. Los vasos fabricados o en el piso usan su sprite vacío (`item_wood_cup_empty.svg`), reservando `item_wood_cup.svg` (lleno de agua) para cuando un colono bebe activamente. Cuando tiene sed acude primero al **Pozo de Agua** (`1 cubo + 4 troncos + 6 piedras`) o, si no hay pozo, al **río** (`ground_river.svg`, con vados de grava).
- **Fauna terrestre por bioma, Caza, Cortes de Carne y Huesos:** Cerditos, pollos, ovejas y vacas según bioma, cacería con espada de cobre, cortes específicos de carne cruda y huesos de animal.
- **Temporada de Cultivos y 4 especies agrícolas:** Ciclo de **6 días** (días 1–3 en **Temporada de Cultivos**, días 4–6 **Fuera de Temporada**) para **Zanahorias**, **Papas**, **Trigo** y **Remolacha**.
- **Atributos de Colonos (`INT`, `FUE`, `DES`, `RES`, `CAR`) y Carpintería:** 5 estadísticas del **1 al 20** (`INT` desbloquea recetas avanzadas en Carpintero, Masón y Horno).
- **Líder de Colonia, 10 Roles (máx. 5 por colono), Audio CC0 y Guardado JSON:** Reuniones de órdenes del líder, popup **Trabajos** con 10 roles, descanso en el suelo con `Zzz`, audio `.ogg` CC0 y exportación Android APK 32/64 bits.

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
