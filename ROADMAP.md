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

## Fase 3 — Construcción Modular, Cultivos Estacionales, Hidratación y Fauna *(En progreso)*

- [x] **Bloques naturales de montaña conectables:** Macizos de piedra, cobre y carbón conectables casilla a casilla (`MountainStone`, `MountainCopper`, `MountainCoal`), minables y con actualización dinámica de caras al excavar.
- [x] **Sistema de Sed, Ríos, Vasos Vacíos/Llenos y Pozos de Agua:** Ríos con vados de grava, rechazo a beber agua cruda en cubos, sprite propio para vaso vacío (`item_wood_cup_empty.svg`) vs lleno (`item_wood_cup.svg`) y **Pozos de Agua** (`1 cubo + 4 troncos + 6 piedras`).
- [x] **Riego de cultivos con Cubos de Madera:** El Botánico llena un cubo en el pozo o río sin consumirlo y riega parcelas `3×3` (`1.60x` fuera de temporada, `+1.45x` adicional en temporada).
- [x] **Rol de El Masón (`Masón`), Horno de Piedra y Ladrillo y Mampostería:** Décimo rol con Mesa de Masón (**Ladrillos de piedra** con `item_stone_bricks.svg`, **Mortero**, **Piedra de molino**, **Estatuilla**), **Horno de Piedra y Ladrillo** (fundición de **Lingotes de cobre** con cobre y carbón), **Pared de Piedra y Ladrillo** conectable, **Suelo de Ladrillos** (`+18%` velocidad) y **Brasero de Piedra y Ladrillo** *(diseño de ladrillos de arcilla conservado en `item_clay_bricks.svg`)*.
- [x] **Menú de Construcción y Zonas:** Popup dedicado para almacén, 4 cultivos, muros (madera y piedra/ladrillo), puerta, suelo de ladrillos, pozo, brasero, Mesa de Carpintero, Mesa de Masón y Horno.
- [x] **Cultivos múltiples, Temporada de Cultivos y Fauna:** Zanahoria, papa, trigo y remolacha en ciclo de 6 días; cerditos, pollos, ovejas y vacas por bioma con caza y huesos.
- [x] **Inventario personal, Atributos (`INT`..`CAR`), Carpintería y 10 Roles (máx. 5, niveles 1 al 10).**
- [ ] Extracción de arcilla en orillas de río y cocción de **Ladrillos de arcilla** (`item_clay_bricks.svg`) en el horno.
- [ ] Fogatas/cocina (hervir agua en cubos, sopa de huesos y carnes cocidas), camas, cascos y armaduras equipables.
- [ ] Necesidades pendientes: hambre (consumo de cosechas y carnes cocidas), daño en combate y estado de ánimo con pensamientos terrestres.
- [ ] Ciclo día/noche visual y clima terrestre (lluvia, temperatura ambiente, crecimiento de arbolitos).

---

## Fase 4 — Tecnología Terrestre Avanzada

- [ ] Árbol de investigación terrestre (forja de bronce/hierro/acero, electricidad, automatización, energía limpia y defensa).
- [ ] Mejora progresiva desde herramientas de cobre hacia herramientas de hierro/acero y equipo eléctrico.
- [ ] Red eléctrica de la colonia: paneles solares, turbinas eólicas, generadores de biocombustible y baterías.
- [ ] Estaciones de fabricación avanzada y estructuras defensivas terrestres.

---

## Fase 5 — Eventos, Audio y Distribución APK

- [ ] Eventos dinámicos terrestres (caravanas comerciales, brotes de clima extremo, incursiones hostiles con uso de la espada de cobre).
- [x] Efectos de sonido de talar, picar y construir (`.ogg` CC0) sincronizados desde Rust y conectados a `SettingsManager` *(pendiente: música ambiental acústica)*.
- [ ] Generación de APKs Release optimizados y firmados listos para publicación en Uptodown y descarga directa.
