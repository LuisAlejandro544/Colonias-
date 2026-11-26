//! Sistema modular de roles, límites de asignación y niveles (1 al 10) para colonos.
//!
//! # Lógica del módulo
//! Define los 10 roles oficiales de **Colonia Centinela** (`ColonyRole`) y el perfil
//! de habilidades de cada colono (`ColonistRoleProfile`), delegando las curvas de
//! experiencia y bonificadores por nivel al submódulo `roles/balance.rs`:
//! - `Leñador`, `Minero`, `Transportador`, `Constructor`, `Agricultor`, `Botánico`,
//!   `Explorador`, `Líder de Colonia`, `Carpintero` y `Masón`.

mod balance;

pub use balance::*;

use crate::tools::CopperTool;

/// Máximo de roles activos que puede tener asignados un colono a la vez.
pub const MAX_ASSIGNED_ROLES: usize = 5;

/// Nivel máximo alcanzable en cualquier rol.
pub const MAX_ROLE_LEVEL: i32 = 10;

/// Los 10 roles disponibles en la colonia.
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
    Carpenter = 8,
    Mason = 9,
}

impl ColonyRole {
    /// Lista ordenada de todos los roles disponibles.
    pub const ALL: [Self; 10] = [
        Self::Lumberjack,
        Self::Miner,
        Self::Hauler,
        Self::Constructor,
        Self::Farmer,
        Self::Botanist,
        Self::Scout,
        Self::ColonyLeader,
        Self::Carpenter,
        Self::Mason,
    ];

    /// Índice fijo (`0..10`) compatible con 32 y 64 bits.
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
            Self::Carpenter => "carpenter",
            Self::Mason => "mason",
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
            Self::Carpenter => "Carpintero",
            Self::Mason => "Masón",
        }
    }

    /// Descripción clara en español de la función del rol.
    pub const fn description_es(self) -> &'static str {
        match self {
            Self::Lumberjack => "Tala árboles con hacha de cobre. Su nivel acelera el corte y aumenta los troncos obtenidos.",
            Self::Miner => "Pica rocas, montañas y vetas minerales. Su nivel acelera la excavación y mejora la extracción de piedra y mena.",
            Self::Hauler => "Recoge ítems del suelo y los guarda en el almacén. Su nivel sube la cantidad transportada por viaje y su rapidez.",
            Self::Constructor => "Construye muros de madera/ladrillo, puertas, mesas, hornos y pozos desde planos. Su nivel acelera la edificación.",
            Self::Farmer => "Ara la tierra y cosecha cultivos maduros. Su nivel acelera la labranza y aumenta la cosecha.",
            Self::Botanist => "Siembra semillas, riega con cubos y cuida cultivos en crecimiento para acelerar su maduración.",
            Self::Scout => "Patrulla e inspecciona un perímetro más amplio. Su nivel incrementa la velocidad de patrulla y el alcance.",
            Self::ColonyLeader => "Reúne al equipo para aprobar órdenes. Su nivel hace más ágiles las reuniones y la coordinación.",
            Self::Carpenter => "Trabaja en la mesa de carpintero para transformar troncos en tablones, cubos, vasos, puertas y arcones según su Inteligencia.",
            Self::Mason => "Labra piedra en ladrillos, morteros, piedras de molino y estatuillas en la Mesa de Masón, y funde cobre con carbón en el Horno.",
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
            "carpenter" => Some(Self::Carpenter),
            "mason" => Some(Self::Mason),
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
            "Carpintero" => Some(Self::Carpenter),
            "Masón" => Some(Self::Mason),
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
            Self::Carpenter => CopperTool::CopperAxe,
            Self::Mason => CopperTool::CopperPickaxe,
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

/// Perfil completo de roles asignados (máx. 5) y niveles (1..=10) de un colono.
#[derive(Clone, Debug)]
pub struct ColonistRoleProfile {
    pub assigned: Vec<ColonyRole>,
    pub skills: [RoleProgress; 10],
}

impl Default for ColonistRoleProfile {
    fn default() -> Self {
        Self {
            assigned: Vec::new(),
            skills: [RoleProgress::default(); 10],
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
                ColonyRole::Carpenter,
                ColonyRole::Hauler,
            ],
            1 => &[
                ColonyRole::Miner,
                ColonyRole::Mason,
                ColonyRole::Botanist,
                ColonyRole::Hauler,
            ],
            2 => &[
                ColonyRole::Farmer,
                ColonyRole::Botanist,
                ColonyRole::Carpenter,
                ColonyRole::Hauler,
            ],
            3 => &[
                ColonyRole::Mason,
                ColonyRole::Constructor,
                ColonyRole::Miner,
                ColonyRole::Lumberjack,
            ],
            4 => &[
                ColonyRole::Scout,
                ColonyRole::Lumberjack,
                ColonyRole::Farmer,
                ColonyRole::Hauler,
            ],
            _ => &[
                ColonyRole::Botanist,
                ColonyRole::Miner,
                ColonyRole::Mason,
                ColonyRole::Carpenter,
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

    #[inline]
    pub fn chop_speed_mult(&self) -> f32 {
        chop_speed_mult(self.level_of(ColonyRole::Lumberjack))
    }

    #[inline]
    pub fn chop_wood_yield(&self) -> i32 {
        chop_wood_yield(self.level_of(ColonyRole::Lumberjack))
    }

    #[inline]
    pub fn mine_speed_mult(&self) -> f32 {
        mine_speed_mult(self.level_of(ColonyRole::Miner))
    }

    #[inline]
    pub fn mine_bonus_yield(&self) -> i32 {
        mine_bonus_yield(self.level_of(ColonyRole::Miner))
    }

    #[inline]
    pub fn haul_capacity(&self) -> i32 {
        haul_capacity(self.level_of(ColonyRole::Hauler))
    }

    #[inline]
    pub fn haul_speed_mult(&self) -> f32 {
        haul_speed_mult(self.level_of(ColonyRole::Hauler))
    }

    #[inline]
    pub fn build_speed_mult(&self) -> f32 {
        build_speed_mult(self.level_of(ColonyRole::Constructor))
    }

    #[inline]
    pub fn farm_speed_mult(&self) -> f32 {
        farm_speed_mult(self.level_of(ColonyRole::Farmer))
    }

    #[inline]
    pub fn farm_harvest_yield(&self) -> (i32, i32) {
        farm_harvest_yield(self.level_of(ColonyRole::Farmer))
    }

    #[inline]
    pub fn botany_speed_mult(&self) -> f32 {
        botany_speed_mult(self.level_of(ColonyRole::Botanist))
    }

    #[inline]
    pub fn botany_growth_boost(&self) -> f32 {
        botany_growth_boost(self.level_of(ColonyRole::Botanist))
    }

    #[inline]
    pub fn scout_speed_mult(&self) -> f32 {
        scout_speed_mult(self.level_of(ColonyRole::Scout))
    }

    #[inline]
    pub fn scout_radius(&self) -> i32 {
        scout_radius(self.level_of(ColonyRole::Scout))
    }

    #[inline]
    pub fn leader_meeting_speed_mult(&self) -> f32 {
        leader_meeting_speed_mult(self.level_of(ColonyRole::ColonyLeader))
    }

    #[inline]
    pub fn carpentry_speed_mult(&self) -> f32 {
        carpentry_speed_mult(self.level_of(ColonyRole::Carpenter))
    }

    #[inline]
    pub fn masonry_speed_mult(&self) -> f32 {
        masonry_speed_mult(self.level_of(ColonyRole::Mason))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ten_roles_roundtrip_code_and_label() {
        assert_eq!(ColonyRole::ALL.len(), 10);
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
        assert!(profile.assign_role(ColonyRole::Miner).is_ok());
        assert_eq!(profile.assigned.len(), 5);

        assert!(profile.assign_role(ColonyRole::Botanist).is_err());
        assert_eq!(profile.assigned.len(), 5);

        assert!(profile.unassign_role(ColonyRole::Miner));
        assert!(profile.assign_role(ColonyRole::Carpenter).is_ok());
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

        assert!(profile.grant_xp(ColonyRole::Lumberjack, 30.0, 90.0));
        assert_eq!(profile.level_of(ColonyRole::Lumberjack), 2);

        profile.grant_xp(ColonyRole::Lumberjack, 50_000.0, 90.0);
        assert_eq!(profile.level_of(ColonyRole::Lumberjack), MAX_ROLE_LEVEL);
        assert_eq!(profile.xp_of(ColonyRole::Lumberjack), 0.0);
        assert!(!profile.grant_xp(ColonyRole::Lumberjack, 100.0, 90.0));
    }
}
