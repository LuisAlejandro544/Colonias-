//! Núcleo de estado y ciclo de actualización principal de la simulación de colonos.
//!
//! # Lógica del módulo
//! Mantiene la estructura central `SimulationCore` (mapa, zonas, cultivos, ítems,
//! órdenes, colonos, reloj y eventos de sonido) junto al arranque inicial de la
//! colonia (`spawn_initial`), la garantía de liderazgo (`ensure_leader_exists`) y
//! el bucle `step` que coordina crecimiento agrícola, reuniones del líder,
//! energía/sueño, asignación de tareas y patrulla/descanso.

use crate::colony_types::{
    CarpentryBenchState, Colonist, ColonistActivity, ColonistInventory, ColonyOrder, CropCell,
    GroundItem, ItemKind, StructureKind, ZoneKind,
};
use crate::noise::SeededRng;
use crate::roles::{ColonistRoleProfile, ColonyRole, MAX_ASSIGNED_ROLES};
use crate::stats::{ColonistStats, StatKind};
use crate::tools::CopperTool;

const FIRST_NAMES: &[&str] = &[
    "Mateo", "Lucía", "Valentina", "Santiago", "Camila", "Diego",
    "Elena", "Andrés", "Sofía", "Tomás", "Valeria", "Nicolás",
    "Clara", "Gabriel", "Paula", "Hugo",
];

const SKIN_HEX: &[&str] = &[
    "#f2d2b6", "#e4b892", "#d19b6c", "#b77d50", "#8d5937",
];

const HAIR_HEX: &[&str] = &[
    "#2c221e", "#5b3b28", "#8f542e", "#c99b52", "#7c756d",
];

const OUTFIT_HEX: &[&str] = &[
    "#4f6b47", "#8f4e37", "#48637a", "#9c7b3e", "#6d526e", "#5e6a4f",
];

pub(super) const WOOD_PER_WALL: i32 = 2;
pub(super) const WOOD_PER_CARPENTER_BENCH: i32 = 6;
pub(super) const DOOR_ITEMS_PER_DOOR: i32 = 1;

/// Estado completo de la simulación en Rust (independiente de Godot para unit tests).
pub struct SimulationCore {
    pub map_width: i32,
    pub map_height: i32,
    pub ground: Vec<i32>,
    pub props: Vec<i32>,
    pub blocked: Vec<i32>,
    pub zones: Vec<i32>,
    pub structures: Vec<i32>,
    pub crops: Vec<CropCell>,
    pub items: Vec<GroundItem>,
    pub orders: Vec<ColonyOrder>,
    pub benches: Vec<CarpentryBenchState>,
    pub colonists: Vec<Colonist>,
    pub rng: SeededRng,
    pub next_item_id: i32,
    pub next_order_id: i32,
    pub next_bench_id: i32,
    pub briefing_pending: bool,
    pub storage_approved: bool,
    pub farm_approved: bool,
    pub meeting_active: bool,
    pub meeting_timer: f32,
    pub meeting_approach_timer: f32,
    pub meeting_target_id: i32,
    pub world_dirty: bool,
    pub day_clock: f32,
    pub day_count: i32,
    pub pending_sfx: Vec<&'static str>,
}

impl SimulationCore {
    pub fn new(
        seed: u64,
        width: i32,
        height: i32,
        ground: Vec<i32>,
        props: Vec<i32>,
        blocked: Vec<i32>,
    ) -> Self {
        let w = width.max(16);
        let h = height.max(16);
        let total = (w * h) as usize;
        let mut g = ground;
        if g.len() != total {
            g.resize(total, 0);
        }
        let mut p = props;
        if p.len() != total {
            p.resize(total, 0);
        }
        let mut b = blocked;
        if b.len() != total {
            b.resize(total, 0);
        }

        Self {
            map_width: w,
            map_height: h,
            ground: g,
            props: p,
            blocked: b,
            zones: vec![ZoneKind::None as i32; total],
            structures: vec![StructureKind::None as i32; total],
            crops: Vec::new(),
            items: Vec::new(),
            orders: Vec::new(),
            benches: Vec::new(),
            colonists: Vec::new(),
            rng: SeededRng::new(seed ^ 0xC010_414A_0001_0002),
            next_item_id: 1,
            next_order_id: 1,
            next_bench_id: 1,
            briefing_pending: false,
            storage_approved: true,
            farm_approved: true,
            meeting_active: false,
            meeting_timer: 0.0,
            meeting_approach_timer: 0.0,
            meeting_target_id: -1,
            world_dirty: true,
            day_clock: 0.0,
            day_count: 1,
            pending_sfx: Vec::new(),
        }
    }

    #[inline]
    pub(super) fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.map_width || y >= self.map_height {
            return None;
        }
        Some((y * self.map_width + x) as usize)
    }

    pub(super) fn tile_blocked(
        map_width: i32,
        map_height: i32,
        blocked: &[i32],
        tx: i32,
        ty: i32,
    ) -> bool {
        if tx < 1 || ty < 1 || tx >= map_width - 1 || ty >= map_height - 1 {
            return true;
        }
        let idx = (ty * map_width + tx) as usize;
        blocked.get(idx).copied().unwrap_or(1) != 0
    }

    /// Crea los colonos iniciales, elige aleatoriamente un Líder de Colonia y deja
    /// suministros iniciales de semillas y madera en el claro central.
    pub fn spawn_initial(&mut self, count: i32) {
        self.colonists.clear();
        let total = count.clamp(1, 10);
        let cx = self.map_width as f32 * 0.5;
        let cy = self.map_height as f32 * 0.5;

        let leader_idx = self.rng.next_usize(total as usize);

        for i in 0..total {
            let angle = (i as f32 / total as f32) * std::f32::consts::TAU;
            let radius = 1.6 + (i % 2) as f32 * 0.9;
            let sx = (cx + angle.cos() * radius).clamp(2.0, (self.map_width - 3) as f32);
            let sy = (cy + angle.sin() * radius).clamp(2.0, (self.map_height - 3) as f32);

            let is_leader = i as usize == leader_idx;
            let name_idx =
                (self.rng.next_usize(FIRST_NAMES.len()) + i as usize) % FIRST_NAMES.len();
            let role_profile = ColonistRoleProfile::new_for_colonist(i as usize, is_leader);
            let role_str = role_profile.primary_label(is_leader).to_string();
            let stats = ColonistStats::generate_for_colonist(&mut self.rng, i, is_leader);
            let max_health = stats.max_health();
            let skin_idx = self.rng.next_usize(SKIN_HEX.len());
            let hair_idx = self.rng.next_usize(HAIR_HEX.len());
            let outfit_idx = self.rng.next_usize(OUTFIT_HEX.len());

            let inventory = ColonistInventory::default_for_role(&role_str, is_leader);

            self.colonists.push(Colonist {
                id: i + 1,
                name: FIRST_NAMES[name_idx].to_string(),
                role: role_str,
                is_leader,
                x: sx,
                y: sy,
                target_x: sx,
                target_y: sy,
                path: Vec::new(),
                facing_x: 0.0,
                facing_y: 1.0,
                speed: 2.35 + self.rng.next_f32() * 0.35,
                health: max_health,
                max_health,
                mood: (84 + self.rng.range_i32(-6, 10) + if is_leader { 4 } else { 0 })
                    .clamp(50, 100),
                energy: 86.0 + (i as f32 * 3.0).min(14.0),
                wait_timer: 0.3 + self.rng.next_f32() * 1.2,
                walk_phase: 0.0,
                work_swing_phase: 0.0,
                body_style: self.rng.range_i32(0, 1),
                hair_style: self.rng.range_i32(0, 2),
                skin_color: SKIN_HEX[skin_idx].to_string(),
                hair_color: HAIR_HEX[hair_idx].to_string(),
                outfit_color: OUTFIT_HEX[outfit_idx].to_string(),
                activity: ColonistActivity::Resting,
                role_profile,
                stats,
                inventory,
                carrying_kind: None,
                carrying_amount: 0,
                active_task: None,
            });
        }

        // Garantizar que al menos un colono inicial alcance INT >= 4 para que la colonia
        // pueda fabricar todas las recetas de carpintería desde el primer asentamiento.
        if !self.colonists.is_empty() && !self.colonists.iter().any(|c| c.stats.intelligence >= 4) {
            if let Some(best_idx) = self
                .colonists
                .iter()
                .enumerate()
                .max_by_key(|(_, c)| c.stats.intelligence)
                .map(|(idx, _)| idx)
            {
                self.colonists[best_idx].stats.set(StatKind::Intelligence, 4);
            }
        }

        // Suministro inicial en el claro: semillas de zanahoria y troncos para el campamento.
        let center_tx = (self.map_width / 2).clamp(2, self.map_width - 3);
        let center_ty = (self.map_height / 2).clamp(2, self.map_height - 3);
        self.spawn_item(ItemKind::CarrotSeed, center_tx - 1, center_ty, 16, false);
        self.spawn_item(ItemKind::WoodLogs, center_tx + 1, center_ty, 6, false);
        self.world_dirty = true;
    }

    pub fn ensure_leader_exists(&mut self) {
        if self.colonists.is_empty() {
            return;
        }
        if !self.colonists.iter().any(|c| c.is_leader) {
            let existing_role_holder = self
                .colonists
                .iter()
                .position(|c| c.role_profile.has_role(ColonyRole::ColonyLeader));
            let idx = existing_role_holder
                .unwrap_or_else(|| self.rng.next_usize(self.colonists.len()));
            self.colonists[idx].is_leader = true;
            if !self.colonists[idx]
                .role_profile
                .has_role(ColonyRole::ColonyLeader)
            {
                if self.colonists[idx].role_profile.assigned.len() >= MAX_ASSIGNED_ROLES {
                    self.colonists[idx].role_profile.assigned.pop();
                }
                let _ = self.colonists[idx]
                    .role_profile
                    .assign_role(ColonyRole::ColonyLeader);
            }
            self.colonists[idx].role = self.colonists[idx]
                .role_profile
                .primary_label(true)
                .to_string();
            self.colonists[idx]
                .inventory
                .equip_main_hand(CopperTool::CopperSword);
        }
    }

    /// Avanza un paso de simulación para el reloj, crecimiento de zanahorias, reuniones del líder y colonos.
    pub fn step(&mut self, delta: f32) {
        let dt = delta.clamp(0.0, 0.1);
        if dt <= 0.0 {
            return;
        }

        self.day_clock += dt;
        if self.day_clock >= 90.0 {
            self.day_clock -= 90.0;
            self.day_count += 1;
        }

        self.step_crops_growth(dt);
        self.update_leader_briefing(dt);

        for c_idx in 0..self.colonists.len() {
            if self.colonists[c_idx].activity == ColonistActivity::ConveningMeeting
                || self.colonists[c_idx].activity == ColonistActivity::AttendingMeeting
            {
                if !self.colonists[c_idx].path.is_empty() {
                    self.step_colonist_movement(c_idx, dt);
                }
                continue;
            }

            if self.colonists[c_idx].activity == ColonistActivity::Sleeping {
                self.colonists[c_idx].inventory.stow_main_hand();
                self.colonists[c_idx].energy =
                    (self.colonists[c_idx].energy + 14.0 * dt).min(100.0);
                if self.colonists[c_idx].energy >= 96.0 {
                    self.colonists[c_idx].activity = ColonistActivity::Resting;
                    self.colonists[c_idx].wait_timer = 0.2;
                    let is_leader = self.colonists[c_idx].is_leader;
                    let default_tool = self.colonists[c_idx].role_profile.default_tool(is_leader);
                    self.colonists[c_idx].inventory.equip_main_hand(default_tool);
                }
                continue;
            }

            let base_drain = match self.colonists[c_idx].activity {
                ColonistActivity::Chopping
                | ColonistActivity::Mining
                | ColonistActivity::Building
                | ColonistActivity::Carpentry
                | ColonistActivity::Farming
                | ColonistActivity::Botanizing => 2.0,
                ColonistActivity::MovingToTask
                | ColonistActivity::Hauling
                | ColonistActivity::Wandering
                | ColonistActivity::Scouting => 0.95,
                _ => 0.25,
            };
            let drain_rate = base_drain * self.colonists[c_idx].stats.energy_drain_mult();
            self.colonists[c_idx].energy =
                (self.colonists[c_idx].energy - drain_rate * dt).max(0.0);

            if self.colonists[c_idx].energy <= 16.0
                && !self.meeting_active
                && self.colonists[c_idx].carrying_kind.is_none()
            {
                self.release_colonist_task(c_idx);
                self.colonists[c_idx].activity = ColonistActivity::Sleeping;
                self.colonists[c_idx].inventory.stow_main_hand();
                continue;
            }

            if self.colonists[c_idx].active_task.is_some() {
                self.execute_active_task(c_idx, dt);
                continue;
            }

            self.assign_task_to_colonist(c_idx);
            if self.colonists[c_idx].active_task.is_some() {
                continue;
            }

            self.step_colonist_idle_or_patrol(c_idx, dt);
        }
    }

    fn step_colonist_idle_or_patrol(&mut self, c_idx: usize, dt: f32) {
        let is_leader = self.colonists[c_idx].is_leader;
        let has_scout = self.colonists[c_idx].role_profile.has_role(ColonyRole::Scout);
        let default_tool = self.colonists[c_idx].role_profile.default_tool(is_leader);
        self.colonists[c_idx].inventory.equip_main_hand(default_tool);

        if self.colonists[c_idx].wait_timer > 0.0 {
            self.colonists[c_idx].wait_timer -= dt;
            self.colonists[c_idx].activity = ColonistActivity::Resting;
            self.colonists[c_idx].facing_x = 0.0;
            self.colonists[c_idx].facing_y = 1.0;
            return;
        }

        let dx = self.colonists[c_idx].target_x - self.colonists[c_idx].x;
        let dy = self.colonists[c_idx].target_y - self.colonists[c_idx].y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 0.12 {
            let mut picked = false;
            let radius = if has_scout {
                self.colonists[c_idx].role_profile.scout_radius()
            } else {
                6
            };
            for _ in 0..8 {
                let ox = self.rng.range_i32(-radius, radius);
                let oy = self.rng.range_i32(-radius, radius);
                let nx = (self.colonists[c_idx].x.round() as i32 + ox)
                    .clamp(2, self.map_width - 3);
                let ny = (self.colonists[c_idx].y.round() as i32 + oy)
                    .clamp(2, self.map_height - 3);
                if !Self::tile_blocked(self.map_width, self.map_height, &self.blocked, nx, ny) {
                    self.colonists[c_idx].target_x = nx as f32 + 0.5;
                    self.colonists[c_idx].target_y = ny as f32 + 0.5;
                    self.colonists[c_idx].activity = if has_scout && self.rng.next_f32() < 0.78 {
                        ColonistActivity::Scouting
                    } else if self.rng.next_f32() < 0.30 {
                        ColonistActivity::Scouting
                    } else {
                        ColonistActivity::Wandering
                    };
                    picked = true;
                    break;
                }
            }
            self.colonists[c_idx].wait_timer = if !picked {
                1.0
            } else if self.rng.next_f32() < 0.45 {
                0.9 + self.rng.next_f32() * 1.8
            } else {
                0.0
            };
            return;
        }

        let nx = dx / dist;
        let ny = dy / dist;
        let patrol_mult =
            if self.colonists[c_idx].activity == ColonistActivity::Scouting && has_scout {
                self.colonists[c_idx].role_profile.scout_speed_mult()
            } else {
                1.0
            };
        let dex_mult = self.colonists[c_idx].stats.move_speed_mult();
        let effective_speed = self.colonists[c_idx].speed * patrol_mult * dex_mult;
        let step_dist = (effective_speed * dt).min(dist);
        let next_x = self.colonists[c_idx].x + nx * step_dist;
        let next_y = self.colonists[c_idx].y + ny * step_dist;
        let tile_x = next_x.floor() as i32;
        let tile_y = next_y.floor() as i32;

        if Self::tile_blocked(self.map_width, self.map_height, &self.blocked, tile_x, tile_y) {
            self.colonists[c_idx].target_x = self.colonists[c_idx].x;
            self.colonists[c_idx].target_y = self.colonists[c_idx].y;
            self.colonists[c_idx].wait_timer = 0.5;
            self.colonists[c_idx].activity = ColonistActivity::Resting;
        } else {
            self.colonists[c_idx].x = next_x;
            self.colonists[c_idx].y = next_y;
            self.colonists[c_idx].facing_x = nx;
            self.colonists[c_idx].facing_y = ny;
            self.colonists[c_idx].walk_phase += dt * effective_speed * 4.5;
            if self.colonists[c_idx].activity == ColonistActivity::Scouting && has_scout {
                let energy = self.colonists[c_idx].energy;
                let xp_mult = self.colonists[c_idx].stats.xp_gain_mult();
                self.colonists[c_idx]
                    .role_profile
                    .grant_xp(ColonyRole::Scout, dt * 3.5 * xp_mult, energy);
                self.colonists[c_idx]
                    .stats
                    .grant_stat_xp(StatKind::Endurance, dt * 0.6);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colony_types::OrderStatus;

    fn empty_core(seed: u64, w: i32, h: i32) -> SimulationCore {
        let total = (w * h) as usize;
        SimulationCore::new(seed, w, h, vec![0; total], vec![0; total], vec![0; total])
    }

    #[test]
    fn spawns_colonists_with_exact_one_random_leader() {
        let mut core = empty_core(1234, 32, 32);
        core.spawn_initial(5);
        assert_eq!(core.colonists.len(), 5);
        let leaders = core.colonists.iter().filter(|c| c.is_leader).count();
        assert_eq!(leaders, 1);
    }

    #[test]
    fn orders_wait_for_leader_briefing_before_execution() {
        let mut core = empty_core(42, 32, 32);
        let idx = (16 * 32 + 20) as usize;
        core.props[idx] = 1; // OakTree
        core.blocked[idx] = 1;
        core.spawn_initial(3);

        let count = core.designate_area("chop", 20, 16, 20, 16);
        assert_eq!(count, 1);
        assert_eq!(core.orders[0].status, OrderStatus::PendingBriefing);

        for _ in 0..160 {
            core.step(0.05);
        }
        assert!(core.orders.is_empty() || core.orders[0].status == OrderStatus::Approved);
    }

    #[test]
    fn chopping_tree_drops_wood_logs_and_storage_hauls_them() {
        let mut core = empty_core(99, 24, 24);
        let tree_idx = (12 * 24 + 14) as usize;
        core.props[tree_idx] = 1;
        core.blocked[tree_idx] = 1;
        core.spawn_initial(2);

        core.designate_area("chop", 14, 12, 14, 12);
        core.designate_area("storage", 10, 12, 11, 12);

        for _ in 0..350 {
            core.step(0.05);
        }

        assert_eq!(core.props[tree_idx], 0);
        let (total_inv, _) = core.compute_inventory();
        assert!(total_inv.wood >= 10);
    }

    #[test]
    fn farming_zone_tills_seeds_grows_and_harvests_carrots() {
        let mut core = empty_core(77, 24, 24);
        core.spawn_initial(3);
        core.designate_area("farm", 12, 10, 12, 10);

        for _ in 0..750 {
            core.step(0.05);
        }

        let (total_inv, _) = core.compute_inventory();
        assert!(total_inv.carrots >= 3);
    }

    #[test]
    fn wooden_wall_consumes_wood_and_blocks_tile() {
        let mut core = empty_core(55, 24, 24);
        core.spawn_initial(2);
        core.designate_area("build_wall", 14, 12, 14, 12);

        for _ in 0..300 {
            core.step(0.05);
        }

        let idx = (12 * 24 + 14) as usize;
        assert_eq!(core.structures[idx], StructureKind::WoodWall as i32);
        assert_eq!(core.blocked[idx], 1);
        assert!(core.pending_sfx.contains(&"build"));
    }

    #[test]
    fn colonist_sleeps_when_tired_and_recovers_energy() {
        let mut core = empty_core(101, 24, 24);
        core.spawn_initial(2);
        core.colonists[0].energy = 14.0;
        core.step(0.05);
        assert_eq!(core.colonists[0].activity, ColonistActivity::Sleeping);
        assert_eq!(core.colonists[0].inventory.main_hand, CopperTool::None);
        assert_ne!(core.colonists[0].inventory.back_tool, CopperTool::None);

        for _ in 0..160 {
            core.step(0.05);
        }
        assert!(core.colonists[0].energy >= 95.0);
        assert_ne!(core.colonists[0].activity, ColonistActivity::Sleeping);
    }

    #[test]
    fn colonist_inventory_equips_back_and_main_hand_slots() {
        let mut inv = ColonistInventory::default_for_role("Minero", false);
        assert!(inv.helmet_slot.is_empty());
        assert!(inv.armor_slot.is_empty());
        assert_eq!(inv.main_hand, CopperTool::CopperPickaxe);
        assert_eq!(inv.back_tool, CopperTool::CopperShovel);

        inv.equip_main_hand(CopperTool::CopperShovel);
        assert_eq!(inv.main_hand, CopperTool::CopperShovel);
        assert_eq!(inv.back_tool, CopperTool::CopperPickaxe);

        inv.stow_main_hand();
        assert_eq!(inv.main_hand, CopperTool::None);
        assert_eq!(inv.back_tool, CopperTool::CopperPickaxe);
    }

    #[test]
    fn role_assignment_enforces_five_role_cap_and_leader_transfer() {
        let mut core = empty_core(202, 24, 24);
        core.spawn_initial(3);
        let c_id = core.colonists[0].id;
        core.colonists[0].role_profile.assigned.clear();

        for r in [
            ColonyRole::Lumberjack,
            ColonyRole::Miner,
            ColonyRole::Hauler,
            ColonyRole::Constructor,
            ColonyRole::Farmer,
        ] {
            assert!(core.set_colonist_role_enabled(c_id, r, true).is_ok());
        }
        assert_eq!(core.colonists[0].role_profile.assigned.len(), 5);
        assert!(core
            .set_colonist_role_enabled(c_id, ColonyRole::Botanist, true)
            .is_err());

        let target_idx = core.colonists.iter().position(|c| !c.is_leader).unwrap();
        let target_id = core.colonists[target_idx].id;
        let first_role = core.colonists[target_idx].role_profile.assigned[0];
        let _ = core.set_colonist_role_enabled(target_id, first_role, false);
        assert!(core
            .set_colonist_role_enabled(target_id, ColonyRole::ColonyLeader, true)
            .is_ok());
        assert!(core.colonists[target_idx].is_leader);
        assert_eq!(core.colonists.iter().filter(|c| c.is_leader).count(), 1);
    }

    #[test]
    fn hauler_level_controls_carry_amount_per_trip() {
        let mut core = empty_core(303, 24, 24);
        core.spawn_initial(2);
        core.colonists[0].role_profile.assigned = vec![ColonyRole::Hauler];
        core.colonists[0].role_profile.set_skill(ColonyRole::Hauler, 1, 0.0);
        core.colonists[0].stats.set(StatKind::Strength, 3);
        core.colonists[1].role_profile.assigned = vec![ColonyRole::ColonyLeader];

        core.designate_area("storage", 8, 12, 9, 12);
        core.approve_all_pending();
        core.spawn_item(ItemKind::StoneChunk, 14, 12, 6, false);

        for _ in 0..80 {
            core.step(0.05);
            if core.colonists[0].carrying_amount > 0 {
                break;
            }
        }
        assert_eq!(core.colonists[0].carrying_amount, 2);

        // Con Fuerza 7 (+2 carga), carga 4 en vez de 2.
        core.colonists[0].carrying_kind = None;
        core.colonists[0].carrying_amount = 0;
        core.colonists[0].active_task = None;
        core.colonists[0].stats.set(StatKind::Strength, 7);
        core.spawn_item(ItemKind::StoneChunk, 14, 13, 6, false);

        for _ in 0..80 {
            core.step(0.05);
            if core.colonists[0].carrying_amount > 0 {
                break;
            }
        }
        assert_eq!(core.colonists[0].carrying_amount, 4);
    }

    #[test]
    fn carpenter_bench_builds_and_respects_intelligence_requirement() {
        use crate::colony_types::CarpentryRecipe;

        let mut core = empty_core(404, 24, 24);
        core.spawn_initial(2);
        core.colonists[0].role_profile.assigned =
            vec![ColonyRole::Constructor, ColonyRole::Carpenter];
        core.colonists[0].stats.set(StatKind::Intelligence, 1);
        core.colonists[1].role_profile.assigned = vec![ColonyRole::ColonyLeader];

        core.designate_area("build_carpenter_bench", 14, 12, 14, 12);
        core.approve_all_pending();

        for _ in 0..220 {
            core.step(0.05);
        }

        let bench_idx = (12 * 24 + 14) as usize;
        assert_eq!(core.structures[bench_idx], StructureKind::CarpenterBench as i32);
        assert_eq!(core.blocked[bench_idx], 1);
        assert_eq!(core.benches.len(), 1);
        let bench_id = core.benches[0].id;

        // Añadir madera para tablones y encolar Puerta (INT 3) + Tablones (INT 1).
        core.spawn_item(ItemKind::WoodLogs, 13, 12, 4, false);
        core.spawn_item(ItemKind::WoodPlank, 13, 13, 4, false);
        assert!(core.queue_carpentry_recipe(bench_id, CarpentryRecipe::Door).is_ok());
        assert!(core.queue_carpentry_recipe(bench_id, CarpentryRecipe::Planks).is_ok());

        for _ in 0..160 {
            core.step(0.05);
        }

        // Con INT 1, debe haber fabricado los Tablones y dejado la Puerta pendiente.
        assert_eq!(core.benches[0].queue, vec![CarpentryRecipe::Door]);
        let (inv_low, _) = core.compute_inventory();
        assert_eq!(inv_low.wood_planks, 8);
        assert_eq!(inv_low.wood_doors, 0);

        // Subir Inteligencia a 4 -> ahora sí fabrica Puerta y Arcón.
        core.colonists[0].stats.set(StatKind::Intelligence, 4);
        assert!(core.queue_carpentry_recipe(bench_id, CarpentryRecipe::Crate).is_ok());
        core.spawn_item(ItemKind::WoodPlank, 13, 13, 6, false);

        for _ in 0..320 {
            core.step(0.05);
        }

        assert!(core.benches[0].queue.is_empty());
        let (inv_high, _) = core.compute_inventory();
        assert_eq!(inv_high.wood_doors, 1);
        assert_eq!(inv_high.wood_crates, 1);
    }

    #[test]
    fn wooden_door_installs_from_door_item_and_stays_walkable() {
        let mut core = empty_core(505, 24, 24);
        core.spawn_initial(2);
        core.colonists[0].role_profile.assigned = vec![ColonyRole::Constructor];
        core.colonists[1].role_profile.assigned = vec![ColonyRole::ColonyLeader];

        core.designate_area("build_door", 14, 12, 14, 12);
        core.approve_all_pending();

        // Sin ítem de puerta en el suelo, la puerta queda en plano.
        for _ in 0..60 {
            core.step(0.05);
        }
        let door_idx = (12 * 24 + 14) as usize;
        assert_eq!(core.structures[door_idx], StructureKind::WoodDoorBlueprint as i32);

        // Al aparecer una hoja de puerta fabricada, el Constructor la instala sin bloquear paso.
        core.spawn_item(ItemKind::WoodDoorItem, 13, 12, 1, false);
        for _ in 0..160 {
            core.step(0.05);
        }
        assert_eq!(core.structures[door_idx], StructureKind::WoodDoor as i32);
        assert_eq!(core.blocked[door_idx], 0);
    }
}
