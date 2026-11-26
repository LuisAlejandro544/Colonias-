//! Asignación, movimiento A* y ejecución de tareas de los colonos.
//!
//! # Lógica del módulo
//! Controla el ciclo operativo de trabajo de cada colono:
//! - Liberación de tareas activas (`release_colonist_task`).
//! - Búsqueda de hidratación (`try_start_drink_task`) priorizando pozos de agua
//!   profunda y recurriendo al río más cercano si no hay pozo (sin usar cubos con
//!   agua cruda del río).
//! - Selección de tareas aprobadas según los roles asignados, la Temporada de
//!   Cultivos y requisitos de Inteligencia (`assign_task_to_colonist`).
//! - Desplazamiento paso a paso por rutas A* (`step_colonist_movement`).
//! - Ejecución de tala, minería (granito, hierro, cobre y carbón), construcción
//!   (muros, puertas, mesas y pozos), carpintería, cultivo multi-especie, caza,
//!   bebida y acarreo (`execute_active_task`).

use super::core::{
    SimulationCore, BRICKS_PER_BRAZIER, BRICKS_PER_FURNACE, BRICKS_PER_STONE_FLOOR,
    BRICKS_PER_STONE_WALL, BUCKETS_PER_WELL, COAL_PER_BRAZIER, DOOR_ITEMS_PER_DOOR,
    STONE_PER_BRAZIER, STONE_PER_FURNACE, STONE_PER_MASON_BENCH, STONE_PER_STONE_WALL,
    STONE_PER_WELL, WOOD_PER_CARPENTER_BENCH, WOOD_PER_WALL, WOOD_PER_WELL,
};
use crate::colony_types::{
    ActiveTask, CarpentryBenchState, CarpentryRecipe, ColonistActivity, ItemKind, MasonryRecipe,
    OrderKind, OrderStatus, StructureKind, WorkshopKind, WorkshopStationState,
};
use crate::pathfinding::find_path;
use crate::roles::ColonyRole;
use crate::stats::StatKind;
use crate::tools::CopperTool;
use crate::world_gen::GroundKind;

impl SimulationCore {
    pub(super) fn release_colonist_task(&mut self, c_idx: usize) {
        if let Some(task) = self.colonists[c_idx].active_task.take() {
            for o in &mut self.orders {
                if o.id == task.order_id {
                    o.assigned_colonist_id = 0;
                }
            }
        }
        self.colonists[c_idx].path.clear();
    }

    pub(super) fn assign_task_to_colonist(&mut self, c_idx: usize) {
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
                let kind = self.colonists[c_idx].carrying_kind.take();
                let amt = self.colonists[c_idx].carrying_amount;
                self.colonists[c_idx].carrying_amount = 0;
                if let Some(k) = kind {
                    self.spawn_item(k, cx, cy, amt, false);
                }
            }
        }

        // 1b. Si tiene sed y manos libres, buscar pozo o río cercano antes de tomar otro oficio.
        if self.colonists[c_idx].hydration <= 36.0 && self.try_start_drink_task(c_idx) {
            return;
        }

        // 2. Evaluar los roles asignados al colono (máximo 5) en orden de prioridad.
        let assigned_roles = self.colonists[c_idx].role_profile.assigned.clone();
        if assigned_roles.is_empty() {
            return;
        }
        let (total_inv, _) = self.compute_inventory();
        let in_crop_season = self.is_crop_season_active();

        for role in assigned_roles {
            match role {
                ColonyRole::Constructor | ColonyRole::Lumberjack | ColonyRole::Miner => {
                    let mut chosen_order_idx: Option<usize> = None;
                    let mut best_dist = i32::MAX;

                    for (oi, order) in self.orders.iter().enumerate() {
                        if order.status != OrderStatus::Approved || order.assigned_colonist_id != 0 {
                            continue;
                        }
                        let matches_role = match role {
                            ColonyRole::Lumberjack => order.kind == OrderKind::ChopTree,
                            ColonyRole::Miner => order.kind == OrderKind::MineRock,
                            ColonyRole::Constructor => matches!(
                                order.kind,
                                OrderKind::BuildWoodWall
                                    | OrderKind::BuildWoodDoor
                                    | OrderKind::BuildCarpenterBench
                                    | OrderKind::BuildWaterWell
                                    | OrderKind::BuildMasonBench
                                    | OrderKind::BuildStoneFurnace
                                    | OrderKind::BuildStoneWall
                                    | OrderKind::BuildStoneFloor
                                    | OrderKind::BuildStoneBrazier
                            ),
                            _ => false,
                        };
                        if !matches_role {
                            continue;
                        }
                        let has_materials = match order.kind {
                            OrderKind::BuildWoodWall => total_inv.wood >= WOOD_PER_WALL,
                            OrderKind::BuildWoodDoor => total_inv.wood_doors >= DOOR_ITEMS_PER_DOOR,
                            OrderKind::BuildCarpenterBench => {
                                total_inv.wood >= WOOD_PER_CARPENTER_BENCH
                            }
                            OrderKind::BuildWaterWell => {
                                total_inv.wood >= WOOD_PER_WELL
                                    && total_inv.stone >= STONE_PER_WELL
                                    && total_inv.wood_buckets >= BUCKETS_PER_WELL
                            }
                            OrderKind::BuildMasonBench => total_inv.stone >= STONE_PER_MASON_BENCH,
                            OrderKind::BuildStoneFurnace => {
                                total_inv.stone >= STONE_PER_FURNACE
                                    && total_inv.stone_bricks >= BRICKS_PER_FURNACE
                            }
                            OrderKind::BuildStoneWall => {
                                total_inv.stone >= STONE_PER_STONE_WALL
                                    && total_inv.stone_bricks >= BRICKS_PER_STONE_WALL
                            }
                            OrderKind::BuildStoneFloor => {
                                total_inv.stone_bricks >= BRICKS_PER_STONE_FLOOR
                            }
                            OrderKind::BuildStoneBrazier => {
                                total_inv.stone >= STONE_PER_BRAZIER
                                    && total_inv.stone_bricks >= BRICKS_PER_BRAZIER
                                    && total_inv.coal >= COAL_PER_BRAZIER
                            }
                            _ => true,
                        };
                        if !has_materials {
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
                ColonyRole::Carpenter => {
                    let mut chosen: Option<(i32, i32, i32, usize, CarpentryRecipe)> = None;
                    let mut best_dist = i32::MAX;

                    for bench in &self.benches {
                        let bench_busy = self.colonists.iter().any(|col| {
                            col.active_task.as_ref().is_some_and(|t| {
                                t.kind == OrderKind::CarpentryJob && t.order_id == bench.id
                            })
                        });
                        if bench_busy || bench.queue.is_empty() {
                            continue;
                        }

                        // Elegir la primera receta en cola que este colono pueda fabricar por INT y materiales.
                        let eligible = bench.queue.iter().enumerate().find(|&(_, &recipe)| {
                            if !self.colonists[c_idx]
                                .stats
                                .can_craft_complexity(recipe.min_intelligence())
                            {
                                return false;
                            }
                            let (req_item, req_amt) = recipe.input_cost();
                            self.available_ground_item(req_item) >= req_amt
                        });

                        if let Some((q_idx, &recipe)) = eligible {
                            let d = (bench.x - cx).abs() + (bench.y - cy).abs();
                            if d < best_dist {
                                best_dist = d;
                                chosen = Some((bench.id, bench.x, bench.y, q_idx, recipe));
                            }
                        }
                    }

                    if let Some((bench_id, bx, by, q_idx, recipe)) = chosen {
                        if let Some(path) = find_path(
                            self.map_width,
                            self.map_height,
                            &self.blocked,
                            cx,
                            cy,
                            bx,
                            by,
                            true,
                        ) {
                            self.colonists[c_idx].active_task = Some(ActiveTask {
                                order_id: bench_id,
                                kind: OrderKind::CarpentryJob,
                                target_x: bx,
                                target_y: by,
                                item_id: recipe as i32,
                                work_timer: 0.0,
                                phase: q_idx as i32,
                            });
                            self.colonists[c_idx].path = path;
                            self.colonists[c_idx].activity = ColonistActivity::MovingToTask;
                            self.colonists[c_idx]
                                .inventory
                                .equip_main_hand(CopperTool::CopperAxe);
                            return;
                        }
                    }
                }
                ColonyRole::Mason => {
                    let mut chosen: Option<(i32, i32, i32, usize, MasonryRecipe)> = None;
                    let mut best_dist = i32::MAX;

                    for ws in &self.workshops {
                        let ws_busy = self.colonists.iter().any(|col| {
                            col.active_task.as_ref().is_some_and(|t| {
                                t.kind == OrderKind::MasonryJob && t.order_id == ws.id
                            })
                        });
                        if ws_busy || ws.queue.is_empty() {
                            continue;
                        }

                        let eligible = ws.queue.iter().enumerate().find(|&(_, &recipe)| {
                            if !self.colonists[c_idx]
                                .stats
                                .can_craft_complexity(recipe.min_intelligence())
                            {
                                return false;
                            }
                            let (in1, amt1) = recipe.input_cost_1();
                            if self.available_ground_item(in1) < amt1 {
                                return false;
                            }
                            if let Some((in2, amt2)) = recipe.input_cost_2() {
                                if self.available_ground_item(in2) < amt2 {
                                    return false;
                                }
                            }
                            true
                        });

                        if let Some((q_idx, &recipe)) = eligible {
                            let d = (ws.x - cx).abs() + (ws.y - cy).abs();
                            if d < best_dist {
                                best_dist = d;
                                chosen = Some((ws.id, ws.x, ws.y, q_idx, recipe));
                            }
                        }
                    }

                    if let Some((ws_id, wx, wy, q_idx, recipe)) = chosen {
                        if let Some(path) = find_path(
                            self.map_width,
                            self.map_height,
                            &self.blocked,
                            cx,
                            cy,
                            wx,
                            wy,
                            true,
                        ) {
                            self.colonists[c_idx].active_task = Some(ActiveTask {
                                order_id: ws_id,
                                kind: OrderKind::MasonryJob,
                                target_x: wx,
                                target_y: wy,
                                item_id: recipe as i32,
                                work_timer: 0.0,
                                phase: q_idx as i32,
                            });
                            self.colonists[c_idx].path = path;
                            self.colonists[c_idx].activity = ColonistActivity::MovingToTask;
                            self.colonists[c_idx]
                                .inventory
                                .equip_main_hand(CopperTool::CopperPickaxe);
                            return;
                        }
                    }
                }
                ColonyRole::Farmer => {
                    if !self.farm_approved {
                        continue;
                    }
                    // Fuera de temporada aún pueden sembrar/labrar, pero tienen pocas ganas (~20%).
                    if in_crop_season
                        || ((self.day_clock * 4.0) as i32 + cid).rem_euclid(5) == 0
                    {
                        if let Some((fx, fy)) = self.pick_farmer_crop(cx, cy) {
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
                                    phase: 0,
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
                }
                ColonyRole::Botanist => {
                    if !self.farm_approved {
                        continue;
                    }
                    // Si hay cubos de madera y cultivos secos, priorizar el riego incluso fuera de temporada.
                    if total_inv.wood_buckets >= 1 {
                        if let Some((crop_x, crop_y)) = self.pick_waterable_crop(cx, cy) {
                            if let Some((wx, wy, path)) =
                                self.find_water_source_for_irrigation(cx, cy)
                            {
                                let packed_crop = crop_y * self.map_width + crop_x;
                                self.colonists[c_idx].active_task = Some(ActiveTask {
                                    order_id: 0,
                                    kind: OrderKind::WaterCrops,
                                    target_x: wx,
                                    target_y: wy,
                                    item_id: packed_crop,
                                    work_timer: 0.0,
                                    phase: 0,
                                });
                                self.colonists[c_idx].path = path;
                                self.colonists[c_idx].activity = ColonistActivity::MovingToTask;
                                self.colonists[c_idx].inventory.stow_main_hand();
                                return;
                            }
                        }
                    }
                    // Fuera de temporada los botánicos también pierden motivación para sembrar/cuidar.
                    if !in_crop_season
                        && ((self.day_clock * 4.0) as i32 + cid + 2).rem_euclid(5) != 0
                    {
                        continue;
                    }
                    if let Some((fx, fy)) = self.pick_botanist_crop(cx, cy, &total_inv) {
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
                                phase: 1,
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
                ColonyRole::Scout | ColonyRole::ColonyLeader => {
                    if role == ColonyRole::ColonyLeader && self.meeting_active {
                        continue;
                    }
                    let mut chosen: Option<(usize, i32, i32)> = None;
                    let mut best_dist = i32::MAX;

                    for (oi, order) in self.orders.iter().enumerate() {
                        if order.kind != OrderKind::HuntAnimal
                            || order.status != OrderStatus::Approved
                            || order.assigned_colonist_id != 0
                        {
                            continue;
                        }
                        let Some(animal) = self.animals.iter().find(|a| a.id == order.target_id)
                        else {
                            continue;
                        };
                        let ax = animal.x.floor() as i32;
                        let ay = animal.y.floor() as i32;
                        let d = (ax - cx).abs() + (ay - cy).abs();
                        if d < best_dist {
                            best_dist = d;
                            chosen = Some((oi, ax, ay));
                        }
                    }

                    if let Some((oi, ax, ay)) = chosen {
                        let order_id = self.orders[oi].id;
                        let animal_id = self.orders[oi].target_id;
                        if let Some(path) = find_path(
                            self.map_width,
                            self.map_height,
                            &self.blocked,
                            cx,
                            cy,
                            ax,
                            ay,
                            true,
                        ) {
                            self.orders[oi].assigned_colonist_id = cid;
                            self.colonists[c_idx].active_task = Some(ActiveTask {
                                order_id,
                                kind: OrderKind::HuntAnimal,
                                target_x: ax,
                                target_y: ay,
                                item_id: animal_id,
                                work_timer: 0.0,
                                phase: 0,
                            });
                            self.colonists[c_idx].path = path;
                            self.colonists[c_idx].activity = ColonistActivity::MovingToTask;
                            self.colonists[c_idx]
                                .inventory
                                .equip_main_hand(CopperTool::CopperSword);
                            return;
                        }
                    }
                }
            }
        }
    }

    fn find_best_drink_target(&self, cx: i32, cy: i32) -> Option<(i32, i32, i32, Vec<(i32, i32)>)> {
        // 1. Prioridad absoluta: pozos de agua profunda (aceptados con agrado por los colonos).
        let mut wells: Vec<(i32, i32, i32)> = Vec::new();
        for y in 1..self.map_height - 1 {
            for x in 1..self.map_width - 1 {
                let idx = (y * self.map_width + x) as usize;
                if self.structures[idx] == StructureKind::WaterWell as i32 {
                    let d = (x - cx).abs() + (y - cy).abs();
                    wells.push((d, x, y));
                }
            }
        }
        wells.sort_by_key(|&(d, _, _)| d);
        for (_, wx, wy) in wells {
            if let Some(path) =
                find_path(self.map_width, self.map_height, &self.blocked, cx, cy, wx, wy, true)
            {
                return Some((wx, wy, 1, path));
            }
        }

        // 2. Alternativa si no hay pozo disponible: acudir a la orilla del río más cercano.
        // Los colonos jamás aceptarán beber agua cruda del río almacenada en cubos.
        let mut river_tiles: Vec<(i32, i32, i32)> = Vec::new();
        for y in 1..self.map_height - 1 {
            for x in 1..self.map_width - 1 {
                let idx = (y * self.map_width + x) as usize;
                if self.ground[idx] == GroundKind::River as i32 {
                    let has_bank = [(0, 1), (1, 0), (-1, 0), (0, -1)].iter().any(|&(ox, oy)| {
                        !Self::tile_blocked(self.map_width, self.map_height, &self.blocked, x + ox, y + oy)
                    });
                    if has_bank {
                        let d = (x - cx).abs() + (y - cy).abs();
                        river_tiles.push((d, x, y));
                    }
                }
            }
        }
        river_tiles.sort_by_key(|&(d, _, _)| d);
        for (_, rx, ry) in river_tiles.into_iter().take(16) {
            if let Some(path) =
                find_path(self.map_width, self.map_height, &self.blocked, cx, cy, rx, ry, true)
            {
                return Some((rx, ry, 0, path));
            }
        }
        None
    }

    pub(super) fn try_start_drink_task(&mut self, c_idx: usize) -> bool {
        let cx = self.colonists[c_idx].x.floor() as i32;
        let cy = self.colonists[c_idx].y.floor() as i32;
        let Some((tx, ty, phase, path)) = self.find_best_drink_target(cx, cy) else {
            return false;
        };
        self.colonists[c_idx].inventory.stow_main_hand();
        self.colonists[c_idx].active_task = Some(ActiveTask {
            order_id: 0,
            kind: OrderKind::DrinkWater,
            target_x: tx,
            target_y: ty,
            item_id: 0,
            work_timer: 0.0,
            phase,
        });
        self.colonists[c_idx].path = path;
        self.colonists[c_idx].activity = ColonistActivity::MovingToTask;
        true
    }

    pub(super) fn try_interrupt_for_drink(&mut self, c_idx: usize) -> bool {
        let cx = self.colonists[c_idx].x.floor() as i32;
        let cy = self.colonists[c_idx].y.floor() as i32;
        let Some((tx, ty, phase, path)) = self.find_best_drink_target(cx, cy) else {
            return false;
        };
        self.release_colonist_task(c_idx);
        self.colonists[c_idx].inventory.stow_main_hand();
        self.colonists[c_idx].active_task = Some(ActiveTask {
            order_id: 0,
            kind: OrderKind::DrinkWater,
            target_x: tx,
            target_y: ty,
            item_id: 0,
            work_timer: 0.0,
            phase,
        });
        self.colonists[c_idx].path = path;
        self.colonists[c_idx].activity = ColonistActivity::MovingToTask;
        true
    }

    pub(super) fn step_colonist_movement(&mut self, c_idx: usize, dt: f32) -> bool {
        let cur_tx = self.colonists[c_idx].x.floor() as i32;
        let cur_ty = self.colonists[c_idx].y.floor() as i32;
        let on_stone_floor = self
            .idx(cur_tx, cur_ty)
            .is_some_and(|i| self.structures[i] == StructureKind::StoneFloor as i32);
        let floor_mult = if on_stone_floor { 1.18 } else { 1.0 };

        let c = &mut self.colonists[c_idx];
        let role_speed_mult = match c.activity {
            ColonistActivity::Hauling => c.role_profile.haul_speed_mult(),
            ColonistActivity::ConveningMeeting => c.role_profile.leader_meeting_speed_mult(),
            ColonistActivity::Scouting | ColonistActivity::Hunting => {
                c.role_profile.scout_speed_mult()
            }
            _ => 1.0,
        };
        let dex_mult = c.stats.move_speed_mult();
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
            let effective_speed = c.speed * role_speed_mult * dex_mult * floor_mult;
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

    pub(super) fn execute_active_task(&mut self, c_idx: usize, dt: f32) {
        let Some(mut task) = self.colonists[c_idx].active_task.clone() else {
            return;
        };

        // Seguimiento dinámico si el objetivo es un animal en movimiento.
        if task.kind == OrderKind::HuntAnimal {
            let Some((ax_f, ay_f)) = self
                .animals
                .iter()
                .find(|a| a.id == task.item_id)
                .map(|a| (a.x, a.y))
            else {
                self.orders.retain(|o| o.id != task.order_id);
                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].path.clear();
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                return;
            };

            let cx = self.colonists[c_idx].x;
            let cy = self.colonists[c_idx].y;
            let dist_animal = ((ax_f - cx).powi(2) + (ay_f - cy).powi(2)).sqrt();
            if dist_animal <= 1.75 {
                self.colonists[c_idx].path.clear();
                task.target_x = ax_f.floor() as i32;
                task.target_y = ay_f.floor() as i32;
            } else {
                let target_drift = ((ax_f - (task.target_x as f32 + 0.5)).powi(2)
                    + (ay_f - (task.target_y as f32 + 0.5)).powi(2))
                .sqrt();
                if self.colonists[c_idx].path.is_empty() || target_drift > 2.1 {
                    let cur_tx = cx.floor() as i32;
                    let cur_ty = cy.floor() as i32;
                    let ax_i = ax_f.floor() as i32;
                    let ay_i = ay_f.floor() as i32;
                    if let Some(new_path) = find_path(
                        self.map_width,
                        self.map_height,
                        &self.blocked,
                        cur_tx,
                        cur_ty,
                        ax_i,
                        ay_i,
                        true,
                    ) {
                        task.target_x = ax_i;
                        task.target_y = ay_i;
                        self.colonists[c_idx].path = new_path;
                        self.colonists[c_idx].active_task = Some(task.clone());
                    } else {
                        self.release_colonist_task(c_idx);
                        self.colonists[c_idx].activity = ColonistActivity::Resting;
                        return;
                    }
                }
            }
        }

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
                let speed_mult = self.colonists[c_idx].role_profile.chop_speed_mult()
                    * self.colonists[c_idx].stats.chop_strength_mult();
                self.colonists[c_idx].activity = ColonistActivity::Chopping;
                self.colonists[c_idx]
                    .inventory
                    .equip_main_hand(CopperTool::CopperAxe);
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

                let wood_yield = self.colonists[c_idx].role_profile.chop_wood_yield();
                if let Some(idx) = self.idx(t.target_x, t.target_y) {
                    if self.props[idx] == 1 || self.props[idx] == 2 {
                        self.props[idx] = 0;
                        self.blocked[idx] = 0;
                        self.spawn_item(
                            ItemKind::WoodLogs,
                            t.target_x,
                            t.target_y,
                            wood_yield,
                            false,
                        );
                        self.pending_sfx.push("chop");
                        let energy = self.colonists[c_idx].energy;
                        let xp_mult = self.colonists[c_idx].stats.xp_gain_mult();
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Lumberjack, 14.0 * xp_mult, energy);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Strength, 4.5);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Endurance, 2.5);
                    }
                }
                self.orders.retain(|o| o.id != t.order_id);
                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.2;
                self.world_dirty = true;
            }
            OrderKind::MineRock => {
                let speed_mult = self.colonists[c_idx].role_profile.mine_speed_mult()
                    * self.colonists[c_idx].stats.mine_strength_mult();
                self.colonists[c_idx].activity = ColonistActivity::Mining;
                self.colonists[c_idx]
                    .inventory
                    .equip_main_hand(CopperTool::CopperPickaxe);
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

                let bonus = self.colonists[c_idx].role_profile.mine_bonus_yield();
                if let Some(idx) = self.idx(t.target_x, t.target_y) {
                    let prop = self.props[idx];
                    let xp_mult = self.colonists[c_idx].stats.xp_gain_mult();
                    let energy = self.colonists[c_idx].energy;
                    if prop == 4 {
                        self.props[idx] = 0;
                        self.blocked[idx] = 0;
                        self.spawn_item(
                            ItemKind::StoneChunk,
                            t.target_x,
                            t.target_y,
                            3 + bonus,
                            false,
                        );
                        self.spawn_item(
                            ItemKind::CopperOre,
                            t.target_x,
                            t.target_y,
                            1 + if bonus >= 2 { 1 } else { 0 },
                            false,
                        );
                        if bonus >= 1 {
                            self.spawn_item(ItemKind::Coal, t.target_x, t.target_y, 1, false);
                        }
                        self.pending_sfx.push("mine");
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Miner, 16.0 * xp_mult, energy);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Strength, 5.0);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Endurance, 3.0);
                    } else if prop == 5 {
                        self.props[idx] = 0;
                        self.blocked[idx] = 0;
                        self.spawn_item(
                            ItemKind::IronOre,
                            t.target_x,
                            t.target_y,
                            3 + bonus,
                            false,
                        );
                        self.spawn_item(
                            ItemKind::StoneChunk,
                            t.target_x,
                            t.target_y,
                            1 + bonus,
                            false,
                        );
                        self.pending_sfx.push("mine");
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Miner, 18.0 * xp_mult, energy);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Strength, 5.0);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Endurance, 3.0);
                    } else if prop == 7 {
                        self.props[idx] = 0;
                        self.blocked[idx] = 0;
                        self.spawn_item(
                            ItemKind::CopperOre,
                            t.target_x,
                            t.target_y,
                            3 + bonus,
                            false,
                        );
                        self.spawn_item(
                            ItemKind::StoneChunk,
                            t.target_x,
                            t.target_y,
                            1 + bonus,
                            false,
                        );
                        self.pending_sfx.push("mine");
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Miner, 18.0 * xp_mult, energy);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Strength, 5.0);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Endurance, 3.0);
                    } else if prop == 8 {
                        self.props[idx] = 0;
                        self.blocked[idx] = 0;
                        self.spawn_item(
                            ItemKind::Coal,
                            t.target_x,
                            t.target_y,
                            3 + bonus,
                            false,
                        );
                        self.spawn_item(
                            ItemKind::StoneChunk,
                            t.target_x,
                            t.target_y,
                            1 + bonus,
                            false,
                        );
                        self.pending_sfx.push("mine");
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Miner, 18.0 * xp_mult, energy);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Strength, 5.0);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Endurance, 3.0);
                    } else if prop == 9 {
                        // Bloque de montaña de piedra natural.
                        self.props[idx] = 0;
                        self.blocked[idx] = 0;
                        self.spawn_item(
                            ItemKind::StoneChunk,
                            t.target_x,
                            t.target_y,
                            4 + bonus,
                            false,
                        );
                        if bonus >= 1 {
                            self.spawn_item(ItemKind::CopperOre, t.target_x, t.target_y, 1, false);
                        }
                        if bonus >= 2 {
                            self.spawn_item(ItemKind::Coal, t.target_x, t.target_y, 1, false);
                        }
                        self.pending_sfx.push("mine");
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Miner, 17.0 * xp_mult, energy);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Strength, 5.0);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Endurance, 3.0);
                    } else if prop == 10 {
                        // Bloque de montaña con veta de cobre.
                        self.props[idx] = 0;
                        self.blocked[idx] = 0;
                        self.spawn_item(
                            ItemKind::CopperOre,
                            t.target_x,
                            t.target_y,
                            3 + bonus,
                            false,
                        );
                        self.spawn_item(
                            ItemKind::StoneChunk,
                            t.target_x,
                            t.target_y,
                            2 + bonus,
                            false,
                        );
                        self.pending_sfx.push("mine");
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Miner, 19.0 * xp_mult, energy);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Strength, 5.0);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Endurance, 3.0);
                    } else if prop == 11 {
                        // Bloque de montaña con veta de carbón.
                        self.props[idx] = 0;
                        self.blocked[idx] = 0;
                        self.spawn_item(
                            ItemKind::Coal,
                            t.target_x,
                            t.target_y,
                            3 + bonus,
                            false,
                        );
                        self.spawn_item(
                            ItemKind::StoneChunk,
                            t.target_x,
                            t.target_y,
                            2 + bonus,
                            false,
                        );
                        self.pending_sfx.push("mine");
                        self.colonists[c_idx]
                            .role_profile
                            .grant_xp(ColonyRole::Miner, 19.0 * xp_mult, energy);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Strength, 5.0);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Endurance, 3.0);
                    }
                }
                self.orders.retain(|o| o.id != t.order_id);
                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.2;
                self.world_dirty = true;
            }
            OrderKind::BuildWoodWall
            | OrderKind::BuildWoodDoor
            | OrderKind::BuildCarpenterBench
            | OrderKind::BuildWaterWell
            | OrderKind::BuildMasonBench
            | OrderKind::BuildStoneFurnace
            | OrderKind::BuildStoneWall
            | OrderKind::BuildStoneFloor
            | OrderKind::BuildStoneBrazier => {
                let speed_mult = self.colonists[c_idx].role_profile.build_speed_mult()
                    * self.colonists[c_idx].stats.build_dex_mult();
                self.colonists[c_idx].activity = ColonistActivity::Building;
                self.colonists[c_idx]
                    .inventory
                    .equip_main_hand(CopperTool::CopperShovel);
                self.colonists[c_idx].work_swing_phase += dt * 10.0 * speed_mult;

                let mut t = task;
                let prev_t = t.work_timer;
                t.work_timer += dt * speed_mult;
                for beat in [0.25f32, 0.90, 1.50] {
                    if prev_t < beat && t.work_timer >= beat {
                        self.pending_sfx.push("build");
                    }
                }
                let required_time = match t.kind {
                    OrderKind::BuildStoneFurnace => 2.7,
                    OrderKind::BuildWaterWell => 2.5,
                    OrderKind::BuildCarpenterBench | OrderKind::BuildMasonBench => 2.4,
                    OrderKind::BuildStoneBrazier => 2.1,
                    OrderKind::BuildWoodDoor | OrderKind::BuildStoneWall => 1.9,
                    OrderKind::BuildStoneFloor => 1.3,
                    _ => 1.8,
                };
                if t.work_timer < required_time {
                    self.colonists[c_idx].active_task = Some(t);
                    return;
                }

                if let Some(idx) = self.idx(t.target_x, t.target_y) {
                    let expected_bp = match t.kind {
                        OrderKind::BuildWoodWall => StructureKind::WoodWallBlueprint as i32,
                        OrderKind::BuildWoodDoor => StructureKind::WoodDoorBlueprint as i32,
                        OrderKind::BuildCarpenterBench => {
                            StructureKind::CarpenterBenchBlueprint as i32
                        }
                        OrderKind::BuildWaterWell => StructureKind::WaterWellBlueprint as i32,
                        OrderKind::BuildMasonBench => StructureKind::MasonBenchBlueprint as i32,
                        OrderKind::BuildStoneFurnace => StructureKind::StoneFurnaceBlueprint as i32,
                        OrderKind::BuildStoneWall => StructureKind::StoneWallBlueprint as i32,
                        OrderKind::BuildStoneFloor => StructureKind::StoneFloorBlueprint as i32,
                        OrderKind::BuildStoneBrazier => StructureKind::StoneBrazierBlueprint as i32,
                        _ => 0,
                    };

                    if self.structures[idx] != expected_bp {
                        self.orders.retain(|o| o.id != t.order_id);
                    } else {
                        let built = match t.kind {
                            OrderKind::BuildWoodWall => {
                                if self.consume_item_from_world(ItemKind::WoodLogs, WOOD_PER_WALL) {
                                    self.structures[idx] = StructureKind::WoodWall as i32;
                                    self.blocked[idx] = 1;
                                    true
                                } else {
                                    false
                                }
                            }
                            OrderKind::BuildWoodDoor => {
                                if self.consume_item_from_world(
                                    ItemKind::WoodDoorItem,
                                    DOOR_ITEMS_PER_DOOR,
                                ) {
                                    self.structures[idx] = StructureKind::WoodDoor as i32;
                                    // La puerta instalada permite el paso de los colonos.
                                    self.blocked[idx] = 0;
                                    true
                                } else {
                                    false
                                }
                            }
                            OrderKind::BuildCarpenterBench => {
                                if self.consume_item_from_world(
                                    ItemKind::WoodLogs,
                                    WOOD_PER_CARPENTER_BENCH,
                                ) {
                                    self.structures[idx] = StructureKind::CarpenterBench as i32;
                                    self.blocked[idx] = 1;
                                    let bench_id = self.next_bench_id;
                                    self.next_bench_id += 1;
                                    self.benches.push(CarpentryBenchState {
                                        id: bench_id,
                                        x: t.target_x,
                                        y: t.target_y,
                                        queue: Vec::new(),
                                    });
                                    true
                                } else {
                                    false
                                }
                            }
                            OrderKind::BuildWaterWell => {
                                if self.available_ground_item(ItemKind::WoodBucket)
                                    >= BUCKETS_PER_WELL
                                    && self.available_ground_item(ItemKind::WoodLogs)
                                        >= WOOD_PER_WELL
                                    && self.available_ground_item(ItemKind::StoneChunk)
                                        >= STONE_PER_WELL
                                {
                                    let _ = self.consume_item_from_world(
                                        ItemKind::WoodBucket,
                                        BUCKETS_PER_WELL,
                                    );
                                    let _ = self.consume_item_from_world(
                                        ItemKind::WoodLogs,
                                        WOOD_PER_WELL,
                                    );
                                    let _ = self.consume_item_from_world(
                                        ItemKind::StoneChunk,
                                        STONE_PER_WELL,
                                    );
                                    self.structures[idx] = StructureKind::WaterWell as i32;
                                    self.blocked[idx] = 1;
                                    true
                                } else {
                                    false
                                }
                            }
                            OrderKind::BuildMasonBench => {
                                if self.consume_item_from_world(
                                    ItemKind::StoneChunk,
                                    STONE_PER_MASON_BENCH,
                                ) {
                                    self.structures[idx] = StructureKind::MasonBench as i32;
                                    self.blocked[idx] = 1;
                                    let ws_id = self.next_workshop_id;
                                    self.next_workshop_id += 1;
                                    self.workshops.push(WorkshopStationState {
                                        id: ws_id,
                                        kind: WorkshopKind::MasonBench,
                                        x: t.target_x,
                                        y: t.target_y,
                                        queue: Vec::new(),
                                    });
                                    true
                                } else {
                                    false
                                }
                            }
                            OrderKind::BuildStoneFurnace => {
                                if self.available_ground_item(ItemKind::StoneChunk)
                                    >= STONE_PER_FURNACE
                                    && self.available_ground_item(ItemKind::StoneBricks)
                                        >= BRICKS_PER_FURNACE
                                {
                                    let _ = self.consume_item_from_world(
                                        ItemKind::StoneChunk,
                                        STONE_PER_FURNACE,
                                    );
                                    let _ = self.consume_item_from_world(
                                        ItemKind::StoneBricks,
                                        BRICKS_PER_FURNACE,
                                    );
                                    self.structures[idx] = StructureKind::StoneFurnace as i32;
                                    self.blocked[idx] = 1;
                                    let ws_id = self.next_workshop_id;
                                    self.next_workshop_id += 1;
                                    self.workshops.push(WorkshopStationState {
                                        id: ws_id,
                                        kind: WorkshopKind::StoneFurnace,
                                        x: t.target_x,
                                        y: t.target_y,
                                        queue: Vec::new(),
                                    });
                                    true
                                } else {
                                    false
                                }
                            }
                            OrderKind::BuildStoneWall => {
                                if self.available_ground_item(ItemKind::StoneChunk)
                                    >= STONE_PER_STONE_WALL
                                    && self.available_ground_item(ItemKind::StoneBricks)
                                        >= BRICKS_PER_STONE_WALL
                                {
                                    let _ = self.consume_item_from_world(
                                        ItemKind::StoneChunk,
                                        STONE_PER_STONE_WALL,
                                    );
                                    let _ = self.consume_item_from_world(
                                        ItemKind::StoneBricks,
                                        BRICKS_PER_STONE_WALL,
                                    );
                                    self.structures[idx] = StructureKind::StoneWall as i32;
                                    self.blocked[idx] = 1;
                                    true
                                } else {
                                    false
                                }
                            }
                            OrderKind::BuildStoneFloor => {
                                if self.consume_item_from_world(
                                    ItemKind::StoneBricks,
                                    BRICKS_PER_STONE_FLOOR,
                                ) {
                                    self.structures[idx] = StructureKind::StoneFloor as i32;
                                    self.blocked[idx] = 0;
                                    true
                                } else {
                                    false
                                }
                            }
                            OrderKind::BuildStoneBrazier => {
                                if self.available_ground_item(ItemKind::StoneChunk)
                                    >= STONE_PER_BRAZIER
                                    && self.available_ground_item(ItemKind::StoneBricks)
                                        >= BRICKS_PER_BRAZIER
                                    && self.available_ground_item(ItemKind::Coal)
                                        >= COAL_PER_BRAZIER
                                {
                                    let _ = self.consume_item_from_world(
                                        ItemKind::StoneChunk,
                                        STONE_PER_BRAZIER,
                                    );
                                    let _ = self.consume_item_from_world(
                                        ItemKind::StoneBricks,
                                        BRICKS_PER_BRAZIER,
                                    );
                                    let _ = self.consume_item_from_world(
                                        ItemKind::Coal,
                                        COAL_PER_BRAZIER,
                                    );
                                    self.structures[idx] = StructureKind::StoneBrazier as i32;
                                    self.blocked[idx] = 1;
                                    true
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        };

                        if built {
                            self.orders.retain(|o| o.id != t.order_id);
                            self.pending_sfx.push("build");
                            let energy = self.colonists[c_idx].energy;
                            let xp_mult = self.colonists[c_idx].stats.xp_gain_mult();
                            let base_xp = match t.kind {
                                OrderKind::BuildStoneFurnace => 28.0,
                                OrderKind::BuildCarpenterBench | OrderKind::BuildMasonBench => 24.0,
                                OrderKind::BuildWaterWell | OrderKind::BuildStoneBrazier => 22.0,
                                OrderKind::BuildWoodDoor | OrderKind::BuildStoneWall => 18.0,
                                OrderKind::BuildStoneFloor => 12.0,
                                _ => 15.0,
                            };
                            self.colonists[c_idx]
                                .role_profile
                                .grant_xp(ColonyRole::Constructor, base_xp * xp_mult, energy);
                            self.colonists[c_idx]
                                .stats
                                .grant_stat_xp(StatKind::Dexterity, 3.5);
                            self.colonists[c_idx]
                                .stats
                                .grant_stat_xp(StatKind::Strength, 2.0);
                        } else {
                            for o in &mut self.orders {
                                if o.id == t.order_id {
                                    o.assigned_colonist_id = 0;
                                }
                            }
                        }
                    }
                }
                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.25;
                self.world_dirty = true;
            }
            OrderKind::CarpentryJob => {
                let q_idx = task.phase.max(0) as usize;
                let recipe_opt = self
                    .benches
                    .iter()
                    .find(|b| b.id == task.order_id)
                    .and_then(|b| b.queue.get(q_idx).copied());

                let Some(recipe) = recipe_opt else {
                    self.colonists[c_idx].active_task = None;
                    self.colonists[c_idx].activity = ColonistActivity::Resting;
                    return;
                };

                if (recipe as i32) != task.item_id
                    || !self.colonists[c_idx]
                        .stats
                        .can_craft_complexity(recipe.min_intelligence())
                {
                    self.colonists[c_idx].active_task = None;
                    self.colonists[c_idx].activity = ColonistActivity::Resting;
                    return;
                }

                let speed_mult = self.colonists[c_idx].role_profile.carpentry_speed_mult()
                    * self.colonists[c_idx].stats.craft_dex_mult();
                self.colonists[c_idx].activity = ColonistActivity::Carpentry;
                self.colonists[c_idx]
                    .inventory
                    .equip_main_hand(CopperTool::CopperAxe);
                self.colonists[c_idx].work_swing_phase += dt * 10.5 * speed_mult;

                let mut t = task;
                let prev_t = t.work_timer;
                t.work_timer += dt * speed_mult;
                for beat in [0.35f32, 1.15, 2.05] {
                    if prev_t < beat && t.work_timer >= beat {
                        self.pending_sfx.push("build");
                    }
                }
                if t.work_timer < recipe.work_duration() {
                    self.colonists[c_idx].active_task = Some(t);
                    return;
                }

                let (in_kind, in_amt) = recipe.input_cost();
                if self.consume_item_from_world(in_kind, in_amt) {
                    let (out_kind, out_amt) = recipe.output_item();
                    let (drop_x, drop_y) = self.find_drop_tile_near(t.target_x, t.target_y);
                    self.spawn_item(out_kind, drop_x, drop_y, out_amt, false);
                    if let Some(bench) = self.benches.iter_mut().find(|b| b.id == t.order_id) {
                        if q_idx < bench.queue.len() && bench.queue[q_idx] == recipe {
                            bench.queue.remove(q_idx);
                        }
                    }
                    self.pending_sfx.push("build");
                    let energy = self.colonists[c_idx].energy;
                    let xp_mult = self.colonists[c_idx].stats.xp_gain_mult();
                    let role_xp = (16.0 + recipe.min_intelligence() as f32 * 4.0) * xp_mult;
                    self.colonists[c_idx]
                        .role_profile
                        .grant_xp(ColonyRole::Carpenter, role_xp, energy);
                    self.colonists[c_idx].stats.grant_stat_xp(
                        StatKind::Intelligence,
                        4.5 + recipe.min_intelligence() as f32 * 1.5,
                    );
                    self.colonists[c_idx]
                        .stats
                        .grant_stat_xp(StatKind::Dexterity, 4.0);
                }

                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.25;
                self.world_dirty = true;
            }
            OrderKind::MasonryJob => {
                let q_idx = task.phase.max(0) as usize;
                let ws_info = self
                    .workshops
                    .iter()
                    .find(|w| w.id == task.order_id)
                    .and_then(|w| w.queue.get(q_idx).copied().map(|r| (w.kind, r)));

                let Some((ws_kind, recipe)) = ws_info else {
                    self.colonists[c_idx].active_task = None;
                    self.colonists[c_idx].activity = ColonistActivity::Resting;
                    return;
                };

                if (recipe as i32) != task.item_id
                    || !self.colonists[c_idx]
                        .stats
                        .can_craft_complexity(recipe.min_intelligence())
                {
                    self.colonists[c_idx].active_task = None;
                    self.colonists[c_idx].activity = ColonistActivity::Resting;
                    return;
                }

                let speed_mult = self.colonists[c_idx].role_profile.masonry_speed_mult()
                    * self.colonists[c_idx].stats.craft_dex_mult();
                self.colonists[c_idx].activity = if ws_kind == WorkshopKind::StoneFurnace {
                    ColonistActivity::Smelting
                } else {
                    ColonistActivity::Masonry
                };
                self.colonists[c_idx]
                    .inventory
                    .equip_main_hand(CopperTool::CopperPickaxe);
                self.colonists[c_idx].work_swing_phase += dt * 10.5 * speed_mult;

                let mut t = task;
                let prev_t = t.work_timer;
                t.work_timer += dt * speed_mult;
                for beat in [0.35f32, 1.15, 2.05] {
                    if prev_t < beat && t.work_timer >= beat {
                        self.pending_sfx.push(if ws_kind == WorkshopKind::StoneFurnace {
                            "build"
                        } else {
                            "mine"
                        });
                    }
                }
                if t.work_timer < recipe.work_duration() {
                    self.colonists[c_idx].active_task = Some(t);
                    return;
                }

                let (in1, amt1) = recipe.input_cost_1();
                let in2_opt = recipe.input_cost_2();
                let has_all = self.available_ground_item(in1) >= amt1
                    && in2_opt.is_none_or(|(in2, amt2)| self.available_ground_item(in2) >= amt2);
                if has_all {
                    let _ = self.consume_item_from_world(in1, amt1);
                    if let Some((in2, amt2)) = in2_opt {
                        let _ = self.consume_item_from_world(in2, amt2);
                    }
                    let (out_kind, out_amt) = recipe.output_item();
                    let (drop_x, drop_y) = self.find_drop_tile_near(t.target_x, t.target_y);
                    self.spawn_item(out_kind, drop_x, drop_y, out_amt, false);
                    if let Some(ws) = self.workshops.iter_mut().find(|w| w.id == t.order_id) {
                        if q_idx < ws.queue.len() && ws.queue[q_idx] == recipe {
                            ws.queue.remove(q_idx);
                        }
                    }
                    self.pending_sfx.push("build");
                    let energy = self.colonists[c_idx].energy;
                    let xp_mult = self.colonists[c_idx].stats.xp_gain_mult();
                    let role_xp = (16.0 + recipe.min_intelligence() as f32 * 4.0) * xp_mult;
                    self.colonists[c_idx]
                        .role_profile
                        .grant_xp(ColonyRole::Mason, role_xp, energy);
                    self.colonists[c_idx].stats.grant_stat_xp(
                        StatKind::Intelligence,
                        4.5 + recipe.min_intelligence() as f32 * 1.5,
                    );
                    self.colonists[c_idx]
                        .stats
                        .grant_stat_xp(StatKind::Dexterity, 3.5);
                }

                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.25;
                self.world_dirty = true;
            }
            OrderKind::FarmCell => {
                self.execute_farm_cell_task(c_idx, task, dt);
            }
            OrderKind::WaterCrops => {
                self.colonists[c_idx].activity = ColonistActivity::WateringCrops;
                self.colonists[c_idx].inventory.stow_main_hand();
                let mut t = task;
                if t.phase == 0 {
                    t.work_timer += dt;
                    if t.work_timer < 0.75 {
                        self.colonists[c_idx].active_task = Some(t);
                        return;
                    }
                    if self.available_ground_item(ItemKind::WoodBucket) < 1 {
                        self.colonists[c_idx].active_task = None;
                        self.colonists[c_idx].activity = ColonistActivity::Resting;
                        return;
                    }
                    let packed_x = t.item_id % self.map_width;
                    let packed_y = t.item_id / self.map_width;
                    let target_crop = if self.crops.iter().any(|c| c.x == packed_x && c.y == packed_y)
                    {
                        Some((packed_x, packed_y))
                    } else {
                        let cur_x = self.colonists[c_idx].x.floor() as i32;
                        let cur_y = self.colonists[c_idx].y.floor() as i32;
                        self.pick_waterable_crop(cur_x, cur_y)
                    };
                    if let Some((crop_x, crop_y)) = target_crop {
                        let cur_x = self.colonists[c_idx].x.floor() as i32;
                        let cur_y = self.colonists[c_idx].y.floor() as i32;
                        if let Some(path) = find_path(
                            self.map_width,
                            self.map_height,
                            &self.blocked,
                            cur_x,
                            cur_y,
                            crop_x,
                            crop_y,
                            false,
                        ) {
                            t.phase = 1;
                            t.target_x = crop_x;
                            t.target_y = crop_y;
                            t.work_timer = 0.0;
                            self.colonists[c_idx].path = path;
                            self.colonists[c_idx].active_task = Some(t);
                            return;
                        }
                    }
                    self.colonists[c_idx].active_task = None;
                    self.colonists[c_idx].activity = ColonistActivity::Resting;
                    return;
                }

                // Fase 1: verter el agua del cubo sobre la parcela y vecinas inmediatas (3x3).
                let speed_mult = self.colonists[c_idx].role_profile.botany_speed_mult()
                    * self.colonists[c_idx].stats.botany_intel_mult();
                self.colonists[c_idx].work_swing_phase += dt * 8.5 * speed_mult;
                t.work_timer += dt * speed_mult;
                if t.work_timer < 0.85 {
                    self.colonists[c_idx].active_task = Some(t);
                    return;
                }

                let mut watered_any = false;
                for crop in &mut self.crops {
                    if (crop.x - t.target_x).abs() <= 1 && (crop.y - t.target_y).abs() <= 1 {
                        crop.watered_timer = 65.0;
                        watered_any = true;
                    }
                }
                if watered_any {
                    let energy = self.colonists[c_idx].energy;
                    let xp_mult = self.colonists[c_idx].stats.xp_gain_mult();
                    self.colonists[c_idx]
                        .role_profile
                        .grant_xp(ColonyRole::Botanist, 12.0 * xp_mult, energy);
                    self.colonists[c_idx]
                        .stats
                        .grant_stat_xp(StatKind::Intelligence, 2.0);
                    self.world_dirty = true;
                }
                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.2;
            }
            OrderKind::HuntAnimal => {
                let speed_mult = self.colonists[c_idx].role_profile.scout_speed_mult()
                    * self.colonists[c_idx].stats.chop_strength_mult();
                self.colonists[c_idx].activity = ColonistActivity::Hunting;
                self.colonists[c_idx]
                    .inventory
                    .equip_main_hand(CopperTool::CopperSword);
                self.colonists[c_idx].work_swing_phase += dt * 11.5 * speed_mult;

                let mut t = task;
                let prev_t = t.work_timer;
                t.work_timer += dt * speed_mult;
                for beat in [0.25f32, 0.90] {
                    if prev_t < beat && t.work_timer >= beat {
                        self.pending_sfx.push("chop");
                    }
                }
                if t.work_timer < 1.5 {
                    self.colonists[c_idx].active_task = Some(t);
                    return;
                }

                if let Some(pos) = self.animals.iter().position(|a| a.id == t.item_id) {
                    let animal = self.animals.remove(pos);
                    let drop_tx = (animal.x.floor() as i32).clamp(1, self.map_width - 2);
                    let drop_ty = (animal.y.floor() as i32).clamp(1, self.map_height - 2);
                    for &(drop_kind, drop_amt) in animal.kind.butcher_drops() {
                        self.spawn_item(drop_kind, drop_tx, drop_ty, drop_amt, false);
                    }
                    self.pending_sfx.push("chop");
                    let energy = self.colonists[c_idx].energy;
                    let xp_mult = self.colonists[c_idx].stats.xp_gain_mult();
                    self.colonists[c_idx]
                        .role_profile
                        .grant_xp(ColonyRole::Scout, 18.0 * xp_mult, energy);
                    self.colonists[c_idx]
                        .stats
                        .grant_stat_xp(StatKind::Strength, 4.0);
                    self.colonists[c_idx]
                        .stats
                        .grant_stat_xp(StatKind::Dexterity, 3.0);
                }

                self.orders.retain(|o| o.id != t.order_id);
                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.25;
                self.world_dirty = true;
            }
            OrderKind::HaulItem => {
                self.colonists[c_idx].activity = ColonistActivity::Hauling;
                self.colonists[c_idx].inventory.stow_main_hand();
                if task.phase == 0 {
                    let carry_cap = self.colonists[c_idx].role_profile.haul_capacity()
                        + self.colonists[c_idx].stats.carry_bonus();
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
                    let kind = self.colonists[c_idx].carrying_kind.take();
                    let amt = self.colonists[c_idx].carrying_amount;
                    self.colonists[c_idx].carrying_amount = 0;
                    if let Some(k) = kind {
                        self.spawn_item(k, task.target_x, task.target_y, amt, true);
                        let energy = self.colonists[c_idx].energy;
                        let xp_mult = self.colonists[c_idx].stats.xp_gain_mult();
                        self.colonists[c_idx].role_profile.grant_xp(
                            ColonyRole::Hauler,
                            (8.0 + (amt as f32) * 1.2) * xp_mult,
                            energy,
                        );
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Strength, 2.5);
                        self.colonists[c_idx]
                            .stats
                            .grant_stat_xp(StatKind::Endurance, 2.0);
                    }
                    self.colonists[c_idx].active_task = None;
                    self.colonists[c_idx].activity = ColonistActivity::Resting;
                    self.colonists[c_idx].wait_timer = 0.15;
                }
            }
            OrderKind::DrinkWater => {
                let from_well = task.phase == 1;
                self.colonists[c_idx].inventory.stow_main_hand();
                self.colonists[c_idx].activity = if from_well {
                    ColonistActivity::DrinkingWell
                } else {
                    ColonistActivity::DrinkingRiver
                };

                let mut t = task;
                t.work_timer += dt;
                if t.work_timer < 1.6 {
                    self.colonists[c_idx].active_task = Some(t);
                    return;
                }

                let target_valid = if from_well {
                    self.idx(t.target_x, t.target_y)
                        .is_some_and(|i| self.structures[i] == StructureKind::WaterWell as i32)
                } else {
                    self.idx(t.target_x, t.target_y)
                        .is_some_and(|i| self.ground[i] == GroundKind::River as i32)
                };

                if target_valid {
                    // Tener vasos de madera mejora la experiencia al beber; los cubos NO se aceptan para agua cruda del río.
                    let has_cup = self.available_ground_item(ItemKind::WoodCup) > 0;
                    if from_well {
                        self.colonists[c_idx].hydration = 100.0;
                        let boost = if has_cup { 4 } else { 2 };
                        self.colonists[c_idx].mood =
                            (self.colonists[c_idx].mood + boost).clamp(0, 100);
                    } else {
                        self.colonists[c_idx].hydration = if has_cup { 96.0 } else { 88.0 };
                        let penalty = if has_cup { 4 } else { 9 };
                        self.colonists[c_idx].mood =
                            (self.colonists[c_idx].mood - penalty).clamp(15, 100);
                    }
                    self.colonists[c_idx]
                        .stats
                        .grant_stat_xp(StatKind::Endurance, 1.5);
                }

                self.colonists[c_idx].active_task = None;
                self.colonists[c_idx].activity = ColonistActivity::Resting;
                self.colonists[c_idx].wait_timer = 0.2;
            }
        }
    }
}
