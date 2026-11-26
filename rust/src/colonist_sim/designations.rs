//! Gestión de designaciones rectangulares, zonas, almacén, mesas e inventario del mundo.
//!
//! # Lógica del módulo
//! Agrupa las operaciones sobre el mapa de la colonia:
//! - Marcar o cancelar áreas de tala, minería (granito, hierro, cobre y carbón),
//!   caza de fauna, almacenamiento, cultivos (zanahoria, papa, trigo y remolacha),
//!   muros, puertas y mesas (`designate_area`).
//! - Apilar y generar recursos sueltos en el piso (`spawn_item`).
//! - Contabilizar inventario total y almacenado (`compute_inventory`) y consumir recursos (`consume_item_from_world`).
//! - Buscar casillas de almacén (`find_storage_tile`), casillas cercanas para soltar piezas y gestionar colas de carpintería.

use super::core::SimulationCore;
use crate::colony_types::{
    CarpentryBenchState, CarpentryRecipe, ColonistActivity, ColonyInventory, ColonyOrder,
    CropCell, CropKind, CropStage, GroundItem, ItemKind, MasonryRecipe, OrderKind, OrderStatus,
    StructureKind, WorkshopStationState, ZoneKind,
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

    fn accumulate_item_into(inv: &mut ColonyInventory, kind: ItemKind, amount: i32) {
        match kind {
            ItemKind::WoodLogs => inv.wood += amount,
            ItemKind::StoneChunk => inv.stone += amount,
            ItemKind::IronOre => inv.iron += amount,
            ItemKind::CopperOre => inv.copper += amount,
            ItemKind::Coal => inv.coal += amount,
            ItemKind::CopperIngot => inv.copper_ingots += amount,
            ItemKind::CarrotSeed => inv.carrot_seeds += amount,
            ItemKind::Carrot => inv.carrots += amount,
            ItemKind::PotatoSeed => inv.potato_seeds += amount,
            ItemKind::Potato => inv.potatoes += amount,
            ItemKind::WheatSeed => inv.wheat_seeds += amount,
            ItemKind::Wheat => inv.wheat += amount,
            ItemKind::BeetSeed => inv.beet_seeds += amount,
            ItemKind::Beet => inv.beets += amount,
            ItemKind::AnimalBones => inv.animal_bones += amount,
            ItemKind::WoodPlank => inv.wood_planks += amount,
            ItemKind::WoodBucket => inv.wood_buckets += amount,
            ItemKind::WoodCup => inv.wood_cups += amount,
            ItemKind::WoodDoorItem => inv.wood_doors += amount,
            ItemKind::WoodCrate => inv.wood_crates += amount,
            ItemKind::StoneBricks => inv.stone_bricks += amount,
            ItemKind::StoneMortar => inv.stone_mortars += amount,
            ItemKind::StoneMillstone => inv.stone_millstones += amount,
            ItemKind::StoneStatuette => inv.stone_statuettes += amount,
            k if k.is_meat_cut() => inv.meat_cuts += amount,
            _ => {}
        }
    }

    /// Calcula el inventario actual contabilizando todos los recursos presentes en el mapa
    /// (almacenados + sueltos en el piso) y diferenciando los almacenados.
    pub fn compute_inventory(&self) -> (ColonyInventory, ColonyInventory) {
        let mut total = ColonyInventory::default();
        let mut stored = ColonyInventory::default();
        for it in &self.items {
            Self::accumulate_item_into(&mut total, it.kind, it.amount);
            if it.stored {
                Self::accumulate_item_into(&mut stored, it.kind, it.amount);
            }
        }
        for c in &self.colonists {
            if let Some(k) = c.carrying_kind {
                Self::accumulate_item_into(&mut total, k, c.carrying_amount);
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

    fn farm_crop_kind_for_mode(mode: &str) -> Option<CropKind> {
        match mode {
            "farm" | "farm_carrot" => Some(CropKind::Carrot),
            "farm_potato" => Some(CropKind::Potato),
            "farm_wheat" => Some(CropKind::Wheat),
            "farm_beet" => Some(CropKind::Beet),
            _ => None,
        }
    }

    /// Designa un rectángulo de órdenes o zonas sobre el mapa.
    /// Devuelve la cantidad de casillas u objetivos afectados.
    pub fn designate_area(&mut self, mode: &str, x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
        let min_x = x0.min(x1).clamp(1, self.map_width - 2);
        let max_x = x0.max(x1).clamp(1, self.map_width - 2);
        let min_y = y0.min(y1).clamp(1, self.map_height - 2);
        let max_y = y0.max(y1).clamp(1, self.map_height - 2);

        let mut affected = 0i32;

        if mode == "hunt" {
            let rx0 = min_x as f32 - 0.35;
            let rx1 = (max_x + 1) as f32 + 0.35;
            let ry0 = min_y as f32 - 0.35;
            let ry1 = (max_y + 1) as f32 + 0.35;
            for animal in &mut self.animals {
                if animal.x >= rx0
                    && animal.x <= rx1
                    && animal.y >= ry0
                    && animal.y <= ry1
                    && !self
                        .orders
                        .iter()
                        .any(|o| o.kind == OrderKind::HuntAnimal && o.target_id == animal.id)
                {
                    animal.marked_for_hunt = true;
                    let id = self.next_order_id;
                    self.next_order_id += 1;
                    self.orders.push(ColonyOrder {
                        id,
                        kind: OrderKind::HuntAnimal,
                        x: animal.x.floor() as i32,
                        y: animal.y.floor() as i32,
                        target_id: animal.id,
                        status: OrderStatus::PendingBriefing,
                        assigned_colonist_id: 0,
                    });
                    self.briefing_pending = true;
                    affected += 1;
                }
            }
            if affected > 0 {
                self.world_dirty = true;
            }
            return affected;
        }

        let farm_crop = Self::farm_crop_kind_for_mode(mode);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let Some(idx) = self.idx(x, y) else {
                    continue;
                };
                if let Some(crop_kind) = farm_crop {
                    let ground_kind = self.ground[idx];
                    if self.blocked[idx] == 0
                        && self.structures[idx] == StructureKind::None as i32
                        && ground_kind != 4
                        && ground_kind != 5
                    {
                        if self.zones[idx] != ZoneKind::Farm as i32 {
                            self.zones[idx] = ZoneKind::Farm as i32;
                            if !self.crops.iter().any(|c| c.x == x && c.y == y) {
                                self.crops.push(CropCell {
                                    x,
                                    y,
                                    crop_kind,
                                    stage: CropStage::None,
                                    progress: 0.0,
                                    tend_cooldown: 0.0,
                                    watered_timer: 0.0,
                                });
                            }
                            self.farm_approved = false;
                            self.briefing_pending = true;
                            affected += 1;
                        } else if let Some(existing) =
                            self.crops.iter_mut().find(|c| c.x == x && c.y == y)
                        {
                            if existing.crop_kind != crop_kind
                                && matches!(existing.stage, CropStage::None | CropStage::Tilled)
                            {
                                existing.crop_kind = crop_kind;
                                self.farm_approved = false;
                                self.briefing_pending = true;
                                affected += 1;
                            }
                        }
                    }
                    continue;
                }

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
                                target_id: 0,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "mine" => {
                        let p = self.props[idx];
                        if matches!(p, 4 | 5 | 7 | 8 | 9 | 10 | 11)
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
                                target_id: 0,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "storage" => {
                        if self.blocked[idx] == 0
                            && self.ground[idx] != 5
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
                    "build_wall" => {
                        if self.blocked[idx] == 0
                            && self.ground[idx] != 5
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
                                target_id: 0,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "build_door" => {
                        if self.blocked[idx] == 0
                            && self.ground[idx] != 5
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
                                target_id: 0,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "build_carpenter_bench" => {
                        if self.blocked[idx] == 0
                            && self.ground[idx] != 5
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
                                target_id: 0,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "build_well" => {
                        if self.blocked[idx] == 0
                            && self.ground[idx] != 5
                            && self.structures[idx] == StructureKind::None as i32
                            && !self
                                .orders
                                .iter()
                                .any(|o| o.x == x && o.y == y && o.kind == OrderKind::BuildWaterWell)
                        {
                            self.zones[idx] = ZoneKind::None as i32;
                            self.crops.retain(|c| !(c.x == x && c.y == y));
                            self.structures[idx] = StructureKind::WaterWellBlueprint as i32;
                            let id = self.next_order_id;
                            self.next_order_id += 1;
                            self.orders.push(ColonyOrder {
                                id,
                                kind: OrderKind::BuildWaterWell,
                                x,
                                y,
                                target_id: 0,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "build_mason_bench" => {
                        if self.blocked[idx] == 0
                            && self.ground[idx] != 5
                            && self.structures[idx] == StructureKind::None as i32
                            && !self.orders.iter().any(|o| {
                                o.x == x && o.y == y && o.kind == OrderKind::BuildMasonBench
                            })
                        {
                            self.zones[idx] = ZoneKind::None as i32;
                            self.crops.retain(|c| !(c.x == x && c.y == y));
                            self.structures[idx] = StructureKind::MasonBenchBlueprint as i32;
                            let id = self.next_order_id;
                            self.next_order_id += 1;
                            self.orders.push(ColonyOrder {
                                id,
                                kind: OrderKind::BuildMasonBench,
                                x,
                                y,
                                target_id: 0,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "build_furnace" => {
                        if self.blocked[idx] == 0
                            && self.ground[idx] != 5
                            && self.structures[idx] == StructureKind::None as i32
                            && !self.orders.iter().any(|o| {
                                o.x == x && o.y == y && o.kind == OrderKind::BuildStoneFurnace
                            })
                        {
                            self.zones[idx] = ZoneKind::None as i32;
                            self.crops.retain(|c| !(c.x == x && c.y == y));
                            self.structures[idx] = StructureKind::StoneFurnaceBlueprint as i32;
                            let id = self.next_order_id;
                            self.next_order_id += 1;
                            self.orders.push(ColonyOrder {
                                id,
                                kind: OrderKind::BuildStoneFurnace,
                                x,
                                y,
                                target_id: 0,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "build_stone_wall" => {
                        if self.blocked[idx] == 0
                            && self.ground[idx] != 5
                            && self.structures[idx] == StructureKind::None as i32
                            && !self
                                .orders
                                .iter()
                                .any(|o| o.x == x && o.y == y && o.kind == OrderKind::BuildStoneWall)
                        {
                            self.zones[idx] = ZoneKind::None as i32;
                            self.crops.retain(|c| !(c.x == x && c.y == y));
                            self.structures[idx] = StructureKind::StoneWallBlueprint as i32;
                            let id = self.next_order_id;
                            self.next_order_id += 1;
                            self.orders.push(ColonyOrder {
                                id,
                                kind: OrderKind::BuildStoneWall,
                                x,
                                y,
                                target_id: 0,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "build_stone_floor" => {
                        if self.blocked[idx] == 0
                            && self.ground[idx] != 5
                            && self.structures[idx] == StructureKind::None as i32
                            && !self.orders.iter().any(|o| {
                                o.x == x && o.y == y && o.kind == OrderKind::BuildStoneFloor
                            })
                        {
                            self.zones[idx] = ZoneKind::None as i32;
                            self.crops.retain(|c| !(c.x == x && c.y == y));
                            self.structures[idx] = StructureKind::StoneFloorBlueprint as i32;
                            let id = self.next_order_id;
                            self.next_order_id += 1;
                            self.orders.push(ColonyOrder {
                                id,
                                kind: OrderKind::BuildStoneFloor,
                                x,
                                y,
                                target_id: 0,
                                status: OrderStatus::PendingBriefing,
                                assigned_colonist_id: 0,
                            });
                            self.briefing_pending = true;
                            affected += 1;
                        }
                    }
                    "build_stone_brazier" => {
                        if self.blocked[idx] == 0
                            && self.ground[idx] != 5
                            && self.structures[idx] == StructureKind::None as i32
                            && !self.orders.iter().any(|o| {
                                o.x == x && o.y == y && o.kind == OrderKind::BuildStoneBrazier
                            })
                        {
                            self.zones[idx] = ZoneKind::None as i32;
                            self.crops.retain(|c| !(c.x == x && c.y == y));
                            self.structures[idx] = StructureKind::StoneBrazierBlueprint as i32;
                            let id = self.next_order_id;
                            self.next_order_id += 1;
                            self.orders.push(ColonyOrder {
                                id,
                                kind: OrderKind::BuildStoneBrazier,
                                x,
                                y,
                                target_id: 0,
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
                                || s == StructureKind::WaterWellBlueprint as i32
                                || s == StructureKind::MasonBenchBlueprint as i32
                                || s == StructureKind::StoneFurnaceBlueprint as i32
                                || s == StructureKind::StoneWallBlueprint as i32
                                || s == StructureKind::StoneFloorBlueprint as i32
                                || s == StructureKind::StoneBrazierBlueprint as i32
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

        if mode == "cancel" {
            let rx0 = min_x as f32 - 0.35;
            let rx1 = (max_x + 1) as f32 + 0.35;
            let ry0 = min_y as f32 - 0.35;
            let ry1 = (max_y + 1) as f32 + 0.35;
            let mut cancelled_ids: Vec<i32> = Vec::new();
            for animal in &mut self.animals {
                if animal.marked_for_hunt
                    && animal.x >= rx0
                    && animal.x <= rx1
                    && animal.y >= ry0
                    && animal.y <= ry1
                {
                    animal.marked_for_hunt = false;
                    cancelled_ids.push(animal.id);
                    affected += 1;
                }
            }
            if !cancelled_ids.is_empty() {
                self.orders.retain(|o| {
                    !(o.kind == OrderKind::HuntAnimal && cancelled_ids.contains(&o.target_id))
                });
                for c in &mut self.colonists {
                    if let Some(t) = &c.active_task {
                        if t.kind == OrderKind::HuntAnimal && cancelled_ids.contains(&t.item_id) {
                            c.active_task = None;
                            c.path.clear();
                            c.activity = ColonistActivity::Resting;
                        }
                    }
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
                            && t.kind != OrderKind::MasonryJob
                            && t.kind != OrderKind::HuntAnimal
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

    pub fn find_workshop_near(&self, tile_x: i32, tile_y: i32) -> Option<&WorkshopStationState> {
        if let Some(exact) = self
            .workshops
            .iter()
            .find(|w| w.x == tile_x && w.y == tile_y)
        {
            return Some(exact);
        }
        self.workshops
            .iter()
            .find(|w| (w.x - tile_x).abs() <= 1 && (w.y - tile_y).abs() <= 1)
    }

    pub fn queue_masonry_recipe(
        &mut self,
        workshop_id: i32,
        recipe: MasonryRecipe,
    ) -> Result<(), &'static str> {
        let Some(ws) = self.workshops.iter_mut().find(|w| w.id == workshop_id) else {
            return Err("Taller de cantería u horno no encontrado.");
        };
        if recipe.station() != ws.kind {
            return Err("Esta receta pertenece a otra estación de trabajo.");
        }
        if ws.queue.len() >= MAX_CARPENTRY_QUEUE {
            return Err("La cola de este taller está llena (máx. 12 piezas).");
        }
        ws.queue.push(recipe);
        self.world_dirty = true;
        Ok(())
    }

    pub fn cancel_masonry_queue_item(&mut self, workshop_id: i32, queue_index: usize) -> bool {
        let Some(ws) = self.workshops.iter_mut().find(|w| w.id == workshop_id) else {
            return false;
        };
        if queue_index >= ws.queue.len() {
            return false;
        }
        ws.queue.remove(queue_index);
        for c in &mut self.colonists {
            if let Some(task) = &mut c.active_task {
                if task.kind == OrderKind::MasonryJob && task.order_id == workshop_id {
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
