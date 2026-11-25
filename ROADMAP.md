# Roadmap — Colonia Centinela

Hoja de ruta modular para llevar **Colonia Centinela** desde la base técnica actual hasta una simulación completa de colonias terrestres con tecnología avanzada en Android.

---

## Fase 1 — Cimientos, Mundo y Colonos *(Completada)*

- [x] Puente nativo **Godot 4.7.2 ↔ Rust 1.100.0** (`godot = "=0.5.5"`, `api-4-7`) para Android 32 y 64 bits.
- [x] Interfaz cálida multipantalla (Menú Principal con diorama, Nueva Partida, 3 Ranuras de Guardado en JSON y Ajustes persistentes).
- [x] Generador procedural terrestre por semilla con 3 biomas (*Bosque Templado*, *Pradera Fértil*, *Sierra Rocosa*).
- [x] Assets vectoriales `.svg` estilo RimWorld (suelos, robles, pinos, arbolitos, rocas de granito/hierro, piedras sueltas y colonos por capas).
- [x] Movimiento autónomo básico con evasión de obstáculos y cámara centinela táctil.
- [x] Flujo manual en GitHub Actions para APKs Debug separados y firmados.

---

## Fase 2 — Herramientas, Tala, Minería e Inventario

- [ ] Catálogo de herramientas en Rust (`ToolDef`): hacha de piedra/acero, pico, pala y herramientas avanzadas (motosierra industrial, cortador láser portátil).
- [ ] Anclaje visual de herramienta en mano (`Marker2D`) con animación procedural de golpe/uso y partículas.
- [ ] Sistema de órdenes táctiles sobre árboles y rocas (marcar para talar o picar) y cola de trabajos por prioridades.
- [ ] Recursos recolectables en el suelo (troncos de madera, bloques de piedra, mena de hierro) y transporte a zonas de almacenamiento.
- [ ] Barra de recursos de la colonia en el HUD.

---

## Fase 3 — Construcción, Necesidades y Ciclo Terrestre

- [ ] Sistema de planos y construcción modular (suelos de madera/piedra, muros, puertas, fogatas, camas y mesas de trabajo).
- [ ] Navegación inteligente (*pathfinding* A* / Dijkstra en Rust sobre grilla dinámica actualizada al construir o talar).
- [ ] Necesidades de colonos: energía/sueño, hambre, salud y estado de ánimo con pensamientos terrestres.
- [ ] Ciclo día/noche y estaciones terrestres (lluvia, temperatura ambiente, crecimiento de arbolitos).

---

## Fase 4 — Tecnología Terrestre Avanzada

- [ ] Árbol de investigación terrestre (metalurgia, electricidad, automatización, energía limpia y defensa).
- [ ] Red eléctrica de la colonia: paneles solares, turbinas eólicas, generadores de biocombustible y baterías.
- [ ] Estaciones de fabricación avanzada y recarga para herramientas eléctricas/láser.
- [ ] Estructuras defensivas y torretas automáticas terrestres.

---

## Fase 5 — Eventos, Audio y Distribución APK

- [ ] Eventos dinámicos terrestres (caravanas comerciales, brotes de clima extremo, fauna local, incursiones).
- [ ] Música ambiental acústica y efectos de sonido conectados a los sliders de `SettingsManager`.
- [ ] Generación de APKs Release optimizados y firmados listos para publicación en Uptodown y descarga directa.
