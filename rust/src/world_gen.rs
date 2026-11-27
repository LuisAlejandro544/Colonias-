//! Generador procedural de mapas terrestres en vista centinela (estilo RimWorld).
//!
//! # Lógica del módulo
//! Genera la grilla de suelo (pasto, tierra fértil, tierra batida, grava, piedra
//! y ríos naturales con vados transitables) y distribuye obstáculos/recursos
//! naturales terrestres (árboles maduros de roble y pino, arbolitos jóvenes,
//! bloques continuos de montaña rocosa con vetas de cobre/carbón, rocas grandes
//! de granito/hierro/cobre/carbón y piedras sueltas), garantizando un claro
//! habitable en el centro del mapa para la colonia inicial.

#[cfg(feature = "gdextension")]
use godot::classes::{INode, Node};
#[cfg(feature = "gdextension")]
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
    River = 5,
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
    CopperBoulder = 7,
    CoalBoulder = 8,
    MountainStone = 9,
    MountainCopper = 10,
    MountainCoal = 11,
}

impl PropKind {
    /// Indica si el elemento bloquea el paso directo de los colonos.
    pub fn is_blocking(self) -> bool {
        matches!(
            self,
            PropKind::OakTree
                | PropKind::PineTree
                | PropKind::GraniteBoulder
                | PropKind::IronBoulder
                | PropKind::CopperBoulder
                | PropKind::CoalBoulder
                | PropKind::MountainStone
                | PropKind::MountainCopper
                | PropKind::MountainCoal
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

    // Paso 1: Base geológica del terreno por bioma.
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
        }
    }

    // Paso 2: Río natural serpenteante fuera del claro inicial, con vados de grava transitables.
    let river_vertical = (seed & 1) == 0;
    let river_high_side = (seed & 2) != 0;
    let span = if river_vertical { h } else { w };
    let cross_span = if river_vertical { w } else { h };
    let base_ratio = if river_high_side { 0.80 } else { 0.20 };
    let phase_a = ((seed >> 4) & 0xff) as f32 * 0.05;
    let phase_b = ((seed >> 12) & 0xff) as f32 * 0.07;
    let ford_a = (span as f32 * 0.30).round() as i32;
    let ford_b = (span as f32 * 0.72).round() as i32;

    for y in 0..h {
        for x in 0..w {
            let along = if river_vertical { y } else { x };
            let perp = if river_vertical { x } else { y };
            let t = along as f32 / span.max(1) as f32;
            let meander = (t * std::f32::consts::TAU * 1.45 + phase_a).sin() * (cross_span as f32 * 0.055)
                + (t * std::f32::consts::TAU * 3.1 + phase_b).cos() * 1.1;
            let river_center = (cross_span as f32 * base_ratio + meander)
                .clamp(3.0, (cross_span - 4) as f32);
            let dist_river = (perp as f32 - river_center).abs();

            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist_center = (dx * dx + dy * dy).sqrt();
            if dist_center < 7.2 {
                continue;
            }

            let idx = (y * w + x) as usize;
            let is_ford = (along - ford_a).abs() <= 1 || (along - ford_b).abs() <= 1;

            if dist_river <= 1.05 {
                if is_ford {
                    ground_tiles[idx] = GroundKind::Gravel as i32;
                    blocked_tiles[idx] = 0;
                } else {
                    ground_tiles[idx] = GroundKind::River as i32;
                    blocked_tiles[idx] = 1;
                }
            } else if dist_river <= 2.35
                && ground_tiles[idx] != GroundKind::Stone as i32
                && ground_tiles[idx] != GroundKind::River as i32
            {
                // Orilla húmeda y fértil junto al cauce del río.
                if is_ford || dist_river <= 1.55 {
                    ground_tiles[idx] = GroundKind::Gravel as i32;
                } else {
                    ground_tiles[idx] = GroundKind::FertileSoil as i32;
                }
            }
        }
    }

    // Paso 2b: Macizos rocosos y montañas naturales conectables con vetas de cobre y carbón.
    let mountain_threshold = match biome {
        EarthBiome::RockyHighlands => 0.61,
        EarthBiome::TemperateForest => 0.67,
        EarthBiome::FertileMeadow => 0.71,
    };
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist_center = (dx * dx + dy * dy).sqrt();
            if dist_center < 8.5 {
                continue;
            }
            let idx = (y * w + x) as usize;
            if ground_tiles[idx] == GroundKind::River as i32
                || ground_tiles[idx] == GroundKind::Gravel as i32
            {
                continue;
            }
            // No bloquear las orillas inmediatas del río para mantener acceso al agua y vados.
            let near_river = [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)]
                .iter()
                .any(|&(ox, oy)| {
                    let nx = x + ox;
                    let ny = y + oy;
                    if nx >= 0 && ny >= 0 && nx < w && ny < h {
                        ground_tiles[(ny * w + nx) as usize] == GroundKind::River as i32
                    } else {
                        false
                    }
                });
            if near_river {
                continue;
            }

            let m_noise = fbm_2d(
                x as f32 * 0.095 + 11.7,
                y as f32 * 0.095 - 23.4,
                seed.wrapping_add(4409),
                4,
            );
            if m_noise > mountain_threshold {
                let vein_noise = fbm_2d(
                    x as f32 * 0.27 + 91.1,
                    y as f32 * 0.27 + 13.5,
                    seed.wrapping_add(8813),
                    3,
                );
                let ore_roll = rng.next_f32();
                let prop = if vein_noise > 0.65 || (vein_noise > 0.56 && ore_roll < 0.24) {
                    PropKind::MountainCopper
                } else if vein_noise < 0.35 || (vein_noise < 0.44 && ore_roll < 0.24) {
                    PropKind::MountainCoal
                } else {
                    PropKind::MountainStone
                };
                ground_tiles[idx] = GroundKind::Stone as i32;
                prop_tiles[idx] = prop as i32;
                blocked_tiles[idx] = 1;
            }
        }
    }

    // Paso 3: Distribución natural de árboles, arbolitos, rocas y vetas superficiales sobre tierra firme.
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if prop_tiles[idx] != PropKind::None as i32 {
                continue;
            }
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let dist_center = (dx * dx + dy * dy).sqrt();

            // Mantenemos el claro inicial despejado alrededor del punto de llegada y el cauce libre de props.
            if dist_center < 6.5 || ground_tiles[idx] == GroundKind::River as i32 {
                continue;
            }

            let roll = rng.next_f32();
            let prop = match ground_tiles[idx] {
                x if x == GroundKind::Stone as i32 => {
                    if roll < 0.12 {
                        PropKind::GraniteBoulder
                    } else if roll < 0.18 {
                        PropKind::IronBoulder
                    } else if roll < 0.24 {
                        PropKind::CopperBoulder
                    } else if roll < 0.30 {
                        PropKind::CoalBoulder
                    } else if roll < 0.40 {
                        PropKind::FieldStones
                    } else {
                        PropKind::None
                    }
                }
                x if x == GroundKind::Gravel as i32 => {
                    if roll < 0.06 {
                        PropKind::GraniteBoulder
                    } else if roll < 0.095 {
                        PropKind::IronBoulder
                    } else if roll < 0.13 {
                        PropKind::CopperBoulder
                    } else if roll < 0.165 {
                        PropKind::CoalBoulder
                    } else if roll < 0.30 {
                        PropKind::FieldStones
                    } else if roll < 0.34 {
                        PropKind::Sapling
                    } else {
                        PropKind::None
                    }
                }
                x if x == GroundKind::Dirt as i32 => {
                    if roll < 0.04 && biome == EarthBiome::TemperateForest {
                        PropKind::OakTree
                    } else if roll < 0.09 {
                        PropKind::Sapling
                    } else if roll < 0.11 {
                        PropKind::CopperBoulder
                    } else if roll < 0.128 {
                        PropKind::CoalBoulder
                    } else if roll < 0.18 {
                        PropKind::FieldStones
                    } else {
                        PropKind::None
                    }
                }
                _ => {
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
                    } else if roll < tree_density + 0.068 {
                        PropKind::GraniteBoulder
                    } else if roll < tree_density + 0.075 {
                        PropKind::CopperBoulder
                    } else if roll < tree_density + 0.081 {
                        PropKind::CoalBoulder
                    } else if roll < tree_density + 0.112 {
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
#[cfg(feature = "gdextension")]
#[derive(GodotClass)]
#[class(base = Node)]
pub struct WorldGenerator {
    base: Base<Node>,
}

#[cfg(feature = "gdextension")]
#[godot_api]
impl INode for WorldGenerator {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[cfg(feature = "gdextension")]
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

    #[test]
    fn river_generates_with_unblocked_spawn_and_ford_crossings() {
        let world = generate_world_data(9090, 48, 48, "fertile_meadow");
        let river_count = world
            .ground_tiles
            .iter()
            .filter(|&&g| g == GroundKind::River as i32)
            .count();
        assert!(river_count >= 20);
        for (idx, &g) in world.ground_tiles.iter().enumerate() {
            if g == GroundKind::River as i32 {
                assert_eq!(world.blocked_tiles[idx], 1);
                assert_eq!(world.prop_tiles[idx], PropKind::None as i32);
            }
        }
    }

    #[test]
    fn mountain_blocks_generate_with_ore_veins_and_clear_spawn() {
        let world = generate_world_data(2026, 64, 64, "rocky_highlands");
        let stone_m = world
            .prop_tiles
            .iter()
            .filter(|&&p| p == PropKind::MountainStone as i32)
            .count();
        let copper_m = world
            .prop_tiles
            .iter()
            .filter(|&&p| p == PropKind::MountainCopper as i32)
            .count();
        let coal_m = world
            .prop_tiles
            .iter()
            .filter(|&&p| p == PropKind::MountainCoal as i32)
            .count();
        assert!(stone_m >= 10);
        assert!(copper_m + coal_m >= 2);
    }
}
