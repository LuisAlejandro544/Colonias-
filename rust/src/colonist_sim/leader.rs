//! Lógica del Líder de Colonia, reuniones de órdenes y asignación de roles.
//!
//! # Lógica del módulo
//! Coordina la cadena de mando de **Colonia Centinela**:
//! - Cuando el jugador designa órdenes o zonas, el Líder de Colonia se acerca o
//!   reúne a los colonos antes de autorizar su ejecución (`update_leader_briefing`).
//! - Aprueba las órdenes pendientes otorgando experiencia al líder (`approve_all_pending`).
//! - Gestiona el alta/baja de hasta 5 roles por colono y la transferencia segura
//!   del liderazgo (`set_colonist_role_enabled`).

use super::core::SimulationCore;
use crate::colony_types::{ColonistActivity, OrderKind, OrderStatus};
use crate::pathfinding::find_path;
use crate::roles::{ColonyRole, MAX_ASSIGNED_ROLES};
use crate::tools::CopperTool;

impl SimulationCore {
    pub(super) fn leader_index(&self) -> Option<usize> {
        self.colonists.iter().position(|c| c.is_leader)
    }

    /// Lógica del Líder de Colonia: cuando hay órdenes o zonas pendientes de comunicar,
    /// el líder reúne o se acerca personalmente a los colonos y autoriza las tareas tras
    /// una breve reunión.
    pub(super) fn update_leader_briefing(&mut self, dt: f32) {
        let has_pending_orders = self
            .orders
            .iter()
            .any(|o| o.status == OrderStatus::PendingBriefing);
        let needs_briefing = self.briefing_pending
            || has_pending_orders
            || !self.storage_approved
            || !self.farm_approved;

        if !needs_briefing {
            self.meeting_active = false;
            self.meeting_timer = 0.0;
            self.meeting_approach_timer = 0.0;
            self.meeting_target_id = -1;
            return;
        }

        let Some(l_idx) = self.leader_index() else {
            self.approve_all_pending();
            return;
        };

        let leader_speed_mult = self.colonists[l_idx]
            .role_profile
            .leader_meeting_speed_mult()
            * self.colonists[l_idx].stats.meeting_charisma_mult();

        // Caso colonia de 1 solo colono: el líder planifica brevemente en el sitio.
        if self.colonists.len() <= 1 {
            self.meeting_active = true;
            self.colonists[l_idx].activity = ColonistActivity::ConveningMeeting;
            self.colonists[l_idx]
                .inventory
                .equip_main_hand(CopperTool::CopperSword);
            self.colonists[l_idx].path.clear();
            self.meeting_timer += dt * leader_speed_mult;
            if self.meeting_timer >= 0.7 {
                self.approve_all_pending();
            }
            return;
        }

        self.meeting_active = true;
        self.meeting_approach_timer += dt;

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
            self.colonists[l_idx]
                .inventory
                .equip_main_hand(CopperTool::CopperSword);
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
            self.colonists[l_idx]
                .inventory
                .equip_main_hand(CopperTool::CopperSword);
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

    pub fn approve_all_pending(&mut self) {
        let mut approved_count = 0usize;
        for o in &mut self.orders {
            if o.status == OrderStatus::PendingBriefing {
                approved_count += 1;
            }
            o.status = OrderStatus::Approved;
        }
        let mut mood_boost = 0i32;
        if let Some(l_idx) = self.leader_index() {
            let xp = 7.0 + (approved_count.min(10) as f32) * 2.5;
            let energy = self.colonists[l_idx].energy;
            let xp_mult = self.colonists[l_idx].stats.xp_gain_mult();
            mood_boost = self.colonists[l_idx].stats.meeting_mood_boost();
            self.colonists[l_idx]
                .role_profile
                .grant_xp(ColonyRole::ColonyLeader, xp * xp_mult, energy);
            self.colonists[l_idx]
                .stats
                .grant_stat_xp(crate::stats::StatKind::Charisma, 4.0);
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
                if mood_boost > 0 {
                    c.mood = (c.mood + mood_boost).min(100);
                }
                c.activity = ColonistActivity::Resting;
                c.wait_timer = 0.15;
                c.path.clear();
            }
        }
        self.world_dirty = true;
    }

    /// Asigna o quita uno de los 9 roles a un colono respetando el límite de 5 roles
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
                let other_leader = self.colonists.iter().enumerate().any(|(i, c)| {
                    i != c_idx && (c.is_leader || c.role_profile.has_role(ColonyRole::ColonyLeader))
                });
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
                    OrderKind::MineRock => !self.colonists[c_idx]
                        .role_profile
                        .has_role(ColonyRole::Miner),
                    OrderKind::BuildWoodWall
                    | OrderKind::BuildWoodDoor
                    | OrderKind::BuildCarpenterBench => !self.colonists[c_idx]
                        .role_profile
                        .has_role(ColonyRole::Constructor),
                    OrderKind::CarpentryJob => !self.colonists[c_idx]
                        .role_profile
                        .has_role(ColonyRole::Carpenter),
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
                            && !self.colonists[c_idx]
                                .role_profile
                                .has_role(ColonyRole::Hauler)
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
