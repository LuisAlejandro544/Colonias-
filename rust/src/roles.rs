//! Sistema modular de roles, límites de asignación y niveles (1 al 10) para colonos.
//!
//! # Lógica del módulo
//! Define los 8 roles oficiales de **Colonia Centinela**:
//! - `Leñador`: tala árboles con Hacha de cobre y obtiene troncos.
//! - `Minero`: pica rocas de granito y vetas de hierro/cobre con Pico de cobre.
//! - `Transportador`: acarrea recursos del piso hacia el almacén (su nivel cambia velocidad y capacidad de carga).
//! - `Constructor`: levanta planos y paredes de madera con Pala de cobre.
//! - `Agricultor`: ara tierra cultivable y cosecha zanahorias maduras con Azada de cobre.
//! - `Botánico`: siembra semillas de zanahoria y cuida plantas en crecimiento para acelerar su maduración.
//! - `Explorador`: patrulla un radio mayor alrededor de la colonia con Espada de cobre.
//! - `Líder de Colonia`: convoca reuniones para aprobar órdenes más rápido y coordina al equipo.
//!
//! Cada colono puede tener asignados hasta un máximo de **5 roles** simultáneos y
//! conserva su nivel (de `1` a `10`) y experiencia acumulada de forma independiente
//! para cada uno de los 8 roles, con una curva gradual que se vuelve más lenta en
//! los niveles altos.

use crate::tools::CopperTool;

/// Máximo de roles activos que puede tener asignados un colono a la vez.
pub const MAX_ASSIGNED_ROLES: usize = 5;

/// Nivel máximo alcanzable en cualquier rol.
pub const MAX_ROLE_LEVEL: i32 = 10;

/// Los 8 roles disponibles en la colonia.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ColonyRole {
    Lumberjack = 0,
    Miner = 1,
    Hauler = 2,
    Constructor = 3,
    Farmer = 4,
    Botanist = 5,
    Scout = 6,
    ColonyLeader = 7,
}

impl ColonyRole {
    /// Lista ordenada de todos los roles disponibles.
    pub const ALL: [Self; 8] = [
        Self::Lumberjack,
        Self::Miner,
        Self::Hauler,
        Self::Constructor,
        Self::Farmer,
        Self::Botanist,
        Self::Scout,
        Self::ColonyLeader,
    ];

    /// Índice fijo (`0..8`) compatible con 32 y 64 bits.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Identificador estable para GDScript y serialización JSON.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Lumberjack => "lumberjack",
            Self::Miner => "miner",
            Self::Hauler => "hauler",
            Self::Constructor => "constructor",
            Self::Farmer => "farmer",
            Self::Botanist => "botanist",
            Self::Scout => "scout",
            Self::ColonyLeader => "colony_leader",
        }
    }

    /// Nombre visible en español.
    pub const fn label_es(self) -> &'static str {
        match self {
            Self::Lumberjack => "Leñador",
            Self::Miner => "Minero",
            Self::Hauler => "Transportador",
            Self::Constructor => "Constructor",
            Self::Farmer => "Agricultor",
            Self::Botanist => "Botánico",
            Self::Scout => "Explorador",
            Self::ColonyLeader => "Líder de Colonia",
        }
    }

    /// Descripción clara en español de la función del rol.
    pub const fn description_es(self) -> &'static str {
        match self {
            Self::Lumberjack => "Tala árboles con hacha de cobre. Su nivel acelera el corte y aumenta los troncos obtenidos.",
            Self::Miner => "Pica rocas y vetas minerales. Su nivel acelera la excavación y mejora la extracción de piedra y mena.",
            Self::Hauler => "Recoge ítems del suelo y los guarda en el almacén. Su nivel sube la cantidad transportada por viaje y su rapidez.",
            Self::Constructor => "Construye muros y estructuras desde planos. Su nivel acelera la edificación.",
            Self::Farmer => "Ara la tierra y cosecha zanahorias maduras. Su nivel acelera la labranza y aumenta la cosecha.",
            Self::Botanist => "Siembra semillas y cuida cultivos en crecimiento para acelerar su maduración.",
            Self::Scout => "Patrulla e inspecciona un perímetro más amplio. Su nivel incrementa la velocidad de patrulla y el alcance.",
            Self::ColonyLeader => "Reúne al equipo para aprobar órdenes. Su nivel hace más ágiles las reuniones y la coordinación.",
        }
    }

    /// Reconstruye un rol a partir de su código interno.
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "lumberjack" => Some(Self::Lumberjack),
            "miner" => Some(Self::Miner),
            "hauler" => Some(Self::Hauler),
            "constructor" => Some(Self::Constructor),
            "farmer" => Some(Self::Farmer),
            "botanist" => Some(Self::Botanist),
            "scout" => Some(Self::Scout),
            "colony_leader" => Some(Self::ColonyLeader),
            _ => None,
        }
    }

    /// Reconstruye un rol a partir de su etiqueta en español (útil para partidas previas).
    pub fn from_label_es(label: &str) -> Option<Self> {
        match label {
            "Leñador" => Some(Self::Lumberjack),
            "Minero" => Some(Self::Miner),
            "Transportador" => Some(Self::Hauler),
            "Constructor" => Some(Self::Constructor),
            "Agricultor" => Some(Self::Farmer),
            "Botánico" => Some(Self::Botanist),
            "Explorador" => Some(Self::Scout),
            "Líder de Colonia" => Some(Self::ColonyLeader),
            _ => None,
        }
    }

    /// Herramienta de cobre asociada al rol.
    pub const fn default_tool(self) -> CopperTool {
        match self {
            Self::Lumberjack => CopperTool::CopperAxe,
            Self::Miner => CopperTool::CopperPickaxe,
            Self::Hauler => CopperTool::CopperShovel,
            Self::Constructor => CopperTool::CopperShovel,
            Self::Farmer => CopperTool::CopperHoe,
            Self::Botanist => CopperTool::CopperHoe,
            Self::Scout => CopperTool::CopperSword,
            Self::ColonyLeader => CopperTool::CopperSword,
        }
    }
}

/// Progreso individual (nivel y experiencia) de un colono en un rol concreto.
#[derive(Copy, Clone, Debug)]
pub struct RoleProgress {
    pub level: i32,
    pub xp: f32,
}

impl Default for RoleProgress {
    fn default() -> Self {
        Self { level: 1, xp: 0.0 }
    }
}

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

/// Perfil completo de roles asignados (máx. 5) y niveles (1..=10) de un colono.
#[derive(Clone, Debug)]
pub struct ColonistRoleProfile {
    pub assigned: Vec<ColonyRole>,
    pub skills: [RoleProgress; 8],
}

impl Default for ColonistRoleProfile {
    fn default() -> Self {
        Self {
            assigned: Vec::new(),
            skills: [RoleProgress::default(); 8],
        }
    }
}

impl ColonistRoleProfile {
    /// Crea un perfil inicial balanceado para el colono según su índice en la colonia.
    pub fn new_for_colonist(colonist_index: usize, is_leader: bool) -> Self {
        let mut profile = Self::default();

        // Roles principales iniciales rotativos para cubrir toda la colonia incluso con 2-3 colonos.
        let preset: &[ColonyRole] = match colonist_index % 6 {
            0 => &[
                ColonyRole::Lumberjack,
                ColonyRole::Constructor,
                ColonyRole::Farmer,
                ColonyRole::Hauler,
            ],
            1 => &[
                ColonyRole::Miner,
                ColonyRole::Botanist,
                ColonyRole::Hauler,
                ColonyRole::Scout,
            ],
            2 => &[
                ColonyRole::Farmer,
                ColonyRole::Botanist,
                ColonyRole::Constructor,
                ColonyRole::Hauler,
            ],
            3 => &[
                ColonyRole::Constructor,
                ColonyRole::Lumberjack,
                ColonyRole::Miner,
                ColonyRole::Hauler,
            ],
            4 => &[
                ColonyRole::Scout,
                ColonyRole::Lumberjack,
                ColonyRole::Botanist,
                ColonyRole::Hauler,
            ],
            _ => &[
                ColonyRole::Botanist,
                ColonyRole::Miner,
                ColonyRole::Farmer,
                ColonyRole::Hauler,
            ],
        };

        for &r in preset {
            let _ = profile.assign_role(r);
        }

        // Dar una ligera ventaja inicial (Nivel 2) en su primer oficio de base.
        if let Some(&primary) = preset.first() {
            profile.skills[primary.index()].level = 2;
        }

        if is_leader {
            let _ = profile.assign_role(ColonyRole::ColonyLeader);
            profile.skills[ColonyRole::ColonyLeader.index()].level = 2;
        }

        profile
    }

    #[inline]
    pub fn has_role(&self, role: ColonyRole) -> bool {
        self.assigned.contains(&role)
    }

    /// Asigna un rol respetando el límite máximo de 5 roles por colono.
    pub fn assign_role(&mut self, role: ColonyRole) -> Result<(), &'static str> {
        if self.has_role(role) {
            return Ok(());
        }
        if self.assigned.len() >= MAX_ASSIGNED_ROLES {
            return Err("Este colono ya tiene el máximo de 5 roles asignados.");
        }
        self.assigned.push(role);
        Ok(())
    }

    /// Quita un rol de la lista activa (sin borrar su nivel ni experiencia aprendida).
    pub fn unassign_role(&mut self, role: ColonyRole) -> bool {
        let before = self.assigned.len();
        self.assigned.retain(|&r| r != role);
        self.assigned.len() != before
    }

    #[inline]
    pub fn level_of(&self, role: ColonyRole) -> i32 {
        self.skills[role.index()].level.clamp(1, MAX_ROLE_LEVEL)
    }

    #[inline]
    pub fn xp_of(&self, role: ColonyRole) -> f32 {
        self.skills[role.index()].xp.max(0.0)
    }

    /// Establece nivel y XP validados (al restaurar desde JSON).
    pub fn set_skill(&mut self, role: ColonyRole, level: i32, xp: f32) {
        let lv = level.clamp(1, MAX_ROLE_LEVEL);
        let max_xp = xp_required_for_next_level(lv);
        let clean_xp = if lv >= MAX_ROLE_LEVEL {
            0.0
        } else {
            xp.clamp(0.0, (max_xp - 0.01).max(0.0))
        };
        self.skills[role.index()] = RoleProgress {
            level: lv,
            xp: clean_xp,
        };
    }

    /// Otorga experiencia en un rol. Cuando el colono está fatigado o en niveles altos,
    /// el progreso se vuelve más lento de forma natural. Devuelve `true` si subió de nivel.
    pub fn grant_xp(&mut self, role: ColonyRole, base_amount: f32, energy: f32) -> bool {
        if base_amount <= 0.0 {
            return false;
        }
        let idx = role.index();
        if self.skills[idx].level >= MAX_ROLE_LEVEL {
            self.skills[idx].xp = 0.0;
            return false;
        }

        // Si el colono está muy cansado aprende más lento; en niveles altos también cuesta más.
        let fatigue_factor = if energy < 35.0 { 0.72 } else { 1.0 };
        let tier_slowdown = if self.skills[idx].level >= 7 {
            0.78
        } else if self.skills[idx].level >= 4 {
            0.90
        } else {
            1.0
        };

        self.skills[idx].xp += base_amount * fatigue_factor * tier_slowdown;

        let mut leveled_up = false;
        while self.skills[idx].level < MAX_ROLE_LEVEL {
            let needed = xp_required_for_next_level(self.skills[idx].level);
            if needed <= 0.0 || self.skills[idx].xp < needed {
                break;
            }
            self.skills[idx].xp -= needed;
            self.skills[idx].level += 1;
            leveled_up = true;
        }

        if self.skills[idx].level >= MAX_ROLE_LEVEL {
            self.skills[idx].level = MAX_ROLE_LEVEL;
            self.skills[idx].xp = 0.0;
        }

        leveled_up
    }

    /// Etiqueta principal del colono según su jerarquía actual.
    pub fn primary_label(&self, is_leader: bool) -> &'static str {
        if is_leader {
            return ColonyRole::ColonyLeader.label_es();
        }
        self.assigned
            .first()
            .map(|r| r.label_es())
            .unwrap_or("Sin rol asignado")
    }

    /// Herramienta por defecto para cuando el colono está en reposo o patrulla.
    pub fn default_tool(&self, is_leader: bool) -> CopperTool {
        if is_leader {
            return CopperTool::CopperSword;
        }
        self.assigned
            .first()
            .map(|r| r.default_tool())
            .unwrap_or(CopperTool::CopperShovel)
    }

    /// Multiplicador de velocidad al talar árboles según el nivel de Leñador.
    #[inline]
    pub fn chop_speed_mult(&self) -> f32 {
        chop_speed_mult(self.level_of(ColonyRole::Lumberjack))
    }

    /// Cantidad de troncos obtenidos por árbol según el nivel de Leñador.
    #[inline]
    pub fn chop_wood_yield(&self) -> i32 {
        chop_wood_yield(self.level_of(ColonyRole::Lumberjack))
    }

    /// Multiplicador de velocidad al minar según el nivel de Minero.
    #[inline]
    pub fn mine_speed_mult(&self) -> f32 {
        mine_speed_mult(self.level_of(ColonyRole::Miner))
    }

    /// Recursos extra obtenidos al picar roca/veta según el nivel de Minero.
    #[inline]
    pub fn mine_bonus_yield(&self) -> i32 {
        mine_bonus_yield(self.level_of(ColonyRole::Miner))
    }

    /// Capacidad máxima de carga por viaje según el nivel de Transportador.
    #[inline]
    pub fn haul_capacity(&self) -> i32 {
        haul_capacity(self.level_of(ColonyRole::Hauler))
    }

    /// Multiplicador de velocidad al caminar cargando ítems según el nivel de Transportador.
    #[inline]
    pub fn haul_speed_mult(&self) -> f32 {
        haul_speed_mult(self.level_of(ColonyRole::Hauler))
    }

    /// Multiplicador de velocidad al construir según el nivel de Constructor.
    #[inline]
    pub fn build_speed_mult(&self) -> f32 {
        build_speed_mult(self.level_of(ColonyRole::Constructor))
    }

    /// Multiplicador de velocidad al arar y cosechar según el nivel de Agricultor.
    #[inline]
    pub fn farm_speed_mult(&self) -> f32 {
        farm_speed_mult(self.level_of(ColonyRole::Farmer))
    }

    /// Rendimiento de cosecha `(zanahorias, semillas)` según el nivel de Agricultor.
    #[inline]
    pub fn farm_harvest_yield(&self) -> (i32, i32) {
        farm_harvest_yield(self.level_of(ColonyRole::Farmer))
    }

    /// Multiplicador de velocidad al sembrar y cuidar plantas según el nivel de Botánico.
    #[inline]
    pub fn botany_speed_mult(&self) -> f32 {
        botany_speed_mult(self.level_of(ColonyRole::Botanist))
    }

    /// Impulso directo al progreso del cultivo al cuidarlo según el nivel de Botánico.
    #[inline]
    pub fn botany_growth_boost(&self) -> f32 {
        botany_growth_boost(self.level_of(ColonyRole::Botanist))
    }

    /// Multiplicador de velocidad al explorar/patrullar según el nivel de Explorador.
    #[inline]
    pub fn scout_speed_mult(&self) -> f32 {
        scout_speed_mult(self.level_of(ColonyRole::Scout))
    }

    /// Radio de patrulla en casillas según el nivel de Explorador.
    #[inline]
    pub fn scout_radius(&self) -> i32 {
        scout_radius(self.level_of(ColonyRole::Scout))
    }

    /// Multiplicador de rapidez en reuniones del Líder de Colonia.
    #[inline]
    pub fn leader_meeting_speed_mult(&self) -> f32 {
        leader_meeting_speed_mult(self.level_of(ColonyRole::ColonyLeader))
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
                format!("Vel. minería x{:.2} • +{} mineral extra", mine_speed_mult(lv), bonus)
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_eight_roles_roundtrip_code_and_label() {
        assert_eq!(ColonyRole::ALL.len(), 8);
        for role in ColonyRole::ALL {
            assert_eq!(ColonyRole::from_code(role.code()), Some(role));
            assert_eq!(ColonyRole::from_label_es(role.label_es()), Some(role));
            assert!(!role.description_es().is_empty());
        }
    }

    #[test]
    fn colonist_can_hold_at_most_five_assigned_roles() {
        let mut profile = ColonistRoleProfile::default();
        assert!(profile.assign_role(ColonyRole::Lumberjack).is_ok());
        assert!(profile.assign_role(ColonyRole::Miner).is_ok());
        assert!(profile.assign_role(ColonyRole::Hauler).is_ok());
        assert!(profile.assign_role(ColonyRole::Constructor).is_ok());
        assert!(profile.assign_role(ColonyRole::Farmer).is_ok());
        // Asignar el mismo rol no falla
        assert!(profile.assign_role(ColonyRole::Miner).is_ok());
        assert_eq!(profile.assigned.len(), 5);

        // El sexto rol distinto debe rechazarse
        assert!(profile.assign_role(ColonyRole::Botanist).is_err());
        assert_eq!(profile.assigned.len(), 5);

        // Quitar uno permite asignar otro preservando límites
        assert!(profile.unassign_role(ColonyRole::Miner));
        assert!(profile.assign_role(ColonyRole::Botanist).is_ok());
        assert_eq!(profile.assigned.len(), 5);
    }

    #[test]
    fn role_levels_scale_gradually_slower_and_cap_at_ten() {
        let mut prev_req = 0.0;
        for lv in 1..MAX_ROLE_LEVEL {
            let req = xp_required_for_next_level(lv);
            assert!(req > prev_req, "Nivel {lv} debe exigir más XP que el anterior");
            prev_req = req;
        }
        assert_eq!(xp_required_for_next_level(MAX_ROLE_LEVEL), 0.0);

        let mut profile = ColonistRoleProfile::default();
        assert_eq!(profile.level_of(ColonyRole::Lumberjack), 1);

        // Subir a nivel 2
        assert!(profile.grant_xp(ColonyRole::Lumberjack, 30.0, 90.0));
        assert_eq!(profile.level_of(ColonyRole::Lumberjack), 2);

        // Subir mucho nunca supera el nivel 10
        profile.grant_xp(ColonyRole::Lumberjack, 50_000.0, 90.0);
        assert_eq!(profile.level_of(ColonyRole::Lumberjack), MAX_ROLE_LEVEL);
        assert_eq!(profile.xp_of(ColonyRole::Lumberjack), 0.0);
        assert!(!profile.grant_xp(ColonyRole::Lumberjack, 100.0, 90.0));
    }

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
    }
}
