# Roadmap — Colonia Centinela

Hoja de ruta modular para llevar **Colonia Centinela** desde la base técnica actual hasta una simulación completa de colonias terrestres con tecnología avanzada en Android.

---

## Fase 1 — Cimientos, Mundo y Colonos *(Completada)*

- [x] Puente nativo **Godot 4.7.2 ↔ Rust 1.100.0** (`godot = "=0.5.5"`, `api-4-7`) para Android 32 y 64 bits.
- [x] Interfaz cálida multipantalla (Menú Principal con diorama, Nueva Partida, 3 Ranuras de Guardado en JSON y Ajustes persistentes).
- [x] Generador procedural terrestre aleatorio y oculto al jugador con 3 biomas internos (*Bosque Templado*, *Pradera Fértil*, *Sierra Rocosa*).
- [x] Assets vectoriales `.svg` estilo RimWorld (suelos, robles, pinos, arbolitos, rocas de granito/hierro, piedras sueltas y colonos por capas).
- [x] Movimiento autónomo básico y cámara centinela táctil.
- [x] Flujo manual en GitHub Actions para APKs Debug separados y firmados.

---

## Fase 2 — Líder de Colonia, Herramientas de Cobre, Tala, Minería y Almacén *(Completada)*

- [x] **Líder de Colonia aleatorio y reuniones:** En lugar de ejecutar órdenes mágicamente, un líder elegido al azar reúne o se acerca personalmente a los colonos para comunicar las tareas antes de iniciarlas.
- [x] **Catálogo inicial de herramientas de cobre (`CopperTool`):** Hacha de cobre, Pico de cobre, Pala de cobre, Azada de cobre y Espada de cobre en Rust + `.svg` con anclaje en mano y animación de golpe.
- [x] **Órdenes táctiles por selección de área:** Marcar árboles para talar y rocas/vetas para minar mediante arrastre táctil.
- [x] **Recursos sueltos en el piso (`.svg`) y Zona de Almacenamiento:** Troncos de madera, bloques de piedra, mena de hierro, mena de cobre, semillas de zanahoria y zanahorias con transporte automático al almacén.
- [x] **Barra de inventario y estado del líder en el HUD.**

---

## Fase 3 — Construcción Modular, Cultivo de Zanahorias y Navegación A* *(En progreso)*

- [x] **Menú de Construcción y Zonas:** Popup dedicado para elegir **Zona de Almacenamiento**, **Zona de Cultivo (Zanahorias)** y planos de **Pared de Madera** (`2` de madera por bloque) que bloquean el paso.
- [x] **Zona de cultivo agrícola (Zanahorias):** Arado con azada de cobre, siembra de semillas de zanahoria, 4 etapas de crecimiento `.svg` y cosecha con recuperación de semillas.
- [x] **Inventario personal por ranuras en colonos:** Espacios para futuro casco, futura armadura, herramienta en la **espalda** (visible detrás del colono), **mano principal** y carga en mochila, con popup de inspección en el HUD.
- [x] **Energía y sueño en el suelo:** Gasto de energía al trabajar o caminar y descanso en el suelo al fatigarse, con postura acostada y partículas `.svg` `Zzz` animadas.
- [x] **Navegación inteligente (*pathfinding* A* en Rust)** sobre la grilla dinámica al construir muros o despejar árboles/rocas.
- [x] **Sistema de 8 Roles (máx. 5 por colono) y Niveles 1 al 10:** Asignación modular desde el popup **Trabajos** (**Leñador**, **Minero**, **Transportador**, **Constructor**, **Agricultor**, **Botánico**, **Explorador** y **Líder de Colonia**) con progresión gradual y escalado de velocidad, capacidad de carga, rendimiento y alcance.
- [ ] Puertas de madera, suelos construidos, fogatas, camas y mesas de trabajo, además de cascos y armaduras equipables.
- [ ] Necesidades pendientes: hambre (consumo de zanahorias), salud y estado de ánimo con pensamientos terrestres.
- [ ] Ciclo día/noche visual y estaciones terrestres (lluvia, temperatura ambiente, crecimiento de arbolitos).

---

## Fase 4 — Tecnología Terrestre Avanzada

- [ ] Árbol de investigación terrestre (forja de bronce/hierro/acero, electricidad, automatización, energía limpia y defensa).
- [ ] Mejora progresiva desde herramientas de cobre hacia herramientas de hierro/acero y equipo eléctrico.
- [ ] Red eléctrica de la colonia: paneles solares, turbinas eólicas, generadores de biocombustible y baterías.
- [ ] Estaciones de fabricación avanzada y estructuras defensivas terrestres.

---

## Fase 5 — Eventos, Audio y Distribución APK

- [ ] Eventos dinámicos terrestres (caravanas comerciales, brotes de clima extremo, fauna local, incursiones con uso de la espada de cobre).
- [x] Efectos de sonido de talar, picar y construir (`.ogg` CC0) sincronizados desde Rust y conectados a `SettingsManager` *(pendiente: música ambiental acústica)*.
- [ ] Generación de APKs Release optimizados y firmados listos para publicación en Uptodown y descarga directa.
