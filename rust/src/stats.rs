//! Sistema de estadísticas y atributos personales de cada colono.
//!
//! # Lógica del módulo
//! Define los 5 atributos oficiales de **Colonia Centinela** (`ColonistStats`)
//! en escala `1..=20`, compatibles con arquitecturas de 32 y 64 bits:
//! - **Inteligencia (`INT`)**: habilita fabricar objetos y estructuras más complejas
//!   según requisito mínimo, acelera el aprendizaje de roles y potencia al Botánico.
//! - **Fuerza (`FUE`)**: aumenta la capacidad de carga y acelera tala y minería.
//! - **Destreza (`DES`)**: mejora velocidad de movimiento, construcción, carpintería y labranza.
//! - **Resistencia (`RES`)**: reduce el desgaste de energía y eleva la salud máxima.
//! - **Carisma (`CAR`)**: agiliza reuniones del Líder y mejora el ánimo del grupo.

use crate::noise::SeededRng;

/// Rango mínimo y máximo de cada estadística personal.
pub const MIN_STAT_VALUE: i32 = 1;
pub const MAX_STAT_VALUE: i32 = 20;

/// Identificador de cada una de las 5 estadísticas base.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StatKind {
    Intelligence = 0,
    Strength = 1,
    Dexterity = 2,
    Endurance = 3,
    Charisma = 4,
}

impl StatKind {
    pub const ALL: [Self; 5] = [
        Self::Intelligence,
        Self::Strength,
        Self::Dexterity,
        Self::Endurance,
        Self::Charisma,
    ];

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Intelligence => "intelligence",
            Self::Strength => "strength",
            Self::Dexterity => "dexterity",
            Self::Endurance => "endurance",
            Self::Charisma => "charisma",
        }
    }

    pub const fn short_es(self) -> &'static str {
        match self {
            Self::Intelligence => "INT",
            Self::Strength => "FUE",
            Self::Dexterity => "DES",
            Self::Endurance => "RES",
            Self::Charisma => "CAR",
        }
    }

    pub const fn label_es(self) -> &'static str {
        match self {
            Self::Intelligence => "Inteligencia",
            Self::Strength => "Fuerza",
            Self::Dexterity => "Destreza",
            Self::Endurance => "Resistencia",
            Self::Charisma => "Carisma",
        }
    }
}

/// Estadísticas base y progreso práctico de un colono.
#[derive(Clone, Debug)]
pub struct ColonistStats {
    pub intelligence: i32,
    pub strength: i32,
    pub dexterity: i32,
    pub endurance: i32,
    pub charisma: i32,
    pub stat_xp: [f32; 5],
}

impl Default for ColonistStats {
    fn default() -> Self {
        Self {
            intelligence: 3,
            strength: 4,
            dexterity: 4,
            endurance: 4,
            charisma: 3,
            stat_xp: [0.0; 5],
        }
    }
}

impl ColonistStats {
    /// Genera un perfil variado y determinista para un colono nuevo.
    pub fn generate_for_colonist(rng: &mut SeededRng, index: i32, is_leader: bool) -> Self {
        let mut s = Self {
            intelligence: (2 + rng.range_i32(0, 4)).clamp(MIN_STAT_VALUE, MAX_STAT_VALUE),
            strength: (2 + rng.range_i32(0, 5)).clamp(MIN_STAT_VALUE, MAX_STAT_VALUE),
            dexterity: (2 + rng.range_i32(0, 5)).clamp(MIN_STAT_VALUE, MAX_STAT_VALUE),
            endurance: (3 + rng.range_i32(0, 4)).clamp(MIN_STAT_VALUE, MAX_STAT_VALUE),
            charisma: (2 + rng.range_i32(0, 4)).clamp(MIN_STAT_VALUE, MAX_STAT_VALUE),
            stat_xp: [0.0; 5],
        };

        // Ligera especialización según arquetipo inicial para crear variedad entre colonos.
        match index % 5 {
            0 => s.strength = (s.strength + 2).min(MAX_STAT_VALUE),
            1 => s.intelligence = (s.intelligence + 2).min(MAX_STAT_VALUE),
            2 => s.dexterity = (s.dexterity + 2).min(MAX_STAT_VALUE),
            3 => s.endurance = (s.endurance + 2).min(MAX_STAT_VALUE),
            _ => s.charisma = (s.charisma + 2).min(MAX_STAT_VALUE),
        }

        if is_leader {
            s.charisma = (s.charisma + 2).clamp(4, MAX_STAT_VALUE);
            s.intelligence = (s.intelligence + 1).clamp(3, MAX_STAT_VALUE);
        }
        s
    }

    #[inline]
    pub fn get(&self, kind: StatKind) -> i32 {
        match kind {
            StatKind::Intelligence => self.intelligence.clamp(MIN_STAT_VALUE, MAX_STAT_VALUE),
            StatKind::Strength => self.strength.clamp(MIN_STAT_VALUE, MAX_STAT_VALUE),
            StatKind::Dexterity => self.dexterity.clamp(MIN_STAT_VALUE, MAX_STAT_VALUE),
            StatKind::Endurance => self.endurance.clamp(MIN_STAT_VALUE, MAX_STAT_VALUE),
            StatKind::Charisma => self.charisma.clamp(MIN_STAT_VALUE, MAX_STAT_VALUE),
        }
    }

    pub fn set(&mut self, kind: StatKind, value: i32) {
        let clamped = value.clamp(MIN_STAT_VALUE, MAX_STAT_VALUE);
        match kind {
            StatKind::Intelligence => self.intelligence = clamped,
            StatKind::Strength => self.strength = clamped,
            StatKind::Dexterity => self.dexterity = clamped,
            StatKind::Endurance => self.endurance = clamped,
            StatKind::Charisma => self.charisma = clamped,
        }
        if clamped >= MAX_STAT_VALUE {
            self.stat_xp[kind.index()] = 0.0;
        }
    }

    #[inline]
    pub fn xp_needed_for_next(current_val: i32) -> f32 {
        if current_val >= MAX_STAT_VALUE {
            return 0.0;
        }
        30.0 + (current_val.clamp(MIN_STAT_VALUE, MAX_STAT_VALUE) as f32) * 18.0
    }

    /// Entrena lentamente un atributo al realizar tareas asociadas. Devuelve `true` si subió.
    pub fn grant_stat_xp(&mut self, kind: StatKind, amount: f32) -> bool {
        if amount <= 0.0 {
            return false;
        }
        let idx = kind.index();
        let cur = self.get(kind);
        if cur >= MAX_STAT_VALUE {
            self.stat_xp[idx] = 0.0;
            return false;
        }
        self.stat_xp[idx] += amount;
        let mut improved = false;
        while self.get(kind) < MAX_STAT_VALUE {
            let need = Self::xp_needed_for_next(self.get(kind));
            if need <= 0.0 || self.stat_xp[idx] < need {
                break;
            }
            self.stat_xp[idx] -= need;
            let next_v = self.get(kind) + 1;
            self.set(kind, next_v);
            improved = true;
        }
        improved
    }

    /// Verifica si el colono cumple el requisito mínimo de Inteligencia para fabricar algo.
    #[inline]
    pub fn can_craft_complexity(&self, min_intelligence: i32) -> bool {
        self.get(StatKind::Intelligence) >= min_intelligence
    }

    /// Nivel descriptivo de complejidad de fabricación habilitada por su Inteligencia.
    pub fn craft_tier_label_es(&self) -> &'static str {
        let int_v = self.get(StatKind::Intelligence);
        if int_v >= 8 {
            "Ingeniería y mecanismos complejos (INT 8+)"
        } else if int_v >= 4 {
            "Ebanistería completa: tablones, cubos, puertas y arcones (INT 4+)"
        } else if int_v >= 3 {
            "Carpintería media: tablones, cubos y puertas (INT 3+)"
        } else if int_v >= 2 {
            "Carpintería inicial: tablones y cubos (INT 2+)"
        } else {
            "Procesado básico: tablones de madera (INT 1+)"
        }
    }

    #[inline]
    pub fn xp_gain_mult(&self) -> f32 {
        0.78 + (self.get(StatKind::Intelligence) as f32) * 0.055
    }

    #[inline]
    pub fn botany_intel_mult(&self) -> f32 {
        0.86 + (self.get(StatKind::Intelligence) as f32) * 0.035
    }

    #[inline]
    pub fn carry_bonus(&self) -> i32 {
        (self.get(StatKind::Strength) - 3).max(0) / 2
    }

    #[inline]
    pub fn chop_strength_mult(&self) -> f32 {
        0.86 + (self.get(StatKind::Strength) as f32) * 0.035
    }

    #[inline]
    pub fn mine_strength_mult(&self) -> f32 {
        0.86 + (self.get(StatKind::Strength) as f32) * 0.035
    }

    #[inline]
    pub fn move_speed_mult(&self) -> f32 {
        0.88 + (self.get(StatKind::Dexterity) as f32) * 0.025
    }

    #[inline]
    pub fn build_dex_mult(&self) -> f32 {
        0.85 + (self.get(StatKind::Dexterity) as f32) * 0.035
    }

    #[inline]
    pub fn craft_dex_mult(&self) -> f32 {
        0.84 + (self.get(StatKind::Dexterity) as f32) * 0.04
    }

    #[inline]
    pub fn farm_dex_mult(&self) -> f32 {
        0.86 + (self.get(StatKind::Dexterity) as f32) * 0.032
    }

    #[inline]
    pub fn energy_drain_mult(&self) -> f32 {
        (1.16 - (self.get(StatKind::Endurance) as f32) * 0.028).clamp(0.55, 1.15)
    }

    #[inline]
    pub fn max_health(&self) -> i32 {
        88 + self.get(StatKind::Endurance) * 3
    }

    #[inline]
    pub fn meeting_charisma_mult(&self) -> f32 {
        0.84 + (self.get(StatKind::Charisma) as f32) * 0.04
    }

    #[inline]
    pub fn meeting_mood_boost(&self) -> i32 {
        if self.get(StatKind::Charisma) >= 7 {
            2
        } else {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intelligence_gates_complex_crafting_and_grows_with_practice() {
        let mut stats = ColonistStats {
            intelligence: 2,
            strength: 5,
            dexterity: 4,
            endurance: 6,
            charisma: 3,
            stat_xp: [0.0; 5],
        };
        assert!(stats.can_craft_complexity(1));
        assert!(stats.can_craft_complexity(2));
        assert!(!stats.can_craft_complexity(3));
        assert!(!stats.can_craft_complexity(4));

        let needed = ColonistStats::xp_needed_for_next(2);
        assert!(stats.grant_stat_xp(StatKind::Intelligence, needed + 1.0));
        assert_eq!(stats.intelligence, 3);
        assert!(stats.can_craft_complexity(3));
    }

    #[test]
    fn stats_modifiers_scale_in_expected_directions() {
        let low = ColonistStats {
            intelligence: 2,
            strength: 2,
            dexterity: 2,
            endurance: 2,
            charisma: 2,
            stat_xp: [0.0; 5],
        };
        let high = ColonistStats {
            intelligence: 10,
            strength: 10,
            dexterity: 10,
            endurance: 10,
            charisma: 10,
            stat_xp: [0.0; 5],
        };
        assert!(high.xp_gain_mult() > low.xp_gain_mult());
        assert!(high.carry_bonus() > low.carry_bonus());
        assert!(high.craft_dex_mult() > low.craft_dex_mult());
        assert!(high.energy_drain_mult() < low.energy_drain_mult());
        assert!(high.max_health() > low.max_health());
        assert!(high.meeting_charisma_mult() > low.meeting_charisma_mult());
    }
}
