//! Lógica agrícola y botánica para múltiples cultivos y temporadas de siembra.
//!
//! # Lógica del módulo
//! Separa las mecánicas del ciclo agrícola:
//! - Avance temporal de maduración sobre tierra normal y tierra fértil (`step_crops_growth`),
//!   acelerado durante la **Temporada de Cultivos**.
//! - Selección de parcelas pendientes para el rol **Agricultor** (arar y cosechar) y
//!   para el rol **Botánico** (sembrar semillas de zanahoria, papa, trigo o remolacha y cuidar crecimiento).
//! - Ejecución de trabajo agrícola en casilla (`execute_farm_cell_task`).

use super::core::SimulationCore;
use crate::colony_types::{
    ActiveTask, ColonistActivity, ColonyInventory, CropKind, CropStage, OrderKind, StructureKind,
};
use crate::pathfinding::find_path;
use crate::roles::ColonyRole;
use crate::tools::CopperTool;
use crate::world_gen::GroundKind;

impl SimulationCore {
    pub(super) fn step_crops_growth(&mut self, dt: f32) {
        let mut crop_stage_changed = false;
        let in_season = self.is_crop_season_active();
        let season_mult = if in_season { 1.65 } else { 1.0 };

        for crop in &mut self.crops {
            if crop.tend_cooldown > 0.0 {
                crop.tend_cooldown = (crop.tend_cooldown - dt).max(0.0);
            }
            if crop.watered_timer > 0.0 {
                let was_watered = crop.watered_timer > 0.0;
                crop.watered_timer = (crop.watered_timer - dt).max(0.0);
                if was_watered && crop.watered_timer <= 0.0 {
                    crop_stage_changed = true;
                }
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
                // Regar con cubo compensa con fuerza estar fuera de temporada (1.60x)
                // y potencia aún más el crecimiento cuando sí es Temporada de Cultivos (1.45x extra).
                let water_mult = if crop.watered_timer > 0.0 {
                    if in_season {
                        1.45
                    } else {
                        1.60
                    }
                } else {
                    1.0
                };
                crop.progress =
                    (crop.progress + fertile_bonus * season_mult * water_mult * dt).min(1.0);
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
    }

    pub(super) fn pick_waterable_crop(&self, cx: i32, cy: i32) -> Option<(i32, i32)> {
        let mut best_crop: Option<(i32, i32)> = None;
        let mut best_score = i32::MAX;
        for crop in &self.crops {
            let needs_water = matches!(
                crop.stage,
                CropStage::Tilled | CropStage::Seeded | CropStage::Sprout | CropStage::Growing
            ) && crop.watered_timer <= 8.0;
            if !needs_water {
                continue;
            }
            let crop_packed = crop.y * self.map_width + crop.x;
            let already_taken = self.colonists.iter().any(|col| {
                col.active_task.as_ref().is_some_and(|t| {
                    t.kind == OrderKind::WaterCrops
                        && ((t.phase == 0 && t.item_id == crop_packed)
                            || (t.phase == 1 && (t.target_x - crop.x).abs() <= 1 && (t.target_y - crop.y).abs() <= 1))
                })
            });
            if already_taken {
                continue;
            }
            let priority_bonus = if matches!(
                crop.stage,
                CropStage::Seeded | CropStage::Sprout | CropStage::Growing
            ) {
                -45
            } else {
                0
            };
            let score = (crop.x - cx).abs() + (crop.y - cy).abs() + priority_bonus;
            if score < best_score {
                best_score = score;
                best_crop = Some((crop.x, crop.y));
            }
        }
        best_crop
    }

    pub(super) fn find_water_source_for_irrigation(
        &self,
        cx: i32,
        cy: i32,
    ) -> Option<(i32, i32, Vec<(i32, i32)>)> {
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
                return Some((wx, wy, path));
            }
        }

        let mut river_tiles: Vec<(i32, i32, i32)> = Vec::new();
        for y in 1..self.map_height - 1 {
            for x in 1..self.map_width - 1 {
                let idx = (y * self.map_width + x) as usize;
                if self.ground[idx] == GroundKind::River as i32 {
                    let has_bank = [(0, 1), (1, 0), (-1, 0), (0, -1)].iter().any(|&(ox, oy)| {
                        Self::tile_blocked(
                            self.map_width,
                            self.map_height,
                            &self.blocked,
                            x + ox,
                            y + oy,
                        ) == false
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
                return Some((rx, ry, path));
            }
        }
        None
    }

    pub(super) fn pick_farmer_crop(&self, cx: i32, cy: i32) -> Option<(i32, i32)> {
        let mut best_crop: Option<(i32, i32)> = None;
        let mut best_crop_score = i32::MAX;
        for crop in &self.crops {
            let already_taken = self.colonists.iter().any(|col| {
                col.active_task.as_ref().is_some_and(|t| {
                    t.kind == OrderKind::FarmCell && t.target_x == crop.x && t.target_y == crop.y
                })
            });
            if already_taken {
                continue;
            }
            let needs_action = matches!(crop.stage, CropStage::None | CropStage::Ready);
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
        best_crop
    }

    fn seeds_for_crop(inv: &ColonyInventory, kind: CropKind) -> i32 {
        match kind {
            CropKind::Carrot => inv.carrot_seeds,
            CropKind::Potato => inv.potato_seeds,
            CropKind::Wheat => inv.wheat_seeds,
            CropKind::Beet => inv.beet_seeds,
        }
    }

    pub(super) fn pick_botanist_crop(
        &self,
        cx: i32,
        cy: i32,
        inv: &ColonyInventory,
    ) -> Option<(i32, i32)> {
        let mut best_crop: Option<(i32, i32)> = None;
        let mut best_crop_score = i32::MAX;
        for crop in &self.crops {
            let already_taken = self.colonists.iter().any(|col| {
                col.active_task.as_ref().is_some_and(|t| {
                    t.kind == OrderKind::FarmCell && t.target_x == crop.x && t.target_y == crop.y
                })
            });
            if already_taken {
                continue;
            }
            let can_seed =
                crop.stage == CropStage::Tilled && Self::seeds_for_crop(inv, crop.crop_kind) > 0;
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
        best_crop
    }

    pub(super) fn execute_farm_cell_task(&mut self, c_idx: usize, task: ActiveTask, dt: f32) {
        let is_botanist_task = task.phase == 1;
        let season_work_mult = if self.is_crop_season_active() {
            1.15
        } else {
            0.72
        };
        let speed_mult = if is_botanist_task {
            self.colonists[c_idx].role_profile.botany_speed_mult()
                * self.colonists[c_idx].stats.botany_intel_mult()
                * season_work_mult
        } else {
            self.colonists[c_idx].role_profile.farm_speed_mult()
                * self.colonists[c_idx].stats.farm_dex_mult()
                * season_work_mult
        };
        self.colonists[c_idx].activity = if is_botanist_task {
            ColonistActivity::Botanizing
        } else {
            ColonistActivity::Farming
        };
        self.colonists[c_idx]
            .inventory
            .equip_main_hand(CopperTool::CopperHoe);
        self.colonists[c_idx].work_swing_phase += dt * 9.5 * speed_mult;

        let mut t = task;
        t.work_timer += dt * speed_mult;
        if t.work_timer < 1.15 {
            self.colonists[c_idx].active_task = Some(t);
            return;
        }

        let mut harvested_crop_kind: Option<CropKind> = None;
        let mut need_seed_crop_kind: Option<CropKind> = None;
        let mut tilled_xp = false;
        let mut tended_xp = false;
        let botany_boost = self.colonists[c_idx].role_profile.botany_growth_boost()
            * self.colonists[c_idx].stats.botany_intel_mult();

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
                    need_seed_crop_kind = Some(crop.crop_kind);
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
                    harvested_crop_kind = Some(crop.crop_kind);
                    crop.stage = CropStage::Tilled;
                    crop.progress = 0.0;
                    crop.tend_cooldown = 0.0;
                }
                _ => {}
            }
        }

        let energy = self.colonists[c_idx].energy;
        let xp_mult = self.colonists[c_idx].stats.xp_gain_mult();
        if tilled_xp {
            self.colonists[c_idx]
                .role_profile
                .grant_xp(ColonyRole::Farmer, 10.0 * xp_mult, energy);
            self.colonists[c_idx]
                .stats
                .grant_stat_xp(crate::stats::StatKind::Dexterity, 2.5);
            self.colonists[c_idx]
                .stats
                .grant_stat_xp(crate::stats::StatKind::Endurance, 2.0);
        }
        if tended_xp {
            self.colonists[c_idx]
                .role_profile
                .grant_xp(ColonyRole::Botanist, 13.0 * xp_mult, energy);
            self.colonists[c_idx]
                .stats
                .grant_stat_xp(crate::stats::StatKind::Intelligence, 3.0);
        }

        if let Some(crop_kind) = need_seed_crop_kind {
            if self.consume_item_from_world(crop_kind.seed_item(), 1) {
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
                    .grant_xp(ColonyRole::Botanist, 11.0 * xp_mult, energy);
                self.colonists[c_idx]
                    .stats
                    .grant_stat_xp(crate::stats::StatKind::Intelligence, 2.0);
            }
        }

        if let Some(crop_kind) = harvested_crop_kind {
            let (produce_yield, seeds_yield) =
                self.colonists[c_idx].role_profile.farm_harvest_yield();
            self.spawn_item(
                crop_kind.harvest_item(),
                t.target_x,
                t.target_y,
                produce_yield,
                false,
            );
            self.spawn_item(
                crop_kind.seed_item(),
                t.target_x,
                t.target_y,
                seeds_yield,
                false,
            );
            self.colonists[c_idx]
                .role_profile
                .grant_xp(ColonyRole::Farmer, 16.0 * xp_mult, energy);
            self.colonists[c_idx]
                .stats
                .grant_stat_xp(crate::stats::StatKind::Dexterity, 3.0);
        }

        self.colonists[c_idx].active_task = None;
        self.colonists[c_idx].activity = ColonistActivity::Resting;
        self.colonists[c_idx].wait_timer = 0.18;
        self.world_dirty = true;
    }
}
