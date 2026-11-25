//! Simulación modular de colonos terrestres en vista centinela (estilo RimWorld).
//!
//! # Lógica del módulo
//! Administra la colonia completa en Rust:
//! - Elección aleatoria determinista de un **Líder de Colonia**.
//! - Sistema de **reuniones y trato personal**: las órdenes marcadas por el jugador
//!   no se ejecutan mágicamente; el líder reúne o se acerca a los colonos para
//!   transmitir las instrucciones antes de que trabajen.
//! - Tala de árboles (Hacha de cobre) y minería de rocas/vetas (Pico de cobre)
//!   soltando recursos físicos al piso.
//! - Zona de almacenamiento para recoger y guardar recursos sueltos.
//! - Zona de construcción con madera para levantar bloques/paredes de casa.
//! - Zona de cultivo exclusivo de zanahorias (Azada de cobre, siembra, crecimiento
//!   por etapas y cosecha).
//! - Navegación A* sobre obstáculos dinámicos y serialización completa para JSON.

use godot::classes::{INode, Node};
use godot::prelude::*;

use crate::colony_types::{
    ActiveTask, Colonist, ColonistActivity, ColonistInventory, ColonyInventory, ColonyOrder,
    CropCell, CropStage, GroundItem, ItemKind, OrderKind, OrderStatus, StructureKind, ZoneKind,
};
use crate::noise::SeededRng;
use crate::pathfinding::find_path;
use crate::roles::{
    effect_summary_es, xp_required_for_next_level, ColonistRoleProfile, ColonyRole,
    MAX_ASSIGNED_ROLES, MAX_ROLE_LEVEL,
};
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

const WOOD_PER_WALL: i32 = 2;

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
    pub colonists: Vec<Colonist>,
    pub rng: SeededRng,
    pub next_item_id: i32,
    pub next_order_id: i32,
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
            colonists: Vec::new(),
            rng: SeededRng::new(seed ^ 0xC010_414A_0001_0002),
            next_item_id: 1,
            next_order_id: 1,
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
    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.map_width || y >= self.map_height {
            return None;
        }
        Some((y * self.map_width + x) as usize)
    }

    fn tile_blocked(map_width: i32, map_height: i32, blocked: &[i32], tx: i32, ty: i32) -> bool {
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
            let name_idx = (self.rng.next_usize(FIRST_NAMES.len()) + i as usize) % FIRST_NAMES.len();
            let role_profile = ColonistRoleProfile::new_for_colonist(i as usize, is_leader);
            let role_str = role_profile.primary_label(is_leader).to_string();
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
                health: 100,
                mood: (84 + self.rng.range_i32(-6, 10) + if is_leader { 4 } else { 0 }).clamp(50, 100),
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
                inventory,
                carrying_kind: None,
                carrying_amount: 0,
                active_task: None,
            });
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
            let idx = existing_role_holder.unwrap_or_else(|| self.rng.next_usize(self.colonists.len()));
            self.colonists[idx].is_leader = true;
            if !self.colonists[idx].role_profile.has_role(ColonyRole::ColonyLeader) {
                if self.colonists[idx].role_profile.assigned.len() >= MAX_ASSIGNED_ROLES {
                    self.colonists[idx].role_profile.assigned.pop();
                }
                let _ = self.colonists[idx]
                    .role_profile
                    .assign_role(ColonyRole::ColonyLeader);
            }
            self.colonists[idx].role = self.colonists[idx].role_profile.primary_label(true).to_string();
            self.colonists[idx].inventory.equip_main_hand(CopperTool::CopperSword);
        }
    }

    pub fn spawn_item(&mut self, kind: ItemKind, x: i32, y: i32, amount: i32, stored: bool) {
        if amount <= 0 {
            return;
        }
        // Si ya existe un montón del mismo tipo en la misma casilla, apilarlo.
        if let Some(existing) = self
            .items
            .iter_mut()
            .find(|it| it.x == x && it.y == y && it.kind == kind)
        {
            existing.amount += amount;
            existing.stored = existing.stored || stored;
            self.world_dirty = true;
            return;
        }
        let id = self.next_item_id;
        self.next_item_id += 1;
        self.items.push(GroundItem {
            id,
            kind,
            x,
            y,
            amount,
            stored,
        });
        self.world_dirty = true;
    }

    /// Calcula el inventario actual contabilizando todos los recursos presentes en el mapa
    /// (almacenados + sueltos en el piso) y diferenciando los almacenados.
    pub fn compute_inventory(&self) -> (ColonyInventory, ColonyInventory) {
        let mut total = ColonyInventory::default();
        let mut stored = ColonyInventory::default();
        for it in &self.items {
            match it.kind {
                ItemKind::WoodLogs => {
                    total.wood += it.amount;
                    if it.stored {
                        stored.wood += it.amount;
                    }
                }
                ItemKind::StoneChunk => {
                    total.stone += it.amount;
                    if it.stored {
                        stored.stone += it.amount;
                    }
                }
                ItemKind::IronOre => {
                    total.iron += it.amount;
                    if it.stored {
                        stored.iron += it.amount;
                    }
                }
                ItemKind::CopperOre => {
                    total.copper += it.amount;
                    if it.stored {
                        stored.copper += it.amount;
                    }
                }
                ItemKind::CarrotSeed => {
                    total.carrot_seeds += it.amount;
                    if it.stored {
                        stored.carrot_seeds += it.amount;
                    }
                }
                ItemKind::Carrot => {
                    total.carrots += it.amount;
                    if it.stored {
                        stored.carrots += it.amount;
                    }
                }
            }
        }
        for c in &self.colonists {
            if let Some(k) = c.carrying_kind {
                match k {
                    ItemKind::WoodLogs => total.wood += c.carrying_amount,
                    ItemKind::StoneChunk => total.stone += c.carrying_amount,
                    ItemKind::IronOre => total.iron += c.carrying_amount,
                    ItemKind::CopperOre => total.copper += c.carrying_amount,
                    ItemKind::CarrotSeed => total.carrot_seeds += c.carrying_amount,
                    ItemKind::Carrot => total.carrots += c.carrying_amount,
                }
            }
        }
        (total, stored)
    }

    fn consume_item_from_world(&mut self, kind: ItemKind, mut needed: i32) -> bool {
        let available: i32 = self
            .items
            .iter()
            .filter(|i| i.kind == kind)
            .map(|i| i.amount)
            .sum();
        if available < needed {
            return false;
        }
        // Priorizar consumir del almacén primero, luego del suelo.
        for stored_first in [true, false] {
            for item in &mut self.items {
                if needed <= 0 {
                    break;
                }
                if item.kind == kind && item.stored == stored_first && item.amount > 0 {
                    let take = item.amount.min(needed);
                    item.amount -= take;
                    needed -= take;
                }
            }
        }
        self.items.retain(|i| i.amount > 0);
        self.world_dirty = true;
        true
    }

    /// Designa un rectángulo de órdenes o zonas sobre el mapa.
    /// Devuelve la cantidad de casillas afectadas.
    pub fn designate_area(&mut self, mode: &str, x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
        let min_x = x0.min(x1).clamp(1, self.map_width - 2);
        let max_x = x0.max(x1).clamp(1, self.map_width - 2);
        let min_y = y0.min(y1).clamp(1, self.map_height - 2);
        let max_y = y0.max(y1).clamp(1, self.map_height - 2);

        let mut affected = 0i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let Some(idx) = self.idx(x, y) else {
                    continue;
                };
                match mode {
                    "chop" => {
                        let p = self.props[idx];
                        if (p == 1 || p == 2)
                            && !self
                                .orders
                                .iter()
                                .any(|o| o.x == x && o.y == y && o.kind == OrderKind::ChopTree)
                        {
                            let id = self.next_order_id;
                            self.next_order_id += 1;
                            self.orders.push(ColonyOrder {
                                id,
                                kind: OrderKind::ChopTree,
                                x,
                                y,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "mine" => {
                        let p = self.props[idx];
                        if (p == 4 || p == 5)
                            && !self
                                .orders
                                .iter()
                                .any(|o| o.x == x && o.y == y && o.kind == OrderKind::MineRock)
                        {
                            let id = self.next_order_id;
                            self.next_order_id += 1;
                            self.orders.push(ColonyOrder {
                                id,
                                kind: OrderKind::MineRock,
                                x,
                                y,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "storage" => {
                        if self.blocked[idx] == 0
                            && self.structures[idx] == StructureKind::None as i32
                            && self.zones[idx] != ZoneKind::Storage as i32
                        {
                            self.zones[idx] = ZoneKind::Storage as i32;
                            self.crops.retain(|c| !(c.x == x && c.y == y));
                            // Actualizar items que ya estaban en esa casilla como almacenados
                            for it in &mut self.items {
                                if it.x == x && it.y == y {
                                    it.stored = true;
                                }
                            }
                            self.storage_approved = false;
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "farm" => {
                        let ground_kind = self.ground[idx];
                        // Se puede cultivar sobre tierra habitable no bloqueada (no piedra sólida)
                        if self.blocked[idx] == 0
                            && self.structures[idx] == StructureKind::None as i32
                            && ground_kind != 4
                            && self.zones[idx] != ZoneKind::Farm as i32
                        {
                            self.zones[idx] = ZoneKind::Farm as i32;
                            if !self.crops.iter().any(|c| c.x == x && c.y == y) {
                                self.crops.push(CropCell {
                                    x,
                                    y,
                                    stage: CropStage::None,
                                    progress: 0.0,
                                    tend_cooldown: 0.0,
                                });
                            }
                            self.farm_approved = false;
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "build_wall" => {
                        if self.blocked[idx] == 0
                            && self.structures[idx] == StructureKind::None as i32
                            && !self
                                .orders
                                .iter()
                                .any(|o| o.x == x && o.y == y && o.kind == OrderKind::BuildWoodWall)
                        {
                            self.zones[idx] = ZoneKind::None as i32;
                            self.crops.retain(|c| !(c.x == x && c.y == y));
                            self.structures[idx] = StructureKind::WoodWallBlueprint as i32;
                            let id = self.next_order_id;
                            self.next_order_id += 1;
                            self.orders.push(ColonyOrder {
                                id,
                                kind: OrderKind::BuildWoodWall,
                                x,
                                y,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "cancel" => {
                        let mut changed = false;
                        if self.zones[idx] != ZoneKind::None as i32 {
                            self.zones[idx] = ZoneKind::None as i32;
                            self.crops.retain(|c| !(c.x == x && c.y == y));
                            changed = true;
                        }
                        if self.structures[idx] == StructureKind::WoodWallBlueprint as i32 {
                            self.structures[idx] = StructureKind::None as i32;
                            changed = true;
                        }
                        let before = self.orders.len();
                        self.orders.retain(|o| !(o.x == x && o.y == y));
                        if self.orders.len() != before {
                            changed = true;
                        }
                        if changed {
                            affected += 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        if affected > 0 {
            if mode == "cancel" {
                for c in &mut self.colonists {
                    if let Some(t) = &c.active_task {
                        let in_rect = t.target_x >= min_x
                            && t.target_x <= max_x
                            && t.target_y >= min_y
                            && t.target_y <= max_y;
                        if in_rect && t.kind != OrderKind::HaulItem {
                            c.active_task = None;
                            c.path.clear();
                            c.activity = ColonistActivity::Resting;
                        }
                    }
                }
            }
            self.world_dirty = true;
            if !self.orders.iter().any(|o| o.status == OrderStatus::PendingBriefing)
                && self.storage_approved
                && self.farm_approved
            {
                self.briefing_pending = false;
            }
        }
        affected
    }

    fn leader_index(&self) -> Option<usize> {
        self.colonists.iter().position(|c| c.is_leader)
    }

    /// Lógica del Líder de Colonia: cuando hay órdenes o zonas pendientes de comunicar,
    /// el líder reúne o se acerca personalmente a los colonos y autoriza las tareas tras
    /// una breve reunión.
    fn update_leader_briefing(&mut self, dt: f32) {
        let has_pending_orders = self
            .orders
            .iter()
            .any(|o| o.status == OrderStatus::PendingBriefing);
        let needs_briefing =
            self.briefing_pending || has_pending_orders || !self.storage_approved || !self.farm_approved;

        if !needs_briefing {
            self.meeting_active = false;
            self.meeting_timer = 0.0;
            self.meeting_approach_timer = 0.0;
            self.meeting_target_id = -1;
            return;
        }

        let Some(l_idx) = self.leader_index() else {
            // Si por algún motivo no hubiera líder, aprobar para evitar bloqueos.
            self.approve_all_pending();
            return;
        };

        let leader_speed_mult = self.colonists[l_idx]
            .role_profile
            .leader_meeting_speed_mult();

        // Caso colonia de 1 solo colono: el líder planifica brevemente en el sitio.
        if self.colonists.len() <= 1 {
            self.meeting_active = true;
            self.colonists[l_idx].activity = ColonistActivity::ConveningMeeting;
            self.colonists[l_idx].inventory.equip_main_hand(CopperTool::CopperSword);
            self.colonists[l_idx].path.clear();
            self.meeting_timer += dt * leader_speed_mult;
            if self.meeting_timer >= 0.7 {
                self.approve_all_pending();
            }
            return;
        }

        self.meeting_active = true;
        self.meeting_approach_timer += dt;

        // Elegir el colono más cercano para ir a reunirse personalmente.
        let lx = self.colonists[l_idx].x;
        let ly = self.colonists[l_idx].y;

        let mut best_partner_idx: Option<usize> = None;
        let mut best_dist = f32::MAX;
        for (i, c) in self.colonists.iter().enumerate() {
            if i == l_idx {
                continue;
            }
            let d = ((c.x - lx).powi(2) + (c.y - ly).powi(2)).sqrt();
            if d < best_dist {
                best_dist = d;
                best_partner_idx = Some(i);
            }
        }

        let Some(p_idx) = best_partner_idx else {
            self.approve_all_pending();
            return;
        };

        let partner_id = self.colonists[p_idx].id;
        let px = self.colonists[p_idx].x;
        let py = self.colonists[p_idx].y;
        self.meeting_target_id = partner_id;

        if best_dist <= 2.3 || self.meeting_approach_timer >= (6.5 / leader_speed_mult) {
            // Ya están reunidos cara a cara (o cerca): el líder transmite las órdenes.
            self.colonists[l_idx].activity = ColonistActivity::ConveningMeeting;
            self.colonists[l_idx].inventory.equip_main_hand(CopperTool::CopperSword);
            self.colonists[l_idx].path.clear();
            self.colonists[l_idx].facing_x = (px - lx).signum();
            self.colonists[l_idx].facing_y = (py - ly).signum();

            for (i, c) in self.colonists.iter_mut().enumerate() {
                if i == l_idx {
                    continue;
                }
                let d = ((c.x - lx).powi(2) + (c.y - ly).powi(2)).sqrt();
                if d <= 3.2 && c.active_task.is_none() && c.activity != ColonistActivity::Sleeping {
                    c.activity = ColonistActivity::AttendingMeeting;
                    c.path.clear();
                    c.facing_x = (lx - c.x).signum();
                    c.facing_y = (ly - c.y).signum();
                }
            }

            self.meeting_timer += dt * leader_speed_mult;
            if self.meeting_timer >= 1.35 {
                self.approve_all_pending();
            }
        } else {
            // El líder camina hacia el colono para reunirse personalmente.
            let l_tx = self.colonists[l_idx].x.floor() as i32;
            let l_ty = self.colonists[l_idx].y.floor() as i32;
            let p_tx = px.floor() as i32;
            let p_ty = py.floor() as i32;

            if self.colonists[l_idx].active_task.is_some() {
                self.release_colonist_task(l_idx);
            }
            self.colonists[l_idx].activity = ColonistActivity::ConveningMeeting;
            self.colonists[l_idx].inventory.equip_main_hand(CopperTool::CopperSword);
            if self.colonists[l_idx].path.is_empty() {
                if let Some(path) = find_path(
                    self.map_width,
                    self.map_height,
                    &self.blocked,
                    l_tx,
                    l_ty,
                    p_tx,
                    p_ty,
                    true,
                ) {
                    self.colonists[l_idx].path = path;
                }
            }

            // Los colonos libres cercanos también acuden al llamado de reunión del líder.
            for i in 0..self.colonists.len() {
                if i == l_idx {
                    continue;
                }
                if self.colonists[i].active_task.is_none()
                    && self.colonists[i].activity != ColonistActivity::Sleeping
                {
                    self.colonists[i].activity = ColonistActivity::AttendingMeeting;
                    if self.colonists[i].path.is_empty() {
                        let cx = self.colonists[i].x.floor() as i32;
                        let cy = self.colonists[i].y.floor() as i32;
                        if let Some(path) = find_path(
                            self.map_width,
                            self.map_height,
                            &self.blocked,
                            cx,
                            cy,
                            l_tx,
                            l_ty,
                            true,
                        ) {
                            self.colonists[i].path = path;
                        }
                    }
                }
            }
        }
    }

    fn approve_all_pending(&mut self) {
        let mut approved_count = 0usize;
        for o in &mut self.orders {
            if o.status == OrderStatus::PendingBriefing {
                approved_count += 1;
            }
            o.status = OrderStatus::Approved;
        }
        if let Some(l_idx) = self.leader_index() {
            let xp = 7.0 + (approved_count.min(10) as f32) * 2.5;
            let energy = self.colonists[l_idx].energy;
            self.colonists[l_idx]
                .role_profile
                .grant_xp(ColonyRole::ColonyLeader, xp, energy);
        }
        self.briefing_pending = false;
        self.storage_approved = true;
        self.farm_approved = true;
        self.meeting_active = false;
        self.meeting_timer = 0.0;
        self.meeting_approach_timer = 0.0;
        self.meeting_target_id = -1;
        for c in &mut self.colonists {
            if matches!(
                c.activity,
                ColonistActivity::ConveningMeeting | ColonistActivity::AttendingMeeting
            ) {
                c.activity = ColonistActivity::Resting;
                c.wait_timer = 0.15;
                c.path.clear();
            }
        }
        self.world_dirty = true;
    }

    fn release_colonist_task(&mut self, c_idx: usize) {
        if let Some(task) = self.colonists[c_idx].active_task.take() {
            for o in &mut self.orders {
                if o.id == task.order_id {
                    o.assigned_colonist_id = 0;
                }
            }
        }
        self.colonists[c_idx].path.clear();
    }

    fn find_storage_tile(&self, prefer_near_x: i32, prefer_near_y: i32) -> Option<(i32, i32)> {
        let mut best: Option<(i32, i32)> = None;
        let mut best_score = i32::MAX;
        for y in 1..self.map_height - 1 {
            for x in 1..self.map_width - 1 {
                let idx = (y * self.map_width + x) as usize;
                if self.zones[idx] == ZoneKind::Storage as i32 && self.blocked[idx] == 0 {
                    let has_item = self.items.iter().any(|it| it.x == x && it.y == y);
                    let dist = (x - prefer_near_x).abs() + (y - prefer_near_y).abs();
                    let score = dist + if has_item { 40 } else { 0 };
                    if score < best_score {
                        best_score = score;
                        best = Some((x, y));
                    }
                }
            }
        }
        best
    }

    fn assign_task_to_colonist(&mut self, c_idx: usize) {
        let cx = self.colonists[c_idx].x.floor() as i32;
        let cy = self.colonists[c_idx].y.floor() as i32;
        let cid = self.colonists[c_idx].id;

        // 1. Si el colono ya carga un recurso, entregarlo en el almacén primero.
        if self.colonists[c_idx].carrying_kind.is_some() {
            if let Some((sx, sy)) = self.find_storage_tile(cx, cy) {
                if let Some(path) =
                    find_path(self.map_width, self.map_height, &self.blocked, cx, cy, sx, sy, false)
                {
                    self.colonists[c_idx].active_task = Some(ActiveTask {
                        order_id: 0,
                        kind: OrderKind::HaulItem,
                        target_x: sx,
                        target_y: sy,
                        item_id: 0,
                        work_timer: 0.0,
                        phase: 1,
                    });
                    self.colonists[c_idx].path = path;
                    self.colonists[c_idx].activity = ColonistActivity::Hauling;
                    self.colonists[c_idx].inventory.stow_main_hand();
                    return;
                }
            } else {
                // Si no hay almacén, dejar el recurso en la casilla actual.
                let kind = self.colonists[c_idx].carrying_kind.take();
                let amt = self.colonists[c_idx].carrying_amount;
                self.colonists[c_idx].carrying_amount = 0;
                if let Some(k) = kind {
                    self.spawn_item(k, cx, cy, amt, false);
                }
            }
        }

        // 2. Evaluar los roles asignados al colono (máximo 5) en orden de prioridad.
        let assigned_roles = self.colonists[c_idx].role_profile.assigned.clone();
        if assigned_roles.is_empty() {
            return;
        }
        let (total_inv, _) = self.compute_inventory();

        for role in assigned_roles {
            match role {
                ColonyRole::Constructor | ColonyRole::Lumberjack | ColonyRole::Miner => {
                    let target_kind = match role {
                        ColonyRole::Constructor => OrderKind::BuildWoodWall,
                        ColonyRole::Lumberjack => OrderKind::ChopTree,
                        _ => OrderKind::MineRock,
                    };
                    let mut chosen_order_idx: Option<usize> = None;
                    let mut best_dist = i32::MAX;

                    for (oi, order) in self.orders.iter().enumerate() {
                        if order.status != OrderStatus::Approved
                            || order.assigned_colonist_id != 0
                            || order.kind != target_kind
                        {
                            continue;
                        }
                        if order.kind == OrderKind::BuildWoodWall && total_inv.wood < WOOD_PER_WALL
                        {
                            continue;
                        }
                        let d = (order.x - cx).abs() + (order.y - cy).abs();
                        if d < best_dist {
                            best_dist = d;
                            chosen_order_idx = Some(oi);
                        }
                    }

                    if let Some(oi) = chosen_order_idx {
                        let order_id = self.orders[oi].id;
                        let order_kind = self.orders[oi].kind;
                        let tx = self.orders[oi].x;
                        let ty = self.orders[oi].y;

                        if let Some(path) = find_path(
                            self.map_width,
                            self.map_height,
                            &self.blocked,
                            cx,
                            cy,
                            tx,
                            ty,
                            true,
                        ) {
                            self.orders[oi].assigned_colonist_id = cid;
                            self.colonists[c_idx].active_task = Some(ActiveTask {
                                order_id,
                                kind: order_kind,
                                target_x: tx,
                                target_y: ty,
                                item_id: 0,
                                work_timer: 0.0,
                                phase: 0,
                            });
                            self.colonists[c_idx].path = path;
                            self.colonists[c_idx].activity = ColonistActivity::MovingToTask;
                            self.colonists[c_idx]
                                .inventory
                                .equip_main_hand(role.default_tool());
                            return;
                        }
                    }
                }
                ColonyRole::Farmer => {
                    if !self.farm_approved {
                        continue;
                    }
                    let mut best_crop: Option<(i32, i32)> = None;
                    let mut best_crop_score = i32::MAX;
                    for crop in &self.crops {
                        let already_taken = self.colonists.iter().any(|col| {
                            col.active_task.as_ref().is_some_and(|t| {
                                t.kind == OrderKind::FarmCell
                                    && t.target_x == crop.x
                                    && t.target_y == crop.y
                            })
                        });
                        if already_taken {
                            continue;
                        }
                        let needs_action =
                            matches!(crop.stage, CropStage::None | CropStage::Ready);
                        if !needs_action {
                            continue;
                        }
                        let priority_bonus = if crop.stage == CropStage::Ready { -50 } else { 0 };
                        let score = (crop.x - cx).abs() + (crop.y - cy).abs() + priority_bonus;
                        if score < best_crop_score {
                            best_crop_score = score;
                            best_crop = Some((crop.x, crop.y));
                        }
                    }

                    if let Some((fx, fy)) = best_crop {
                        if let Some(path) = find_path(
                            self.map_width,
                            self.map_height,
                            &self.blocked,
                            cx,
                            cy,
                            fx,
                            fy,
                            false,
                        ) {
                            self.colonists[c_idx].active_task = Some(ActiveTask {
                                order_id: 0,
                                kind: OrderKind::FarmCell,
                                target_x: fx,
                                target_y: fy,
                                item_id: 0,
                                work_timer: 0.0,
                                phase: 0, // 0 = Agricultor (arar / cosechar)
                            });
                            self.colonists[c_idx].path = path;
                            self.colonists[c_idx].activity = ColonistActivity::MovingToTask;
                            self.colonists[c_idx]
                                .inventory
                                .equip_main_hand(CopperTool::CopperHoe);
                            return;
                        }
                    }
                }
                ColonyRole::Botanist => {
                    if !self.farm_approved {
                        continue;
                    }
                    let mut best_crop: Option<(i32, i32)> = None;
                    let mut best_crop_score = i32::MAX;
                    for crop in &self.crops {
                        let already_taken = self.colonists.iter().any(|col| {
                            col.active_task.as_ref().is_some_and(|t| {
                                t.kind == OrderKind::FarmCell
                                    && t.target_x == crop.x
                                    && t.target_y == crop.y
                            })
                        });
                        if already_taken {
                            continue;
                        }
                        let can_seed =
                            crop.stage == CropStage::Tilled && total_inv.carrot_seeds > 0;
                        let can_tend = matches!(
                            crop.stage,
                            CropStage::Seeded | CropStage::Sprout | CropStage::Growing
                        ) && crop.tend_cooldown <= 0.0;
                        if !can_seed && !can_tend {
                            continue;
                        }
                        let priority_bonus = if can_seed { -35 } else { 0 };
                        let score = (crop.x - cx).abs() + (crop.y - cy).abs() + priority_bonus;
                        if score < best_crop_score {
                            best_crop_score = score;
                            best_crop = Some((crop.x, crop.y));
                        }
                    }

                    if let Some((fx, fy)) = best_crop {
                        if let Some(path) = find_path(
                            self.map_width,
                            self.map_height,
                            &self.blocked,
                            cx,
                            cy,
                            fx,
                            fy,
                            false,
                        ) {
                            self.colonists[c_idx].active_task = Some(ActiveTask {
                                order_id: 0,
                                kind: OrderKind::FarmCell,
                                target_x: fx,
                                target_y: fy,
                                item_id: 0,
                                work_timer: 0.0,
                                phase: 1, // 1 = Botánico (sembrar / cuidar crecimiento)
                            });
                            self.colonists[c_idx].path = path;
                            self.colonists[c_idx].activity = ColonistActivity::MovingToTask;
                            self.colonists[c_idx]
                                .inventory
                                .equip_main_hand(CopperTool::CopperHoe);
                            return;
                        }
                    }
                }
                ColonyRole::Hauler => {
                    if !self.storage_approved || self.find_storage_tile(cx, cy).is_none() {
                        continue;
                    }
                    let mut best_item: Option<(i32, i32, i32)> = None;
                    let mut best_item_dist = i32::MAX;
                    for it in &self.items {
                        if it.stored {
                            continue;
                        }
                        let reserved = self.colonists.iter().any(|col| {
                            col.active_task
                                .as_ref()
                                .is_some_and(|t| t.kind == OrderKind::HaulItem && t.item_id == it.id)
                        });
                        if reserved {
                            continue;
                        }
                        let d = (it.x - cx).abs() + (it.y - cy).abs();
                        if d < best_item_dist {
                            best_item_dist = d;
                            best_item = Some((it.id, it.x, it.y));
                        }
                    }

                    if let Some((item_id, ix, iy)) = best_item {
                        if let Some(path) = find_path(
                            self.map_width,
                            self.map_height,
                            &self.blocked,
                            cx,
                            cy,
                            ix,
                            iy,
                            false,
                        ) {
                            self.colonists[c_idx].active_task = Some(ActiveTask {
                                order_id: 0,
                                kind: OrderKind::HaulItem,
                                target_x: ix,
                                target_y: iy,
                                item_id,
                                work_timer: 0.0,
                                phase: 0,
                            });
                            self.colonists[c_idx].path = path;
                            self.colonists[c_idx].activity = ColonistActivity::Hauling;
                            self.colonists[c_idx].inventory.stow_main_hand();
                            return;
                        }
                    }
                }
                ColonyRole::Scout | ColonyRole::ColonyLeader => {}
            }
        }
    }

    fn step_colonist_movement(&mut self, c_idx: usize, dt: f32) -> bool {
        let c = &mut self.colonists[c_idx];
        let role_speed_mult = match c.activity {
            ColonistActivity::Hauling => c.role_profile.haul_speed_mult(),
            ColonistActivity::ConveningMeeting => c.role_profile.leader_meeting_speed_mult(),
            ColonistActivity::Scouting => c.role_profile.scout_speed_mult(),
            _ => 1.0,
        };
        if let Some(&(nx_tile, ny_tile)) = c.path.first() {
            let tx = nx_tile as f32 + 0.5;
            let ty = ny_tile as f32 + 0.5;
            let dx = tx - c.x;
            let dy = ty - c.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 0.08 {
                c.x = tx;
                c.y = ty;
                c.path.remove(0);
                return false;
            }
            let effective_speed = c.speed * role_speed_mult;
            let step = (effective_speed * dt).min(dist);
            let ux = dx / dist;
            let uy = dy / dist;
            c.x += ux * step;
            c.y += uy * step;
            c.facing_x = ux;
            c.facing_y = uy;
            c.walk_phase += dt * effective_speed * 4.8;
            return true;
        }
        false
    }

    fn execute_active_task(&mut self, c_idx: usize, dt: f32) {
        let Some(task) = self.colonists[c_idx].active_task.clone() else {
            return;
        };

        // Si aún está caminando por la ruta A*, avanzar.
        if !self.colonists[c_idx].path.is_empty() {
            self.step_colonist_movement(c_idx, dt);
            return;
        }

        let cx = self.colonists[c_idx].x;
        let cy = self.colonists[c_idx].y;
        let target_cx = task.target_x as f32 + 0.5;
        let target_cy = task.target_y as f32 + 0.5;
        self.colonists[c_idx].facing_x = (target_cx - cx).signum();
        self.colonists[c_idx].facing_y = (target_cy - cy).signum();

        match task.kind {
            OrderKind::ChopTree => {
                let speed_mult = self.colonists[c_idx].role_profile.chop_speed_mult();
                self.colonists[c_idx].activity = ColonistActivity::Chopping;
                self.colonists[c_idx].inventory.equip_main_hand(CopperTool::CopperAxe);
                self.colonists[c_idx].work_swing_phase += dt * 11.0 * speed_mult;

                let mut t = task;
                let prev_t = t.work_timer;
                t.work_timer += dt * speed_mult;
                for beat in [0.25f32, 0.95, 1.65] {
                    if prev_t < beat && t.work_timer >= beat {
                        self.pending_sfx.push("chop");
                    }
                }
                if t.work_timer < 2.1 {
                    self.colonists[c_idx].active_task = Some(t);
                    return;
                }

                // Tala completada: quitar árbol y soltar troncos según nivel de Leñador.
                let wood_yield = self.colonists[c_idx].role_profile.chop_wood_yield();
                if let Some(idx) = self.idx(t.target_x, t.target_y) {
                    if self.props[idx] == 1 || self.props[idx] == 2 {
                        self.props[idx] = 0;
                        self.blocked[idx] = 0;
                        self.spawn_item(ItemKind::WoodLogs, t.target_x, t.target_y, wood_yield, false);
                        self.pending_sfx.push("chop");
                        let energy = self.colonists[c_idx].energy;
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Lumberjack, 14.0, energy);
                    }
                }
                self.orders.retain(|o| o.id != t.order_id);
                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.2;
                self.world_dirty = true;
            }
            OrderKind::MineRock => {
                let speed_mult = self.colonists[c_idx].role_profile.mine_speed_mult();
                self.colonists[c_idx].activity = ColonistActivity::Mining;
                self.colonists[c_idx].inventory.equip_main_hand(CopperTool::CopperPickaxe);
                self.colonists[c_idx].work_swing_phase += dt * 12.0 * speed_mult;

                let mut t = task;
                let prev_t = t.work_timer;
                t.work_timer += dt * speed_mult;
                for beat in [0.25f32, 1.05, 1.85] {
                    if prev_t < beat && t.work_timer >= beat {
                        self.pending_sfx.push("mine");
                    }
                }
                if t.work_timer < 2.5 {
                    self.colonists[c_idx].active_task = Some(t);
                    return;
                }

                // Minería completada: soltar piedra / hierro / cobre + bono según nivel de Minero.
                let bonus = self.colonists[c_idx].role_profile.mine_bonus_yield();
                if let Some(idx) = self.idx(t.target_x, t.target_y) {
                    let prop = self.props[idx];
                    if prop == 4 {
                        self.props[idx] = 0;
                        self.blocked[idx] = 0;
                        self.spawn_item(ItemKind::StoneChunk, t.target_x, t.target_y, 3 + bonus, false);
                        self.spawn_item(
                            ItemKind::CopperOre,
                            t.target_x,
                            t.target_y,
                            1 + if bonus >= 2 { 1 } else { 0 },
                            false,
                        );
                        self.pending_sfx.push("mine");
                        let energy = self.colonists[c_idx].energy;
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Miner, 16.0, energy);
                    } else if prop == 5 {
                        self.props[idx] = 0;
                        self.blocked[idx] = 0;
                        self.spawn_item(ItemKind::IronOre, t.target_x, t.target_y, 3 + bonus, false);
                        self.spawn_item(ItemKind::StoneChunk, t.target_x, t.target_y, 1 + bonus, false);
                        self.pending_sfx.push("mine");
                        let energy = self.colonists[c_idx].energy;
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Miner, 18.0, energy);
                    }
                }
                self.orders.retain(|o| o.id != t.order_id);
                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.2;
                self.world_dirty = true;
            }
            OrderKind::BuildWoodWall => {
                let speed_mult = self.colonists[c_idx].role_profile.build_speed_mult();
                self.colonists[c_idx].activity = ColonistActivity::Building;
                self.colonists[c_idx].inventory.equip_main_hand(CopperTool::CopperShovel);
                self.colonists[c_idx].work_swing_phase += dt * 10.0 * speed_mult;

                let mut t = task;
                let prev_t = t.work_timer;
                t.work_timer += dt * speed_mult;
                for beat in [0.25f32, 0.90, 1.50] {
                    if prev_t < beat && t.work_timer >= beat {
                        self.pending_sfx.push("build");
                    }
                }
                if t.work_timer < 1.8 {
                    self.colonists[c_idx].active_task = Some(t);
                    return;
                }

                if let Some(idx) = self.idx(t.target_x, t.target_y) {
                    if self.structures[idx] == StructureKind::WoodWallBlueprint as i32
                        && self.consume_item_from_world(ItemKind::WoodLogs, WOOD_PER_WALL)
                    {
                        self.structures[idx] = StructureKind::WoodWall as i32;
                        self.blocked[idx] = 1;
                        self.orders.retain(|o| o.id != t.order_id);
                        self.pending_sfx.push("build");
                        let energy = self.colonists[c_idx].energy;
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Constructor, 15.0, energy);
                    } else if self.structures[idx] != StructureKind::WoodWallBlueprint as i32 {
                        self.orders.retain(|o| o.id != t.order_id);
                    } else {
                        // Faltó madera mientras llegaba: liberar orden hasta que haya madera.
                        for o in &mut self.orders {
                            if o.id == t.order_id {
                                o.assigned_colonist_id = 0;
                            }
                        }
                    }
                }
                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.25;
                self.world_dirty = true;
            }
            OrderKind::FarmCell => {
                let is_botanist_task = task.phase == 1;
                let speed_mult = if is_botanist_task {
                    self.colonists[c_idx].role_profile.botany_speed_mult()
                } else {
                    self.colonists[c_idx].role_profile.farm_speed_mult()
                };
                self.colonists[c_idx].activity = if is_botanist_task {
                    ColonistActivity::Botanizing
                } else {
                    ColonistActivity::Farming
                };
                self.colonists[c_idx].inventory.equip_main_hand(CopperTool::CopperHoe);
                self.colonists[c_idx].work_swing_phase += dt * 9.5 * speed_mult;

                let mut t = task;
                t.work_timer += dt * speed_mult;
                if t.work_timer < 1.15 {
                    self.colonists[c_idx].active_task = Some(t);
                    return;
                }

                let mut harvested_drop = false;
                let mut need_seed = false;
                let mut tilled_xp = false;
                let mut tended_xp = false;
                let botany_boost = self.colonists[c_idx].role_profile.botany_growth_boost();

                if let Some(crop) = self
                    .crops
                    .iter_mut()
                    .find(|c| c.x == t.target_x && c.y == t.target_y)
                {
                    match crop.stage {
                        CropStage::None if !is_botanist_task => {
                            crop.stage = CropStage::Tilled;
                            crop.progress = 0.0;
                            tilled_xp = true;
                        }
                        CropStage::Tilled if is_botanist_task => {
                            need_seed = true;
                        }
                        CropStage::Seeded | CropStage::Sprout | CropStage::Growing
                            if is_botanist_task && crop.tend_cooldown <= 0.0 =>
                        {
                            crop.progress = (crop.progress + botany_boost).min(1.0);
                            crop.tend_cooldown = 10.0;
                            crop.stage = if crop.progress >= 1.0 {
                                CropStage::Ready
                            } else if crop.progress >= 0.66 {
                                CropStage::Growing
                            } else if crop.progress >= 0.33 {
                                CropStage::Sprout
                            } else {
                                CropStage::Seeded
                            };
                            tended_xp = true;
                        }
                        CropStage::Ready if !is_botanist_task => {
                            crop.stage = CropStage::Tilled;
                            crop.progress = 0.0;
                            crop.tend_cooldown = 0.0;
                            harvested_drop = true;
                        }
                        _ => {}
                    }
                }

                let energy = self.colonists[c_idx].energy;
                if tilled_xp {
                    self.colonists[c_idx]
                        .role_profile
                        .grant_xp(ColonyRole::Farmer, 10.0, energy);
                }
                if tended_xp {
                    self.colonists[c_idx]
                        .role_profile
                        .grant_xp(ColonyRole::Botanist, 13.0, energy);
                }

                if need_seed && self.consume_item_from_world(ItemKind::CarrotSeed, 1) {
                    if let Some(crop) = self
                        .crops
                        .iter_mut()
                        .find(|c| c.x == t.target_x && c.y == t.target_y)
                    {
                        crop.stage = CropStage::Seeded;
                        crop.progress = 0.0;
                        crop.tend_cooldown = 4.0;
                    }
                    self.colonists[c_idx]
                        .role_profile
                        .grant_xp(ColonyRole::Botanist, 11.0, energy);
                }

                if harvested_drop {
                    let (carrots_yield, seeds_yield) =
                        self.colonists[c_idx].role_profile.farm_harvest_yield();
                    self.spawn_item(ItemKind::Carrot, t.target_x, t.target_y, carrots_yield, false);
                    self.spawn_item(
                        ItemKind::CarrotSeed,
                        t.target_x,
                        t.target_y,
                        seeds_yield,
                        false,
                    );
                    self.colonists[c_idx]
                        .role_profile
                        .grant_xp(ColonyRole::Farmer, 16.0, energy);
                }

                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.18;
                self.world_dirty = true;
            }
            OrderKind::HaulItem => {
                self.colonists[c_idx].activity = ColonistActivity::Hauling;
                self.colonists[c_idx].inventory.stow_main_hand();
                if task.phase == 0 {
                    // Recoger el ítem del piso según la capacidad de carga del Transportador.
                    let carry_cap = self.colonists[c_idx].role_profile.haul_capacity();
                    if let Some(pos) = self.items.iter().position(|it| it.id == task.item_id) {
                        let taken_kind = self.items[pos].kind;
                        let taken_amt = if self.items[pos].amount <= carry_cap {
                            let it = self.items.remove(pos);
                            it.amount
                        } else {
                            self.items[pos].amount -= carry_cap;
                            carry_cap
                        };
                        self.colonists[c_idx].carrying_kind = Some(taken_kind);
                        self.colonists[c_idx].carrying_amount = taken_amt;
                        self.world_dirty = true;

                        let cur_x = self.colonists[c_idx].x.floor() as i32;
                        let cur_y = self.colonists[c_idx].y.floor() as i32;
                        if let Some((sx, sy)) = self.find_storage_tile(cur_x, cur_y) {
                            if let Some(path) = find_path(
                                self.map_width,
                                self.map_height,
                                &self.blocked,
                                cur_x,
                                cur_y,
                                sx,
                                sy,
                                false,
                            ) {
                                self.colonists[c_idx].active_task = Some(ActiveTask {
                                    order_id: 0,
                                    kind: OrderKind::HaulItem,
                                    target_x: sx,
                                    target_y: sy,
                                    item_id: 0,
                                    work_timer: 0.0,
                                    phase: 1,
                                });
                                self.colonists[c_idx].path = path;
                                return;
                            }
                        }
                        // Si no encontró almacén, soltarlo de vuelta.
                        let k = self.colonists[c_idx].carrying_kind.take();
                        let a = self.colonists[c_idx].carrying_amount;
                        self.colonists[c_idx].carrying_amount = 0;
                        if let Some(kind) = k {
                            self.spawn_item(kind, cur_x, cur_y, a, false);
                        }
                    }
                    self.colonists[c_idx].active_task = None;
                    self.colonists[c_idx].activity = ColonistActivity::Resting;
                } else {
                    // Depositar ítem en la casilla de almacén destino y ganar experiencia de Transportador.
                    let kind = self.colonists[c_idx].carrying_kind.take();
                    let amt = self.colonists[c_idx].carrying_amount;
                    self.colonists[c_idx].carrying_amount = 0;
                    if let Some(k) = kind {
                        self.spawn_item(k, task.target_x, task.target_y, amt, true);
                        let energy = self.colonists[c_idx].energy;
                        self.colonists[c_idx].role_profile.grant_xp(
                            ColonyRole::Hauler,
                            8.0 + (amt as f32) * 1.2,
                            energy,
                        );
                    }
                    self.colonists[c_idx].active_task = None;
                    self.colonists[c_idx].activity = ColonistActivity::Resting;
                    self.colonists[c_idx].wait_timer = 0.15;
                }
            }
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

        // 1. Crecimiento progresivo de las zanahorias sembradas y enfriamiento de cuidado botánico.
        let mut crop_stage_changed = false;
        for crop in &mut self.crops {
            if crop.tend_cooldown > 0.0 {
                crop.tend_cooldown = (crop.tend_cooldown - dt).max(0.0);
            }
            if matches!(
                crop.stage,
                CropStage::Seeded | CropStage::Sprout | CropStage::Growing
            ) {
                let idx = (crop.y * self.map_width + crop.x) as usize;
                let fertile_bonus = if self.ground.get(idx).copied().unwrap_or(0) == 1 {
                    0.078
                } else {
                    0.048
                };
                crop.progress = (crop.progress + fertile_bonus * dt).min(1.0);
                let next_stage = if crop.progress >= 1.0 {
                    CropStage::Ready
                } else if crop.progress >= 0.66 {
                    CropStage::Growing
                } else if crop.progress >= 0.33 {
                    CropStage::Sprout
                } else {
                    CropStage::Seeded
                };
                if next_stage != crop.stage {
                    crop.stage = next_stage;
                    crop_stage_changed = true;
                }
            }
        }
        if crop_stage_changed {
            self.world_dirty = true;
        }

        // 2. Actualizar reunión del Líder de Colonia si hay órdenes nuevas pendientes.
        self.update_leader_briefing(dt);

        // 3. Actualizar cada colono (energía, sueño en el suelo, reuniones, patrulla y tareas).
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
                self.colonists[c_idx].energy = (self.colonists[c_idx].energy + 14.0 * dt).min(100.0);
                if self.colonists[c_idx].energy >= 96.0 {
                    self.colonists[c_idx].activity = ColonistActivity::Resting;
                    self.colonists[c_idx].wait_timer = 0.2;
                    let is_leader = self.colonists[c_idx].is_leader;
                    let default_tool = self.colonists[c_idx].role_profile.default_tool(is_leader);
                    self.colonists[c_idx].inventory.equip_main_hand(default_tool);
                }
                continue;
            }

            let drain_rate = match self.colonists[c_idx].activity {
                ColonistActivity::Chopping
                | ColonistActivity::Mining
                | ColonistActivity::Building
                | ColonistActivity::Farming
                | ColonistActivity::Botanizing => 2.0,
                ColonistActivity::MovingToTask
                | ColonistActivity::Hauling
                | ColonistActivity::Wandering
                | ColonistActivity::Scouting => 0.95,
                _ => 0.25,
            };
            self.colonists[c_idx].energy = (self.colonists[c_idx].energy - drain_rate * dt).max(0.0);

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

            // Buscar nueva tarea aprobada según los roles asignados al colono.
            self.assign_task_to_colonist(c_idx);
            if self.colonists[c_idx].active_task.is_some() {
                continue;
            }

            // Sin tareas activas: patrullar/explorar si tiene rol Explorador, o pasear cerca.
            let is_leader = self.colonists[c_idx].is_leader;
            let has_scout = self.colonists[c_idx].role_profile.has_role(ColonyRole::Scout);
            let default_tool = self.colonists[c_idx].role_profile.default_tool(is_leader);
            self.colonists[c_idx].inventory.equip_main_hand(default_tool);

            if self.colonists[c_idx].wait_timer > 0.0 {
                self.colonists[c_idx].wait_timer -= dt;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                continue;
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
                        self.colonists[c_idx].activity =
                            if has_scout && self.rng.next_f32() < 0.78 {
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
                continue;
            }

            let nx = dx / dist;
            let ny = dy / dist;
            let patrol_mult = if self.colonists[c_idx].activity == ColonistActivity::Scouting && has_scout {
                self.colonists[c_idx].role_profile.scout_speed_mult()
            } else {
                1.0
            };
            let effective_speed = self.colonists[c_idx].speed * patrol_mult;
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
                    self.colonists[c_idx]
                        .role_profile
                        .grant_xp(ColonyRole::Scout, dt * 3.5, energy);
                }
            }
        }
    }

    /// Asigna o quita uno de los 8 roles a un colono respetando el límite de 5 roles
    /// y manteniendo siempre un Líder de Colonia activo.
    pub fn set_colonist_role_enabled(
        &mut self,
        colonist_id: i32,
        role: ColonyRole,
        enabled: bool,
    ) -> Result<&'static str, &'static str> {
        let Some(c_idx) = self.colonists.iter().position(|c| c.id == colonist_id) else {
            return Err("Colono no encontrado.");
        };

        if role == ColonyRole::ColonyLeader {
            if enabled {
                if !self.colonists[c_idx]
                    .role_profile
                    .has_role(ColonyRole::ColonyLeader)
                    && self.colonists[c_idx].role_profile.assigned.len() >= MAX_ASSIGNED_ROLES
                {
                    return Err("Máximo 5 roles por colono. Quita un rol antes.");
                }
                for (i, other) in self.colonists.iter_mut().enumerate() {
                    if i != c_idx {
                        other.is_leader = false;
                        other.role_profile.unassign_role(ColonyRole::ColonyLeader);
                        other.role = other.role_profile.primary_label(false).to_string();
                        if other.activity == ColonistActivity::ConveningMeeting {
                            other.activity = ColonistActivity::Resting;
                            other.path.clear();
                        }
                    }
                }
                self.colonists[c_idx].is_leader = true;
                self.colonists[c_idx]
                    .role_profile
                    .assign_role(ColonyRole::ColonyLeader)?;
                self.colonists[c_idx].role = self.colonists[c_idx]
                    .role_profile
                    .primary_label(true)
                    .to_string();
                if self.colonists[c_idx].active_task.is_none()
                    && self.colonists[c_idx].activity != ColonistActivity::Sleeping
                {
                    self.colonists[c_idx]
                        .inventory
                        .equip_main_hand(CopperTool::CopperSword);
                }
                self.world_dirty = true;
                return Ok("Líder de Colonia asignado.");
            } else {
                let other_leader = self
                    .colonists
                    .iter()
                    .enumerate()
                    .any(|(i, c)| i != c_idx && (c.is_leader || c.role_profile.has_role(ColonyRole::ColonyLeader)));
                if self.colonists[c_idx].is_leader || !other_leader {
                    return Err("La colonia necesita un Líder. Asígnalo primero a otro colono.");
                }
                self.colonists[c_idx]
                    .role_profile
                    .unassign_role(ColonyRole::ColonyLeader);
                self.world_dirty = true;
                return Ok("Rol actualizado.");
            }
        }

        if enabled {
            self.colonists[c_idx].role_profile.assign_role(role)?;
            let is_leader = self.colonists[c_idx].is_leader;
            self.colonists[c_idx].role = self.colonists[c_idx]
                .role_profile
                .primary_label(is_leader)
                .to_string();
            Ok("Rol asignado.")
        } else {
            self.colonists[c_idx].role_profile.unassign_role(role);
            let is_leader = self.colonists[c_idx].is_leader;
            self.colonists[c_idx].role = self.colonists[c_idx]
                .role_profile
                .primary_label(is_leader)
                .to_string();

            let should_release = if let Some(task) = &self.colonists[c_idx].active_task {
                match task.kind {
                    OrderKind::ChopTree => !self.colonists[c_idx]
                        .role_profile
                        .has_role(ColonyRole::Lumberjack),
                    OrderKind::MineRock => {
                        !self.colonists[c_idx].role_profile.has_role(ColonyRole::Miner)
                    }
                    OrderKind::BuildWoodWall => !self.colonists[c_idx]
                        .role_profile
                        .has_role(ColonyRole::Constructor),
                    OrderKind::FarmCell => {
                        if task.phase == 1 {
                            !self.colonists[c_idx]
                                .role_profile
                                .has_role(ColonyRole::Botanist)
                        } else {
                            !self.colonists[c_idx]
                                .role_profile
                                .has_role(ColonyRole::Farmer)
                        }
                    }
                    OrderKind::HaulItem => {
                        task.phase == 0
                            && self.colonists[c_idx].carrying_kind.is_none()
                            && !self.colonists[c_idx].role_profile.has_role(ColonyRole::Hauler)
                    }
                }
            } else {
                false
            };

            if should_release {
                self.release_colonist_task(c_idx);
                self.colonists[c_idx].activity = ColonistActivity::Resting;
            }
            Ok("Rol quitado.")
        }
    }
}

/// Nodo GDExtension que administra la simulación completa de la colonia en tiempo real.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct ColonySimulation {
    base: Base<Node>,
    core: Option<SimulationCore>,
}

#[godot_api]
impl INode for ColonySimulation {
    fn init(base: Base<Node>) -> Self {
        Self { base, core: None }
    }
}

#[godot_api]
impl ColonySimulation {
    /// Inicializa la colonia nueva sobre el terreno generado, eligiendo un líder aleatorio.
    #[func]
    pub fn setup_colony(
        &mut self,
        seed: i64,
        width: i32,
        height: i32,
        ground_tiles: PackedInt32Array,
        prop_tiles: PackedInt32Array,
        blocked_tiles: PackedInt32Array,
        colonist_count: i32,
    ) {
        let mut core = SimulationCore::new(
            seed as u64,
            width,
            height,
            ground_tiles.to_vec(),
            prop_tiles.to_vec(),
            blocked_tiles.to_vec(),
        );
        core.spawn_initial(colonist_count);
        self.core = Some(core);
    }

    /// Restaura una colonia guardada desde JSON (`user://saves/slot_N.json`).
    #[func]
    pub fn restore_colony(
        &mut self,
        seed: i64,
        width: i32,
        height: i32,
        ground_tiles: PackedInt32Array,
        prop_tiles: PackedInt32Array,
        blocked_tiles: PackedInt32Array,
        saved_state: VarDictionary,
    ) {
        let mut core = SimulationCore::new(
            seed as u64,
            width,
            height,
            ground_tiles.to_vec(),
            prop_tiles.to_vec(),
            blocked_tiles.to_vec(),
        );

        core.day_count = saved_state
            .get("day")
            .and_then(|v| v.try_to::<i32>().ok())
            .unwrap_or(1)
            .max(1);

        if let Some(arr) = saved_state.get("props").and_then(|v| v.try_to::<PackedInt32Array>().ok()) {
            let v = arr.to_vec();
            if v.len() == core.props.len() {
                core.props = v;
            }
        }
        if let Some(arr) = saved_state.get("blocked").and_then(|v| v.try_to::<PackedInt32Array>().ok()) {
            let v = arr.to_vec();
            if v.len() == core.blocked.len() {
                core.blocked = v;
            }
        }
        if let Some(arr) = saved_state.get("zones").and_then(|v| v.try_to::<PackedInt32Array>().ok()) {
            let v = arr.to_vec();
            if v.len() == core.zones.len() {
                core.zones = v;
            }
        }
        if let Some(arr) = saved_state.get("structures").and_then(|v| v.try_to::<PackedInt32Array>().ok()) {
            let v = arr.to_vec();
            if v.len() == core.structures.len() {
                core.structures = v;
            }
        }

        if let Some(crops_arr) = saved_state.get("crops").and_then(|v| v.try_to::<VarArray>().ok()) {
            for entry in crops_arr.iter_shared() {
                if let Ok(d) = entry.try_to::<VarDictionary>() {
                    let x = d.get("x").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                    let y = d.get("y").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                    let stage_i = d.get("stage").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                    let progress = d.get("progress").and_then(|v| v.try_to::<f32>().ok()).unwrap_or(0.0);
                    let tend_cooldown = d
                        .get("tend_cooldown")
                        .and_then(|v| v.try_to::<f32>().ok())
                        .unwrap_or(0.0)
                        .max(0.0);
                    let stage = match stage_i {
                        1 => CropStage::Tilled,
                        2 => CropStage::Seeded,
                        3 => CropStage::Sprout,
                        4 => CropStage::Growing,
                        5 => CropStage::Ready,
                        _ => CropStage::None,
                    };
                    core.crops.push(CropCell {
                        x,
                        y,
                        stage,
                        progress,
                        tend_cooldown,
                    });
                }
            }
        }

        if let Some(items_arr) = saved_state.get("items").and_then(|v| v.try_to::<VarArray>().ok()) {
            for entry in items_arr.iter_shared() {
                if let Ok(d) = entry.try_to::<VarDictionary>() {
                    let kind_code = d
                        .get("kind")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|g| g.to_string())
                        .unwrap_or_default();
                    if let Some(kind) = ItemKind::from_code(&kind_code) {
                        let x = d.get("x").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                        let y = d.get("y").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                        let amount = d.get("amount").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(1);
                        let stored = d.get("stored").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false);
                        core.spawn_item(kind, x, y, amount, stored);
                    }
                }
            }
        }

        if let Some(orders_arr) = saved_state.get("orders").and_then(|v| v.try_to::<VarArray>().ok()) {
            for entry in orders_arr.iter_shared() {
                if let Ok(d) = entry.try_to::<VarDictionary>() {
                    let kind_i = d.get("kind").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(1);
                    let x = d.get("x").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                    let y = d.get("y").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                    let status_i = d.get("status").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                    let kind = match kind_i {
                        1 => OrderKind::ChopTree,
                        2 => OrderKind::MineRock,
                        3 => OrderKind::BuildWoodWall,
                        4 => OrderKind::FarmCell,
                        _ => OrderKind::HaulItem,
                    };
                    let status = if status_i == 1 {
                        OrderStatus::Approved
                    } else {
                        OrderStatus::PendingBriefing
                    };
                    let id = core.next_order_id;
                    core.next_order_id += 1;
                    core.orders.push(ColonyOrder {
                        id,
                        kind,
                        x,
                        y,
                        status,
                        assigned_colonist_id: 0,
                    });
                }
            }
        }

        if let Some(cols_arr) = saved_state.get("colonists").and_then(|v| v.try_to::<VarArray>().ok()) {
            for item in cols_arr.iter_shared() {
                if let Ok(dict) = item.try_to::<VarDictionary>() {
                    let id = dict.get("id").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(1);
                    let name = dict
                        .get("name")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|g| g.to_string())
                        .unwrap_or_else(|| "Colono".to_string());
                    let role = dict
                        .get("role")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|g| g.to_string())
                        .unwrap_or_else(|| "Explorador".to_string());
                    let is_leader = dict
                        .get("is_leader")
                        .and_then(|v| v.try_to::<bool>().ok())
                        .unwrap_or(false);
                    let x = dict
                        .get("x")
                        .and_then(|v| v.try_to::<f32>().ok())
                        .unwrap_or(width as f32 * 0.5);
                    let y = dict
                        .get("y")
                        .and_then(|v| v.try_to::<f32>().ok())
                        .unwrap_or(height as f32 * 0.5);
                    let health = dict.get("health").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(100);
                    let mood = dict.get("mood").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(80);
                    let body_style = dict.get("body_style").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                    let hair_style = dict.get("hair_style").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                    let skin_color = dict
                        .get("skin_color")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|g| g.to_string())
                        .unwrap_or_else(|| "#e4b892".to_string());
                    let hair_color = dict
                        .get("hair_color")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|g| g.to_string())
                        .unwrap_or_else(|| "#2c221e".to_string());
                    let outfit_color = dict
                        .get("outfit_color")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|g| g.to_string())
                        .unwrap_or_else(|| "#4f6b47".to_string());
                    let energy = dict
                        .get("energy")
                        .and_then(|v| v.try_to::<f32>().ok())
                        .unwrap_or(92.0)
                        .clamp(10.0, 100.0);
                    let helmet_slot = dict
                        .get("helmet_slot")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|g| g.to_string())
                        .unwrap_or_default();
                    let armor_slot = dict
                        .get("armor_slot")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|g| g.to_string())
                        .unwrap_or_default();
                    let back_code = dict
                        .get("back_tool")
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|g| g.to_string())
                        .unwrap_or_default();
                    let main_code = dict
                        .get("main_hand_tool")
                        .or_else(|| dict.get("equipped_tool"))
                        .and_then(|v| v.try_to::<GString>().ok())
                        .map(|g| g.to_string())
                        .unwrap_or_default();

                    let mut role_profile = ColonistRoleProfile::new_for_colonist(
                        core.colonists.len(),
                        is_leader,
                    );
                    if let Some(roles_arr) = dict.get("assigned_roles").and_then(|v| v.try_to::<VarArray>().ok()) {
                        let mut parsed_roles: Vec<ColonyRole> = Vec::new();
                        for rv in roles_arr.iter_shared() {
                            if let Ok(gs) = rv.try_to::<GString>() {
                                if let Some(r) = ColonyRole::from_code(&gs.to_string()) {
                                    if !parsed_roles.contains(&r) && parsed_roles.len() < MAX_ASSIGNED_ROLES {
                                        parsed_roles.push(r);
                                    }
                                }
                            }
                        }
                        role_profile.assigned = parsed_roles;
                    } else if let Some(legacy_role) = ColonyRole::from_label_es(&role) {
                        if !role_profile.has_role(legacy_role)
                            && role_profile.assigned.len() < MAX_ASSIGNED_ROLES
                        {
                            let _ = role_profile.assign_role(legacy_role);
                        }
                    }

                    if is_leader && !role_profile.has_role(ColonyRole::ColonyLeader) {
                        if role_profile.assigned.len() >= MAX_ASSIGNED_ROLES {
                            role_profile.assigned.pop();
                        }
                        let _ = role_profile.assign_role(ColonyRole::ColonyLeader);
                    }

                    if let Some(levels_dict) = dict.get("role_levels").and_then(|v| v.try_to::<VarDictionary>().ok()) {
                        let xp_dict = dict.get("role_xp").and_then(|v| v.try_to::<VarDictionary>().ok());
                        for r in ColonyRole::ALL {
                            let lv = levels_dict
                                .get(r.code())
                                .and_then(|v| v.try_to::<i32>().ok())
                                .unwrap_or(1);
                            let xp = xp_dict
                                .as_ref()
                                .and_then(|xd| xd.get(r.code()))
                                .and_then(|v| v.try_to::<f32>().ok())
                                .unwrap_or(0.0);
                            role_profile.set_skill(r, lv, xp);
                        }
                    }

                    let mut inventory = ColonistInventory::default_for_role(&role, is_leader);
                    inventory.helmet_slot = helmet_slot;
                    inventory.armor_slot = armor_slot;
                    if !back_code.is_empty() {
                        inventory.back_tool = CopperTool::from_code(&back_code);
                    }
                    if !main_code.is_empty() {
                        inventory.main_hand = CopperTool::from_code(&main_code);
                    }

                    let display_role = role_profile.primary_label(is_leader).to_string();

                    core.colonists.push(Colonist {
                        id,
                        name,
                        role: display_role,
                        is_leader,
                        x,
                        y,
                        target_x: x,
                        target_y: y,
                        path: Vec::new(),
                        facing_x: 0.0,
                        facing_y: 1.0,
                        speed: 2.35,
                        health,
                        mood,
                        energy,
                        wait_timer: 0.4,
                        walk_phase: 0.0,
                        work_swing_phase: 0.0,
                        body_style,
                        hair_style,
                        skin_color,
                        hair_color,
                        outfit_color,
                        activity: ColonistActivity::Resting,
                        role_profile,
                        inventory,
                        carrying_kind: None,
                        carrying_amount: 0,
                        active_task: None,
                    });
                }
            }
        }

        core.ensure_leader_exists();
        core.world_dirty = true;
        self.core = Some(core);
    }

    /// Marca un área rectangular para tala, minería, almacén, cultivo, pared de madera o cancelar.
    #[func]
    pub fn designate_area(&mut self, mode: GString, x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
        let Some(core) = &mut self.core else {
            return 0;
        };
        core.designate_area(&mode.to_string(), x0, y0, x1, y1)
    }

    /// Avanza la simulación un paso de tiempo `delta`.
    #[func]
    pub fn step_simulation(&mut self, delta: f32) {
        if let Some(core) = &mut self.core {
            core.step(delta);
        }
    }

    /// Devuelve el estado actual de todos los colonos para sincronizar la escena Godot.
    #[func]
    pub fn get_colonists_snapshot(&self) -> VarArray {
        let mut out = VarArray::new();
        let Some(core) = &self.core else {
            return out;
        };

        for c in &core.colonists {
            let mut d = VarDictionary::new();
            d.set("id", &c.id.to_variant());
            d.set("name", &GString::from(&c.name).to_variant());
            d.set("role", &GString::from(&c.role).to_variant());
            d.set("is_leader", &c.is_leader.to_variant());
            d.set("x", &c.x.to_variant());
            d.set("y", &c.y.to_variant());
            d.set("facing_x", &c.facing_x.to_variant());
            d.set("facing_y", &c.facing_y.to_variant());
            d.set("health", &c.health.to_variant());
            d.set("mood", &c.mood.to_variant());
            d.set("energy", &(c.energy.round() as i32).to_variant());
            d.set("walk_phase", &c.walk_phase.to_variant());
            d.set("work_swing_phase", &c.work_swing_phase.to_variant());
            let is_moving = !c.path.is_empty()
                || matches!(
                    c.activity,
                    ColonistActivity::Wandering | ColonistActivity::Scouting | ColonistActivity::MovingToTask
                );
            let is_working = matches!(
                c.activity,
                ColonistActivity::Chopping
                    | ColonistActivity::Mining
                    | ColonistActivity::Building
                    | ColonistActivity::Farming
                    | ColonistActivity::Botanizing
            );
            d.set("is_moving", &is_moving.to_variant());
            d.set("is_working", &is_working.to_variant());
            d.set(
                "is_sleeping",
                &(c.activity == ColonistActivity::Sleeping).to_variant(),
            );
            d.set(
                "is_meeting",
                &(c.activity == ColonistActivity::ConveningMeeting).to_variant(),
            );
            d.set("activity", &GString::from(c.activity.label_es()).to_variant());
            d.set(
                "equipped_tool",
                &GString::from(c.inventory.main_hand.code()).to_variant(),
            );
            d.set(
                "equipped_tool_label",
                &GString::from(c.inventory.main_hand.label_es()).to_variant(),
            );
            d.set(
                "main_hand_tool",
                &GString::from(c.inventory.main_hand.code()).to_variant(),
            );
            d.set(
                "main_hand_tool_label",
                &GString::from(c.inventory.main_hand.label_es()).to_variant(),
            );
            d.set(
                "back_tool",
                &GString::from(c.inventory.back_tool.code()).to_variant(),
            );
            d.set(
                "back_tool_label",
                &GString::from(c.inventory.back_tool.label_es()).to_variant(),
            );
            d.set(
                "helmet_slot",
                &GString::from(&c.inventory.helmet_slot).to_variant(),
            );
            d.set(
                "armor_slot",
                &GString::from(&c.inventory.armor_slot).to_variant(),
            );
            d.set(
                "carrying_kind",
                &GString::from(c.carrying_kind.map_or("", |k| k.code())).to_variant(),
            );
            d.set(
                "carrying_label",
                &GString::from(c.carrying_kind.map_or("", |k| k.label_es())).to_variant(),
            );
            d.set("carrying_amount", &c.carrying_amount.to_variant());
            d.set("body_style", &c.body_style.to_variant());
            d.set("hair_style", &c.hair_style.to_variant());
            d.set("skin_color", &GString::from(&c.skin_color).to_variant());
            d.set("hair_color", &GString::from(&c.hair_color).to_variant());
            d.set("outfit_color", &GString::from(&c.outfit_color).to_variant());

            let mut roles_codes = VarArray::new();
            let mut roles_labels = VarArray::new();
            for &r in &c.role_profile.assigned {
                roles_codes.push(&GString::from(r.code()).to_variant());
                roles_labels.push(&GString::from(r.label_es()).to_variant());
            }
            let mut levels_dict = VarDictionary::new();
            let mut xp_dict = VarDictionary::new();
            for r in ColonyRole::ALL {
                levels_dict.set(r.code(), &c.role_profile.level_of(r).to_variant());
                xp_dict.set(r.code(), &c.role_profile.xp_of(r).to_variant());
            }
            d.set("assigned_roles", &roles_codes.to_variant());
            d.set("assigned_roles_labels", &roles_labels.to_variant());
            d.set("role_levels", &levels_dict.to_variant());
            d.set("role_xp", &xp_dict.to_variant());
            d.set("max_roles", &(MAX_ASSIGNED_ROLES as i32).to_variant());
            out.push(&d.to_variant());
        }

        out
    }

    /// Devuelve el detalle completo de los 8 roles y niveles (1..=10) de un colono para el panel de Trabajos.
    #[func]
    pub fn get_colonist_roles_detail(&self, colonist_id: i32) -> VarArray {
        let mut out = VarArray::new();
        let Some(core) = &self.core else {
            return out;
        };
        let Some(c) = core.colonists.iter().find(|col| col.id == colonist_id) else {
            return out;
        };
        for r in ColonyRole::ALL {
            let mut rd = VarDictionary::new();
            let lv = c.role_profile.level_of(r);
            let xp = c.role_profile.xp_of(r);
            let xp_next = xp_required_for_next_level(lv);
            let order_pos = c
                .role_profile
                .assigned
                .iter()
                .position(|&ar| ar == r)
                .map_or(0, |idx| (idx + 1) as i32);

            rd.set("code", &GString::from(r.code()).to_variant());
            rd.set("label", &GString::from(r.label_es()).to_variant());
            rd.set("description", &GString::from(r.description_es()).to_variant());
            rd.set("assigned", &(order_pos > 0).to_variant());
            rd.set("order", &order_pos.to_variant());
            rd.set("level", &lv.to_variant());
            rd.set("max_level", &MAX_ROLE_LEVEL.to_variant());
            rd.set("xp", &xp.to_variant());
            rd.set("xp_next", &xp_next.to_variant());
            rd.set(
                "effect_summary",
                &GString::from(effect_summary_es(r, lv)).to_variant(),
            );
            out.push(&rd.to_variant());
        }
        out
    }

    /// Asigna o quita un rol a un colono desde la interfaz táctil (máximo 5 roles por colono).
    #[func]
    pub fn set_colonist_role_enabled(
        &mut self,
        colonist_id: i32,
        role_code: GString,
        enabled: bool,
    ) -> VarDictionary {
        let mut res = VarDictionary::new();
        let Some(core) = &mut self.core else {
            res.set("ok", &false.to_variant());
            res.set("message", &GString::from("Simulación no iniciada.").to_variant());
            return res;
        };
        let Some(role) = ColonyRole::from_code(&role_code.to_string()) else {
            res.set("ok", &false.to_variant());
            res.set("message", &GString::from("Rol desconocido.").to_variant());
            return res;
        };
        match core.set_colonist_role_enabled(colonist_id, role, enabled) {
            Ok(msg) => {
                res.set("ok", &true.to_variant());
                res.set("message", &GString::from(msg).to_variant());
            }
            Err(err) => {
                res.set("ok", &false.to_variant());
                res.set("message", &GString::from(err).to_variant());
            }
        }
        res
    }

    /// Devuelve el estado del mundo (zonas, cultivos, construcciones, ítems en el piso,
    /// órdenes pendientes, líder e inventario) para renderizado y HUD en Godot.
    #[func]
    pub fn get_world_snapshot(&mut self, force_full: bool) -> VarDictionary {
        let mut out = VarDictionary::new();
        let Some(core) = &mut self.core else {
            return out;
        };

        let (total_inv, stored_inv) = core.compute_inventory();
        let leader_name = core
            .colonists
            .iter()
            .find(|c| c.is_leader)
            .map(|c| c.name.as_str())
            .unwrap_or("Sin líder");
        let pending_count = core
            .orders
            .iter()
            .filter(|o| o.status == OrderStatus::PendingBriefing)
            .count() as i32;

        let meeting_partner_name = core
            .colonists
            .iter()
            .find(|c| c.id == core.meeting_target_id)
            .map(|c| c.name.as_str())
            .unwrap_or("");

        out.set("dirty", &(core.world_dirty || force_full).to_variant());
        out.set("day", &core.day_count.to_variant());
        out.set("leader_name", &GString::from(leader_name).to_variant());
        out.set("meeting_active", &core.meeting_active.to_variant());
        out.set("meeting_partner_name", &GString::from(meeting_partner_name).to_variant());
        out.set("pending_briefing_count", &pending_count.to_variant());

        let mut inv = VarDictionary::new();
        inv.set("wood", &total_inv.wood.to_variant());
        inv.set("stone", &total_inv.stone.to_variant());
        inv.set("iron", &total_inv.iron.to_variant());
        inv.set("copper", &total_inv.copper.to_variant());
        inv.set("carrot_seeds", &total_inv.carrot_seeds.to_variant());
        inv.set("carrots", &total_inv.carrots.to_variant());
        inv.set("stored_wood", &stored_inv.wood.to_variant());
        inv.set("stored_carrots", &stored_inv.carrots.to_variant());
        out.set("inventory", &inv.to_variant());

        if core.world_dirty || force_full {
            core.world_dirty = false;
            out.set("props", &PackedInt32Array::from(core.props.as_slice()).to_variant());
            out.set("blocked", &PackedInt32Array::from(core.blocked.as_slice()).to_variant());
            out.set("zones", &PackedInt32Array::from(core.zones.as_slice()).to_variant());
            out.set("structures", &PackedInt32Array::from(core.structures.as_slice()).to_variant());

            let mut crops_arr = VarArray::new();
            for crop in &core.crops {
                let mut cd = VarDictionary::new();
                cd.set("x", &crop.x.to_variant());
                cd.set("y", &crop.y.to_variant());
                cd.set("stage", &(crop.stage as i32).to_variant());
                cd.set("progress", &crop.progress.to_variant());
                cd.set("tend_cooldown", &crop.tend_cooldown.to_variant());
                crops_arr.push(&cd.to_variant());
            }
            out.set("crops", &crops_arr.to_variant());

            let mut items_arr = VarArray::new();
            for it in &core.items {
                let mut id_dict = VarDictionary::new();
                id_dict.set("id", &it.id.to_variant());
                id_dict.set("kind", &GString::from(it.kind.code()).to_variant());
                id_dict.set("label", &GString::from(it.kind.label_es()).to_variant());
                id_dict.set("x", &it.x.to_variant());
                id_dict.set("y", &it.y.to_variant());
                id_dict.set("amount", &it.amount.to_variant());
                id_dict.set("stored", &it.stored.to_variant());
                items_arr.push(&id_dict.to_variant());
            }
            out.set("items", &items_arr.to_variant());

            let mut orders_arr = VarArray::new();
            for o in &core.orders {
                let mut od = VarDictionary::new();
                od.set("kind", &(o.kind as i32).to_variant());
                od.set("x", &o.x.to_variant());
                od.set("y", &o.y.to_variant());
                od.set("status", &(o.status as i32).to_variant());
                orders_arr.push(&od.to_variant());
            }
            out.set("orders", &orders_arr.to_variant());
        }

        let mut sfx_arr = VarArray::new();
        for ev in core.pending_sfx.drain(..) {
            sfx_arr.push(&GString::from(ev).to_variant());
        }
        out.set("sfx_events", &sfx_arr.to_variant());

        out
    }

    /// Empaqueta el estado completo para guardar en JSON desde Godot.
    #[func]
    pub fn export_save_state(&mut self) -> VarDictionary {
        let prev_dirty = self.core.as_ref().is_some_and(|c| c.world_dirty);
        let prev_sfx = self
            .core
            .as_ref()
            .map(|c| c.pending_sfx.clone())
            .unwrap_or_default();
        let mut state = self.get_world_snapshot(true);
        if let Some(core) = &mut self.core {
            core.world_dirty = prev_dirty;
            core.pending_sfx = prev_sfx;
        }
        state.set("colonists", &self.get_colonists_snapshot().to_variant());
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Colocar un árbol cerca del claro
        let idx = (16 * 32 + 20) as usize;
        core.props[idx] = 1; // OakTree
        core.blocked[idx] = 1;
        core.spawn_initial(3);

        let count = core.designate_area("chop", 20, 16, 20, 16);
        assert_eq!(count, 1);
        assert_eq!(core.orders[0].status, OrderStatus::PendingBriefing);

        // Avanzar unos segundos para que el líder reúna al grupo y apruebe
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

        // El árbol debe haberse talado y deben existir troncos en el almacén
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

        // Transferir Líder de Colonia a otro colono que tenga espacio
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
        // Dejar solo al colono 0 como Transportador nivel 1 y quitar otros roles para aislar prueba
        core.colonists[0].role_profile.assigned = vec![ColonyRole::Hauler];
        core.colonists[0].role_profile.set_skill(ColonyRole::Hauler, 1, 0.0);
        core.colonists[1].role_profile.assigned = vec![ColonyRole::ColonyLeader];

        core.designate_area("storage", 8, 12, 9, 12);
        core.approve_all_pending();
        core.spawn_item(ItemKind::StoneChunk, 14, 12, 6, false);

        // Avanzar lo justo para que recoja el primer viaje (capacidad nivel 1 = 2 uds)
        for _ in 0..80 {
            core.step(0.05);
            if core.colonists[0].carrying_amount > 0 {
                break;
            }
        }
        assert_eq!(core.colonists[0].carrying_amount, 2);
    }
}
