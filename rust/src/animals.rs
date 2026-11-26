//! Fauna terrestre (cerditos, pollos, ovejas y vacas) dependiente del bioma.
//!
//! # Lógica del módulo
//! Define las especies de animales que caminan por el mapa según el bioma
//! (`Bosque Templado`, `Pradera Fértil`, `Sierra Rocosa`) junto a sus cortes
//! específicos de carne cruda (pechuga/muslo de pollo, panceta/costillar de
//! cerdo, paletilla/pierna de oveja, solomillo/pecho de res) y huesos para
//! futuras recetas como sopa de huesos.

use crate::colony_types::ItemKind;
use crate::noise::SeededRng;

/// Especies de animales terrestres disponibles en el mundo.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AnimalKind {
    Pig = 1,
    Chicken = 2,
    Sheep = 3,
    Cow = 4,
}

impl AnimalKind {
    pub const ALL: [Self; 4] = [Self::Pig, Self::Chicken, Self::Sheep, Self::Cow];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Pig => "pig",
            Self::Chicken => "chicken",
            Self::Sheep => "sheep",
            Self::Cow => "cow",
        }
    }

    pub const fn label_es(self) -> &'static str {
        match self {
            Self::Pig => "Cerdito",
            Self::Chicken => "Pollo",
            Self::Sheep => "Oveja",
            Self::Cow => "Vaca",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "pig" => Some(Self::Pig),
            "chicken" => Some(Self::Chicken),
            "sheep" => Some(Self::Sheep),
            "cow" => Some(Self::Cow),
            _ => None,
        }
    }

    /// Cortes específicos de carne cruda y huesos obtenidos al cazar este animal.
    pub fn butcher_drops(self) -> &'static [(ItemKind, i32)] {
        match self {
            Self::Chicken => &[
                (ItemKind::RawChickenBreast, 2),
                (ItemKind::RawChickenThigh, 2),
                (ItemKind::AnimalBones, 1),
            ],
            Self::Pig => &[
                (ItemKind::RawPorkBelly, 2),
                (ItemKind::RawPorkRibs, 2),
                (ItemKind::AnimalBones, 2),
            ],
            Self::Sheep => &[
                (ItemKind::RawMuttonShoulder, 2),
                (ItemKind::RawMuttonLeg, 2),
                (ItemKind::AnimalBones, 2),
            ],
            Self::Cow => &[
                (ItemKind::RawBeefSirloin, 3),
                (ItemKind::RawBeefBrisket, 2),
                (ItemKind::AnimalBones, 3),
            ],
        }
    }

    /// Velocidad base de caminata en casillas por segundo.
    pub const fn base_speed(self) -> f32 {
        match self {
            Self::Chicken => 1.65,
            Self::Pig => 1.45,
            Self::Sheep => 1.30,
            Self::Cow => 1.10,
        }
    }
}

/// Estado individual de un animal libre en el mapa.
#[derive(Clone, Debug)]
pub struct AnimalEntity {
    pub id: i32,
    pub kind: AnimalKind,
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub facing_x: f32,
    pub facing_y: f32,
    pub speed: f32,
    pub wait_timer: f32,
    pub walk_phase: f32,
    pub marked_for_hunt: bool,
}

/// Elige una especie de animal acorde a la ecología de cada bioma terrestre.
pub fn pick_animal_for_biome(rng: &mut SeededRng, biome_code: &str) -> AnimalKind {
    let roll = rng.next_f32();
    match biome_code {
        "fertile_meadow" => {
            // Pradera fértil: abundan vacas, ovejas y pollos; menos cerditos.
            if roll < 0.36 {
                AnimalKind::Cow
            } else if roll < 0.68 {
                AnimalKind::Sheep
            } else if roll < 0.90 {
                AnimalKind::Chicken
            } else {
                AnimalKind::Pig
            }
        }
        "rocky_highlands" => {
            // Sierra rocosa: predominan ovejas de montaña, con pocos pollos y cerditos.
            if roll < 0.56 {
                AnimalKind::Sheep
            } else if roll < 0.80 {
                AnimalKind::Chicken
            } else if roll < 0.94 {
                AnimalKind::Pig
            } else {
                AnimalKind::Cow
            }
        }
        _ => {
            // Bosque templado: predominan cerditos y pollos, con algunas ovejas y pocas vacas.
            if roll < 0.40 {
                AnimalKind::Pig
            } else if roll < 0.74 {
                AnimalKind::Chicken
            } else if roll < 0.90 {
                AnimalKind::Sheep
            } else {
                AnimalKind::Cow
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animal_kinds_roundtrip_and_yield_specific_cuts_plus_bones() {
        for kind in AnimalKind::ALL {
            assert_eq!(AnimalKind::from_code(kind.code()), Some(kind));
            assert!(!kind.label_es().is_empty());
            let drops = kind.butcher_drops();
            assert!(drops.iter().any(|(it, amt)| *it == ItemKind::AnimalBones && *amt > 0));
            assert!(drops.iter().any(|(it, amt)| *it != ItemKind::AnimalBones && *amt > 0));
        }
    }

    #[test]
    fn biome_distribution_matches_ecology() {
        let mut rng_forest = SeededRng::new(111);
        let mut pigs = 0;
        for _ in 0..200 {
            if pick_animal_for_biome(&mut rng_forest, "temperate_forest") == AnimalKind::Pig {
                pigs += 1;
            }
        }
        assert!(pigs > 45);

        let mut rng_meadow = SeededRng::new(222);
        let mut cows = 0;
        for _ in 0..200 {
            if pick_animal_for_biome(&mut rng_meadow, "fertile_meadow") == AnimalKind::Cow {
                cows += 1;
            }
        }
        assert!(cows > 40);

        let mut rng_rocky = SeededRng::new(333);
        let mut sheep = 0;
        for _ in 0..200 {
            if pick_animal_for_biome(&mut rng_rocky, "rocky_highlands") == AnimalKind::Sheep {
                sheep += 1;
            }
        }
        assert!(sheep > 75);
    }
}
