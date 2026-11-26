//! Gestión de designaciones rectangulares, zonas, almacén, mesas e inventario del mundo.
//!
//! # Lógica del módulo
//! Agrupa las operaciones sobre el mapa de la colonia:
//! - Marcar o cancelar áreas de tala, minería, almacenamiento, cultivo, muros, puertas y mesas (`designate_area`).
//! - Apilar y generar recursos sueltos en el piso (`spawn_item`).
//! - Contabilizar inventario total y almacenado (`compute_inventory`) y consumir recursos (`consume_item_from_world`).
//! - Buscar casillas de almacén (`find_storage_tile`), casillas cercanas para soltar piezas y gestionar colas de carpintería.

use super::core::SimulationCore;
use crate::colony_types::{
    CarpentryBenchState, CarpentryRecipe, ColonistActivity, ColonyInventory, ColonyOrder,
    CropCell, CropStage, GroundItem, ItemKind, OrderKind, OrderStatus, StructureKind, ZoneKind,
};

pub const MAX_CARPENTRY_QUEUE: usize = 12;

impl SimulationCore {
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
                ItemKind::WoodPlank => {
                    total.wood_planks += it.amount;
                    if it.stored {
                        stored.wood_planks += it.amount;
                    }
                }
                ItemKind::WoodBucket => {
                    total.wood_buckets += it.amount;
                    if it.stored {
                        stored.wood_buckets += it.amount;
                    }
                }
                ItemKind::WoodDoorItem => {
                    total.wood_doors += it.amount;
                    if it.stored {
                        stored.wood_doors += it.amount;
                    }
                }
                ItemKind::WoodCrate => {
                    total.wood_crates += it.amount;
                    if it.stored {
                        stored.wood_crates += it.amount;
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
                    ItemKind::WoodPlank => total.wood_planks += c.carrying_amount,
                    ItemKind::WoodBucket => total.wood_buckets += c.carrying_amount,
                    ItemKind::WoodDoorItem => total.wood_doors += c.carrying_amount,
                    ItemKind::WoodCrate => total.wood_crates += c.carrying_amount,
                }
            }
        }
        (total, stored)
    }

    pub(super) fn available_ground_item(&self, kind: ItemKind) -> i32 {
        self.items
            .iter()
            .filter(|i| i.kind == kind)
            .map(|i| i.amount)
            .sum()
    }

    pub(super) fn consume_item_from_world(&mut self, kind: ItemKind, mut needed: i32) -> bool {
        let available = self.available_ground_item(kind);
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
                    "build_door" => {
                        if self.blocked[idx] == 0
                            && self.structures[idx] == StructureKind::None as i32
                            && !self
                                .orders
                                .iter()
                                .any(|o| o.x == x && o.y == y && o.kind == OrderKind::BuildWoodDoor)
                        {
                            self.zones[idx] = ZoneKind::None as i32;
                            self.crops.retain(|c| !(c.x == x && c.y == y));
                            self.structures[idx] = StructureKind::WoodDoorBlueprint as i32;
                            let id = self.next_order_id;
                            self.next_order_id += 1;
                            self.orders.push(ColonyOrder {
                                id,
                                kind: OrderKind::BuildWoodDoor,
                                x,
                                y,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "build_carpenter_bench" => {
                        if self.blocked[idx] == 0
                            && self.structures[idx] == StructureKind::None as i32
                            && !self.orders.iter().any(|o| {
                                o.x == x && o.y == y && o.kind == OrderKind::BuildCarpenterBench
                            })
                        {
                            self.zones[idx] = ZoneKind::None as i32;
                            self.crops.retain(|c| !(c.x == x && c.y == y));
                            self.structures[idx] = StructureKind::CarpenterBenchBlueprint as i32;
                            let id = self.next_order_id;
                            self.next_order_id += 1;
                            self.orders.push(ColonyOrder {
                                id,
                                kind: OrderKind::BuildCarpenterBench,
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
                        if matches!(
                            self.structures[idx],
                            s if s == StructureKind::WoodWallBlueprint as i32
                                || s == StructureKind::WoodDoorBlueprint as i32
                                || s == StructureKind::CarpenterBenchBlueprint as i32
                        ) {
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
                        if in_rect
                            && t.kind != OrderKind::HaulItem
                            && t.kind != OrderKind::CarpentryJob
                        {
                            c.active_task = None;
                            c.path.clear();
                            c.activity = ColonistActivity::Resting;
                        }
                    }
                }
            }
            self.world_dirty = true;
            if !self
                .orders
                .iter()
                .any(|o| o.status == OrderStatus::PendingBriefing)
                && self.storage_approved
                && self.farm_approved
            {
                self.briefing_pending = false;
            }
        }
        affected
    }

    pub fn find_bench_near(&self, tile_x: i32, tile_y: i32) -> Option<&CarpentryBenchState> {
        if let Some(exact) = self.benches.iter().find(|b| b.x == tile_x && b.y == tile_y) {
            return Some(exact);
        }
        self.benches
            .iter()
            .find(|b| (b.x - tile_x).abs() <= 1 && (b.y - tile_y).abs() <= 1)
    }

    pub fn queue_carpentry_recipe(
        &mut self,
        bench_id: i32,
        recipe: CarpentryRecipe,
    ) -> Result<(), &'static str> {
        let Some(bench) = self.benches.iter_mut().find(|b| b.id == bench_id) else {
            return Err("Mesa de carpintero no encontrada.");
        };
        if bench.queue.len() >= MAX_CARPENTRY_QUEUE {
            return Err("La cola de esta mesa está llena (máx. 12 piezas).");
        }
        bench.queue.push(recipe);
        self.world_dirty = true;
        Ok(())
    }

    pub fn cancel_carpentry_queue_item(&mut self, bench_id: i32, queue_index: usize) -> bool {
        let Some(bench) = self.benches.iter_mut().find(|b| b.id == bench_id) else {
            return false;
        };
        if queue_index >= bench.queue.len() {
            return false;
        }
        bench.queue.remove(queue_index);
        for c in &mut self.colonists {
            if let Some(task) = &mut c.active_task {
                if task.kind == OrderKind::CarpentryJob && task.order_id == bench_id {
                    let q_i = queue_index as i32;
                    if task.phase == q_i {
                        c.active_task = None;
                        c.path.clear();
                        c.activity = ColonistActivity::Resting;
                    } else if task.phase > q_i {
                        task.phase -= 1;
                    }
                }
            }
        }
        self.world_dirty = true;
        true
    }

    pub(super) fn find_drop_tile_near(&self, origin_x: i32, origin_y: i32) -> (i32, i32) {
        let offsets = [
            (0, 1),
            (1, 0),
            (-1, 0),
            (0, -1),
            (1, 1),
            (-1, 1),
            (1, -1),
            (-1, -1),
        ];
        for (ox, oy) in offsets {
            let nx = origin_x + ox;
            let ny = origin_y + oy;
            if let Some(idx) = self.idx(nx, ny) {
                if nx >= 1
                    && ny >= 1
                    && nx < self.map_width - 1
                    && ny < self.map_height - 1
                    && self.blocked[idx] == 0
                {
                    return (nx, ny);
                }
            }
        }
        (origin_x, origin_y)
    }

    pub(super) fn find_storage_tile(
        &self,
        prefer_near_x: i32,
        prefer_near_y: i32,
    ) -> Option<(i32, i32)> {
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
}
