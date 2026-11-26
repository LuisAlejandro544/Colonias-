//! Curva de experiencia, multiplicadores y rendimientos por nivel (1 al 10) de cada rol.
//!
//! # Lógica del módulo
//! Separa las fórmulas de progresión y balanceo de los 10 roles oficiales de
//! **Colonia Centinela**:
//! - Experiencia requerida por nivel (`xp_required_for_next_level`).
//! - Bonificadores reales de velocidad, capacidad de carga, rendimiento de tala/minería,
//!   cosecha agrícola, impulso botánico, carpintería, cantería/horno, patrulla y liderazgo.
//! - Resumen legible en español (`effect_summary_es`) para el popup de Trabajos.

use super::{ColonyRole, MAX_ROLE_LEVEL};

/// Experiencia necesaria para avanzar desde `level` hasta `level + 1`.
/// La curva es gradual al inicio y presenta saltos más exigentes al acercarse al nivel 10.
pub fn xp_required_for_next_level(level: i32) -> f32 {
    match level {
        1 => 25.0,
        2 => 45.0,
        3 => 75.0,
        4 => 125.0,
        5 => 185.0,
        6 => 270.0,
        7 => 390.0,
        8 => 550.0,
        9 => 800.0,
        _ => 0.0,
    }
}

#[inline]
pub fn chop_speed_mult(level: i32) -> f32 {
    0.82 + (level.clamp(1, MAX_ROLE_LEVEL) - 1) as f32 * 0.12
}

#[inline]
pub fn chop_wood_yield(level: i32) -> i32 {
    let lv = level.clamp(1, MAX_ROLE_LEVEL);
    if lv >= 9 {
        6
    } else if lv >= 5 {
        5
    } else {
        4
    }
}

#[inline]
pub fn mine_speed_mult(level: i32) -> f32 {
    0.80 + (level.clamp(1, MAX_ROLE_LEVEL) - 1) as f32 * 0.125
}

#[inline]
pub fn mine_bonus_yield(level: i32) -> i32 {
    let lv = level.clamp(1, MAX_ROLE_LEVEL);
    if lv >= 8 {
        2
    } else if lv >= 4 {
        1
    } else {
        0
    }
}

#[inline]
pub fn haul_capacity(level: i32) -> i32 {
    match level.clamp(1, MAX_ROLE_LEVEL) {
        1 => 2,
        2..=3 => 4,
        4..=5 => 6,
        6..=7 => 9,
        8..=9 => 12,
        _ => 16,
    }
}

#[inline]
pub fn haul_speed_mult(level: i32) -> f32 {
    0.85 + (level.clamp(1, MAX_ROLE_LEVEL) - 1) as f32 * 0.085
}

#[inline]
pub fn build_speed_mult(level: i32) -> f32 {
    0.82 + (level.clamp(1, MAX_ROLE_LEVEL) - 1) as f32 * 0.12
}

#[inline]
pub fn farm_speed_mult(level: i32) -> f32 {
    0.82 + (level.clamp(1, MAX_ROLE_LEVEL) - 1) as f32 * 0.12
}

#[inline]
pub fn farm_harvest_yield(level: i32) -> (i32, i32) {
    let lv = level.clamp(1, MAX_ROLE_LEVEL);
    if lv >= 8 {
        (5, 3)
    } else if lv >= 4 {
        (4, 2)
    } else {
        (3, 2)
    }
}

#[inline]
pub fn botany_speed_mult(level: i32) -> f32 {
    0.82 + (level.clamp(1, MAX_ROLE_LEVEL) - 1) as f32 * 0.12
}

#[inline]
pub fn botany_growth_boost(level: i32) -> f32 {
    0.16 + (level.clamp(1, MAX_ROLE_LEVEL) - 1) as f32 * 0.025
}

#[inline]
pub fn scout_speed_mult(level: i32) -> f32 {
    0.90 + (level.clamp(1, MAX_ROLE_LEVEL) - 1) as f32 * 0.075
}

#[inline]
pub fn scout_radius(level: i32) -> i32 {
    6 + level.clamp(1, MAX_ROLE_LEVEL)
}

#[inline]
pub fn leader_meeting_speed_mult(level: i32) -> f32 {
    0.85 + (level.clamp(1, MAX_ROLE_LEVEL) - 1) as f32 * 0.10
}

#[inline]
pub fn carpentry_speed_mult(level: i32) -> f32 {
    0.82 + (level.clamp(1, MAX_ROLE_LEVEL) - 1) as f32 * 0.12
}

#[inline]
pub fn masonry_speed_mult(level: i32) -> f32 {
    0.82 + (level.clamp(1, MAX_ROLE_LEVEL) - 1) as f32 * 0.12
}

/// Texto breve en español con los bonos reales del nivel actual de un rol.
pub fn effect_summary_es(role: ColonyRole, level: i32) -> String {
    let lv = level.clamp(1, MAX_ROLE_LEVEL);
    match role {
        ColonyRole::Lumberjack => {
            format!(
                "Vel. tala x{:.2} • {} troncos por árbol",
                chop_speed_mult(lv),
                chop_wood_yield(lv)
            )
        }
        ColonyRole::Miner => {
            let bonus = mine_bonus_yield(lv);
            if bonus > 0 {
                format!(
                    "Vel. minería x{:.2} • +{} mineral extra",
                    mine_speed_mult(lv),
                    bonus
                )
            } else {
                format!("Vel. minería x{:.2} • Extracción base", mine_speed_mult(lv))
            }
        }
        ColonyRole::Hauler => {
            format!(
                "Carga máx: {} uds • Vel. acarreo x{:.2}",
                haul_capacity(lv),
                haul_speed_mult(lv)
            )
        }
        ColonyRole::Constructor => {
            format!("Vel. construcción x{:.2}", build_speed_mult(lv))
        }
        ColonyRole::Farmer => {
            let (carrots, seeds) = farm_harvest_yield(lv);
            format!(
                "Vel. labranza x{:.2} • Cosecha: {} zanahorias, {} sem.",
                farm_speed_mult(lv),
                carrots,
                seeds
            )
        }
        ColonyRole::Botanist => {
            format!(
                "Vel. cuidado x{:.2} • Impulso crecimiento +{:.0}%",
                botany_speed_mult(lv),
                botany_growth_boost(lv) * 100.0
            )
        }
        ColonyRole::Scout => {
            format!(
                "Vel. patrulla x{:.2} • Radio {} casillas",
                scout_speed_mult(lv),
                scout_radius(lv)
            )
        }
        ColonyRole::ColonyLeader => {
            format!(
                "Agilidad de reuniones x{:.2}",
                leader_meeting_speed_mult(lv)
            )
        }
        ColonyRole::Carpenter => {
            format!(
                "Vel. carpintería x{:.2} • Procesa madera y piezas en mesa",
                carpentry_speed_mult(lv)
            )
        }
        ColonyRole::Mason => {
            format!(
                "Vel. cantería y horno x{:.2} • Ladrillos, piezas y cobre",
                masonry_speed_mult(lv)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn higher_levels_improve_speed_yield_and_haul_capacity() {
        assert!(chop_speed_mult(10) > chop_speed_mult(1));
        assert!(chop_wood_yield(10) > chop_wood_yield(1));
        assert!(mine_speed_mult(10) > mine_speed_mult(1));
        assert!(mine_bonus_yield(10) > mine_bonus_yield(1));
        assert!(haul_capacity(1) < haul_capacity(5));
        assert!(haul_capacity(5) < haul_capacity(10));
        assert!(haul_speed_mult(10) > haul_speed_mult(1));
        assert!(build_speed_mult(10) > build_speed_mult(1));
        assert!(farm_speed_mult(10) > farm_speed_mult(1));
        assert!(farm_harvest_yield(10).0 > farm_harvest_yield(1).0);
        assert!(botany_growth_boost(10) > botany_growth_boost(1));
        assert!(scout_radius(10) > scout_radius(1));
        assert!(leader_meeting_speed_mult(10) > leader_meeting_speed_mult(1));
        assert!(carpentry_speed_mult(10) > carpentry_speed_mult(1));
        assert!(masonry_speed_mult(10) > masonry_speed_mult(1));
    }
}
