//! Generador procedural de mapas terrestres en vista centinela (estilo RimWorld).
//!
//! # Lógica del módulo
//! Genera la grilla de suelo (pasto, tierra fértil, tierra batida, grava y piedra)
//! y distribuye obstáculos/recursos naturales terrestres (árboles maduros de roble
//! y pino, arbolitos jóvenes, rocas grandes de granito/hierro y piedras sueltas),
//! garantizando un claro habitable en el centro del mapa para la colonia inicial.

use godot::classes::{INode, Node};
use godot::prelude::*;

use crate::noise::{fbm_2d, SeededRng};

/// Tipos de suelo (piso) representados en la grilla.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GroundKind {
    Grass = 0,
    FertileSoil = 1,
    Dirt = 2,
    Gravel = 3,
    Stone = 4,
}

/// Tipos de elementos naturales sobre el terreno.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PropKind {
    None = 0,
    OakTree = 1,
    PineTree = 2,
    Sapling = 3,
    GraniteBoulder = 4,
    IronBoulder = 5,
    FieldStones = 6,
}

impl PropKind {
    /// Indica si el elemento bloquea el paso directo de los colonos.
    pub fn is_blocking(self) -> bool {
        matches!(
            self,
            PropKind::OakTree | PropKind::PineTree | PropKind::GraniteBoulder | PropKind::IronBoulder
        )
    }
}

/// Biomas terrestres soportados al fundar una colonia.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EarthBiome {
    TemperateForest,
    FertileMeadow,
    RockyHighlands,
}

impl EarthBiome {
    pub fn from_code(code: &str) -> Self {
        match code {
            "fertile_meadow" => Self::FertileMeadow,
            "rocky_highlands" => Self::RockyHighlands,
            _ => Self::TemperateForest,
        }
    }
}

/// Datos puros del mundo generado listos para enviarse a Godot o probarse en Rust.
#[derive(Clone, Debug)]
pub struct GeneratedWorld {
    pub width: i32,
    pub height: i32,
    pub ground_tiles: Vec<i32>,
    pub prop_tiles: Vec<i32>,
    pub blocked_tiles: Vec<i32>,
}

/// Genera un mapa terrestre completo de forma determinista a partir de semilla y bioma.
pub fn generate_world_data(seed: u64, width: i32, height: i32, biome_code: &str) -> GeneratedWorld {
    let w = width.clamp(32, 160);
    let h = height.clamp(32, 160);
    let total = (w * h) as usize;
    let biome = EarthBiome::from_code(biome_code);

    let mut ground_tiles = vec![GroundKind::Grass as i32; total];
    let mut prop_tiles = vec![PropKind::None as i32; total];
    let mut blocked_tiles = vec![0i32; total];

    let mut rng = SeededRng::new(seed ^ 0xA5A5_5A5A_1234_5678);
    let center_x = w as f32 * 0.5;
    let center_y = h as f32 * 0.5;

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            let nx = x as f32 * 0.065;
            let ny = y as f32 * 0.065;

            let elevation = fbm_2d(nx, ny, seed, 4);
            let moisture = fbm_2d(nx + 41.3, ny + 17.9, seed.wrapping_add(777), 4);
            let detail = fbm_2d(x as f32 * 0.19, y as f32 * 0.19, seed.wrapping_add(1999), 2);

            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist_center = (dx * dx + dy * dy).sqrt();

            // 1. Selección del suelo (piso) según bioma terrestre.
            let ground = match biome {
                EarthBiome::TemperateForest => {
                    if elevation > 0.72 {
                        GroundKind::Stone
                    } else if elevation > 0.64 {
                        GroundKind::Gravel
                    } else if moisture > 0.63 && dist_center > 5.0 {
                        GroundKind::FertileSoil
                    } else if detail < 0.30 {
                        GroundKind::Dirt
                    } else {
                        GroundKind::Grass
                    }
                }
                EarthBiome::FertileMeadow => {
                    if elevation > 0.78 {
                        GroundKind::Stone
                    } else if elevation > 0.70 {
                        GroundKind::Gravel
                    } else if moisture > 0.50 {
                        GroundKind::FertileSoil
                    } else if detail < 0.22 {
                        GroundKind::Dirt
                    } else {
                        GroundKind::Grass
                    }
                }
                EarthBiome::RockyHighlands => {
                    if elevation > 0.58 {
                        GroundKind::Stone
                    } else if elevation > 0.45 {
                        GroundKind::Gravel
                    } else if moisture > 0.68 {
                        GroundKind::FertileSoil
                    } else if detail < 0.48 {
                        GroundKind::Dirt
                    } else {
                        GroundKind::Grass
                    }
                }
            };

            ground_tiles[idx] = ground as i32;

            // Mantenemos el claro inicial despejado alrededor del punto de llegada.
            if dist_center < 6.5 {
                continue;
            }

            // 2. Distribución natural de árboles, arbolitos, rocas y piedras.
            let roll = rng.next_f32();
            let prop = match ground {
                GroundKind::Stone => {
                    if roll < 0.16 {
                        PropKind::GraniteBoulder
                    } else if roll < 0.22 {
                        PropKind::IronBoulder
                    } else if roll < 0.34 {
                        PropKind::FieldStones
                    } else {
                        PropKind::None
                    }
                }
                GroundKind::Gravel => {
                    if roll < 0.08 {
                        PropKind::GraniteBoulder
                    } else if roll < 0.11 {
                        PropKind::IronBoulder
                    } else if roll < 0.27 {
                        PropKind::FieldStones
                    } else if roll < 0.31 {
                        PropKind::Sapling
                    } else {
                        PropKind::None
                    }
                }
                GroundKind::Dirt => {
                    if roll < 0.04 && biome == EarthBiome::TemperateForest {
                        PropKind::OakTree
                    } else if roll < 0.09 {
                        PropKind::Sapling
                    } else if roll < 0.15 {
                        PropKind::FieldStones
                    } else {
                        PropKind::None
                    }
                }
                GroundKind::Grass | GroundKind::FertileSoil => {
                    let tree_density = match biome {
                        EarthBiome::TemperateForest => 0.13,
                        EarthBiome::FertileMeadow => 0.065,
                        EarthBiome::RockyHighlands => 0.045,
                    };
                    if roll < tree_density * 0.58 {
                        PropKind::OakTree
                    } else if roll < tree_density {
                        PropKind::PineTree
                    } else if roll < tree_density + 0.055 {
                        PropKind::Sapling
                    } else if roll < tree_density + 0.072 {
                        PropKind::GraniteBoulder
                    } else if roll < tree_density + 0.105 {
                        PropKind::FieldStones
                    } else {
                        PropKind::None
                    }
                }
            };

            prop_tiles[idx] = prop as i32;
            blocked_tiles[idx] = if prop.is_blocking() { 1 } else { 0 };
        }
    }

    GeneratedWorld {
        width: w,
        height: h,
        ground_tiles,
        prop_tiles,
        blocked_tiles,
    }
}

/// Nodo GDExtension expuesto a Godot 4.7 para generar mundos procedurales desde GDScript.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct WorldGenerator {
    base: Base<Node>,
}

#[godot_api]
impl INode for WorldGenerator {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl WorldGenerator {
    /// Genera el terreno y los elementos naturales devolviendo un `Dictionary` con
    /// `width`, `height`, `ground` (`PackedInt32Array`), `props` (`PackedInt32Array`)
    /// y `blocked` (`PackedInt32Array`).
    #[func]
    pub fn generate_world(
        &self,
        seed: i64,
        width: i32,
        height: i32,
        biome_code: GString,
    ) -> VarDictionary {
        let world = generate_world_data(seed as u64, width, height, &biome_code.to_string());

        let ground_arr = PackedInt32Array::from(world.ground_tiles.as_slice());
        let prop_arr = PackedInt32Array::from(world.prop_tiles.as_slice());
        let blocked_arr = PackedInt32Array::from(world.blocked_tiles.as_slice());

        let mut out = VarDictionary::new();
        out.set("width", &world.width.to_variant());
        out.set("height", &world.height.to_variant());
        out.set("ground", &ground_arr.to_variant());
        out.set("props", &prop_arr.to_variant());
        out.set("blocked", &blocked_arr.to_variant());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_spawn_clearing_remains_unblocked() {
        let world = generate_world_data(4242, 64, 64, "temperate_forest");
        let cx = 32;
        let cy = 32;
        for dy in -3..=3 {
            for dx in -3..=3 {
                let idx = ((cy + dy) * world.width + (cx + dx)) as usize;
                assert_eq!(world.prop_tiles[idx], PropKind::None as i32);
                assert_eq!(world.blocked_tiles[idx], 0);
            }
        }
    }

    #[test]
    fn same_seed_generates_identical_maps() {
        let a = generate_world_data(777, 48, 48, "rocky_highlands");
        let b = generate_world_data(777, 48, 48, "rocky_highlands");
        assert_eq!(a.ground_tiles, b.ground_tiles);
        assert_eq!(a.prop_tiles, b.prop_tiles);
    }
}
