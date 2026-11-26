//! Asignación, movimiento A* y ejecución de tareas de los colonos.
//!
//! # Lógica del módulo
//! Controla el ciclo operativo de trabajo de cada colono:
//! - Liberación de tareas activas (`release_colonist_task`).
//! - Selección de tareas aprobadas según los roles asignados y requisitos de Inteligencia (`assign_task_to_colonist`).
//! - Desplazamiento paso a paso por rutas A* (`step_colonist_movement`).
//! - Ejecución de tala, minería, construcción (muros, puertas, mesas), carpintería, cultivo y acarreo (`execute_active_task`).

use super::core::{SimulationCore, DOOR_ITEMS_PER_DOOR, WOOD_PER_CARPENTER_BENCH, WOOD_PER_WALL};
use crate::colony_types::{
    ActiveTask, CarpentryBenchState, CarpentryRecipe, ColonistActivity, ItemKind, OrderKind,
    OrderStatus, StructureKind,
};
use crate::pathfinding::find_path;
use crate::roles::ColonyRole;
use crate::stats::StatKind;
use crate::tools::CopperTool;

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

        // 2. Evaluar los roles asignados al colono (máximo 5) en orden de prioridad.
        let assigned_roles = self.colonists[c_idx].role_profile.assigned.clone();
        if assigned_roles.is_empty() {
            return;
        }
        let (total_inv, _) = self.compute_inventory();

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
                ColonyRole::Farmer => {
                    if !self.farm_approved {
                        continue;
                    }
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
                ColonyRole::Botanist => {
                    if !self.farm_approved {
                        continue;
                    }
                    if let Some((fx, fy)) = self.pick_botanist_crop(cx, cy, total_inv.carrot_seeds) {
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
                ColonyRole::Scout | ColonyRole::ColonyLeader => {}
            }
        }
    }

    pub(super) fn step_colonist_movement(&mut self, c_idx: usize, dt: f32) -> bool {
        let c = &mut self.colonists[c_idx];
        let role_speed_mult = match c.activity {
            ColonistActivity::Hauling => c.role_profile.haul_speed_mult(),
            ColonistActivity::ConveningMeeting => c.role_profile.leader_meeting_speed_mult(),
            ColonistActivity::Scouting => c.role_profile.scout_speed_mult(),
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
            let effective_speed = c.speed * role_speed_mult * dex_mult;
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
        let Some(task) = self.colonists[c_idx].active_task.clone() else {
            return;
        };

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
            | OrderKind::BuildCarpenterBench => {
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
                    OrderKind::BuildCarpenterBench => 2.4,
                    OrderKind::BuildWoodDoor => 1.9,
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
                            _ => false,
                        };

                        if built {
                            self.orders.retain(|o| o.id != t.order_id);
                            self.pending_sfx.push("build");
                            let energy = self.colonists[c_idx].energy;
                            let xp_mult = self.colonists[c_idx].stats.xp_gain_mult();
                            let base_xp = match t.kind {
                                OrderKind::BuildCarpenterBench => 24.0,
                                OrderKind::BuildWoodDoor => 18.0,
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
            OrderKind::FarmCell => {
                self.execute_farm_cell_task(c_idx, task, dt);
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
        }
    }
}
