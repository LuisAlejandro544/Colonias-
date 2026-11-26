//! Serialización y restauración de snapshots entre `SimulationCore` y Godot (`gdext`).
//!
//! # Lógica del módulo
//! Aísla el mapeo de tipos `VarDictionary` / `VarArray` / `PackedInt32Array` de la
//! lógica pura de simulación:
//! - Reconstrucción segura de partidas guardadas en JSON (`restore_colony_core`).
//! - Snapshots por frame de colonos (`colonists_snapshot`) y detalle de roles (`colonist_roles_detail`).
//! - Snapshot del mundo e inventario más eventos SFX (`world_snapshot`).

use godot::prelude::*;

use super::core::SimulationCore;
use crate::colony_types::{
    CarpentryBenchState, CarpentryRecipe, Colonist, ColonistActivity, ColonistInventory,
    ColonyOrder, CropCell, CropStage, ItemKind, OrderKind, OrderStatus, StructureKind,
};
use crate::roles::{
    effect_summary_es, xp_required_for_next_level, ColonistRoleProfile, ColonyRole,
    MAX_ASSIGNED_ROLES, MAX_ROLE_LEVEL,
};
use crate::stats::{ColonistStats, StatKind, MAX_STAT_VALUE};
use crate::tools::CopperTool;

#[allow(clippy::too_many_arguments)]
pub(super) fn restore_colony_core(
    seed: u64,
    width: i32,
    height: i32,
    ground: Vec<i32>,
    props: Vec<i32>,
    blocked: Vec<i32>,
    saved_state: &VarDictionary,
) -> SimulationCore {
    let mut core = SimulationCore::new(seed, width, height, ground, props, blocked);

    core.day_count = saved_state
        .get("day")
        .and_then(|v| v.try_to::<i32>().ok())
        .unwrap_or(1)
        .max(1);

    if let Some(arr) = saved_state
        .get("props")
        .and_then(|v| v.try_to::<PackedInt32Array>().ok())
    {
        let v = arr.to_vec();
        if v.len() == core.props.len() {
            core.props = v;
        }
    }
    if let Some(arr) = saved_state
        .get("blocked")
        .and_then(|v| v.try_to::<PackedInt32Array>().ok())
    {
        let v = arr.to_vec();
        if v.len() == core.blocked.len() {
            core.blocked = v;
        }
    }
    if let Some(arr) = saved_state
        .get("zones")
        .and_then(|v| v.try_to::<PackedInt32Array>().ok())
    {
        let v = arr.to_vec();
        if v.len() == core.zones.len() {
            core.zones = v;
        }
    }
    if let Some(arr) = saved_state
        .get("structures")
        .and_then(|v| v.try_to::<PackedInt32Array>().ok())
    {
        let v = arr.to_vec();
        if v.len() == core.structures.len() {
            core.structures = v;
        }
    }

    if let Some(crops_arr) = saved_state
        .get("crops")
        .and_then(|v| v.try_to::<VarArray>().ok())
    {
        for entry in crops_arr.iter_shared() {
            if let Ok(d) = entry.try_to::<VarDictionary>() {
                let x = d.get("x").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                let y = d.get("y").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                let stage_i = d.get("stage").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                let progress = d
                    .get("progress")
                    .and_then(|v| v.try_to::<f32>().ok())
                    .unwrap_or(0.0);
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

    if let Some(items_arr) = saved_state
        .get("items")
        .and_then(|v| v.try_to::<VarArray>().ok())
    {
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
                    let amount = d
                        .get("amount")
                        .and_then(|v| v.try_to::<i32>().ok())
                        .unwrap_or(1);
                    let stored = d
                        .get("stored")
                        .and_then(|v| v.try_to::<bool>().ok())
                        .unwrap_or(false);
                    core.spawn_item(kind, x, y, amount, stored);
                }
            }
        }
    }

    if let Some(orders_arr) = saved_state
        .get("orders")
        .and_then(|v| v.try_to::<VarArray>().ok())
    {
        for entry in orders_arr.iter_shared() {
            if let Ok(d) = entry.try_to::<VarDictionary>() {
                let kind_i = d.get("kind").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(1);
                let x = d.get("x").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                let y = d.get("y").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                let status_i = d
                    .get("status")
                    .and_then(|v| v.try_to::<i32>().ok())
                    .unwrap_or(0);
                let kind = match kind_i {
                    1 => OrderKind::ChopTree,
                    2 => OrderKind::MineRock,
                    3 => OrderKind::BuildWoodWall,
                    4 => OrderKind::FarmCell,
                    6 => OrderKind::BuildWoodDoor,
                    7 => OrderKind::BuildCarpenterBench,
                    8 => OrderKind::CarpentryJob,
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

    if let Some(benches_arr) = saved_state
        .get("benches")
        .and_then(|v| v.try_to::<VarArray>().ok())
    {
        for entry in benches_arr.iter_shared() {
            if let Ok(d) = entry.try_to::<VarDictionary>() {
                let id = d
                    .get("id")
                    .and_then(|v| v.try_to::<i32>().ok())
                    .unwrap_or(core.next_bench_id)
                    .max(1);
                let x = d.get("x").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                let y = d.get("y").and_then(|v| v.try_to::<i32>().ok()).unwrap_or(0);
                let mut queue = Vec::new();
                if let Some(q_arr) = d.get("queue").and_then(|v| v.try_to::<VarArray>().ok()) {
                    for qv in q_arr.iter_shared() {
                        if let Ok(gs) = qv.try_to::<GString>() {
                            if let Some(recipe) = CarpentryRecipe::from_code(&gs.to_string()) {
                                queue.push(recipe);
                            }
                        }
                    }
                }
                core.next_bench_id = core.next_bench_id.max(id + 1);
                core.benches.push(CarpentryBenchState { id, x, y, queue });
            }
        }
    }

    // Reconstrucción defensiva si el mapa tiene estructuras CarpenterBench sin registro en benches.
    for y in 0..core.map_height {
        for x in 0..core.map_width {
            let idx = (y * core.map_width + x) as usize;
            if core.structures[idx] == StructureKind::CarpenterBench as i32
                && !core.benches.iter().any(|b| b.x == x && b.y == y)
            {
                let id = core.next_bench_id;
                core.next_bench_id += 1;
                core.benches.push(CarpentryBenchState {
                    id,
                    x,
                    y,
                    queue: Vec::new(),
                });
            }
        }
    }

    if let Some(cols_arr) = saved_state
        .get("colonists")
        .and_then(|v| v.try_to::<VarArray>().ok())
    {
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
                let health = dict
                    .get("health")
                    .and_then(|v| v.try_to::<i32>().ok())
                    .unwrap_or(100);
                let mood = dict
                    .get("mood")
                    .and_then(|v| v.try_to::<i32>().ok())
                    .unwrap_or(80);
                let body_style = dict
                    .get("body_style")
                    .and_then(|v| v.try_to::<i32>().ok())
                    .unwrap_or(0);
                let hair_style = dict
                    .get("hair_style")
                    .and_then(|v| v.try_to::<i32>().ok())
                    .unwrap_or(0);
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

                let mut role_profile =
                    ColonistRoleProfile::new_for_colonist(core.colonists.len(), is_leader);
                if let Some(roles_arr) = dict
                    .get("assigned_roles")
                    .and_then(|v| v.try_to::<VarArray>().ok())
                {
                    let mut parsed_roles: Vec<ColonyRole> = Vec::new();
                    for rv in roles_arr.iter_shared() {
                        if let Ok(gs) = rv.try_to::<GString>() {
                            if let Some(r) = ColonyRole::from_code(&gs.to_string()) {
                                if !parsed_roles.contains(&r)
                                    && parsed_roles.len() < MAX_ASSIGNED_ROLES
                                {
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

                if let Some(levels_dict) = dict
                    .get("role_levels")
                    .and_then(|v| v.try_to::<VarDictionary>().ok())
                {
                    let xp_dict = dict
                        .get("role_xp")
                        .and_then(|v| v.try_to::<VarDictionary>().ok());
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

                let mut stats = ColonistStats::generate_for_colonist(
                    &mut core.rng,
                    core.colonists.len() as i32,
                    is_leader,
                );
                if let Some(stats_dict) = dict
                    .get("stats")
                    .and_then(|v| v.try_to::<VarDictionary>().ok())
                {
                    for sk in StatKind::ALL {
                        if let Some(val) = stats_dict
                            .get(sk.code())
                            .and_then(|v| v.try_to::<i32>().ok())
                        {
                            stats.set(sk, val);
                        }
                    }
                }
                if let Some(xp_arr) = dict
                    .get("stat_xp")
                    .and_then(|v| v.try_to::<VarArray>().ok())
                {
                    for (idx, xv) in xp_arr.iter_shared().take(5).enumerate() {
                        if let Ok(xp_val) = xv.try_to::<f32>() {
                            stats.stat_xp[idx] = xp_val.max(0.0);
                        }
                    }
                }
                let max_health = dict
                    .get("max_health")
                    .and_then(|v| v.try_to::<i32>().ok())
                    .unwrap_or_else(|| stats.max_health())
                    .max(60);
                let clean_health = health.clamp(1, max_health);

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
                    health: clean_health,
                    max_health,
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
                    stats,
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
    core
}

fn stat_effect_summary_es(stats: &ColonistStats, kind: StatKind) -> String {
    match kind {
        StatKind::Intelligence => format!(
            "Aprendizaje x{:.2} • Botánica x{:.2} • {}",
            stats.xp_gain_mult(),
            stats.botany_intel_mult(),
            stats.craft_tier_label_es()
        ),
        StatKind::Strength => format!(
            "+{} carga por viaje • Tala x{:.2} • Minería x{:.2}",
            stats.carry_bonus(),
            stats.chop_strength_mult(),
            stats.mine_strength_mult()
        ),
        StatKind::Dexterity => format!(
            "Paso x{:.2} • Construcción x{:.2} • Carpintería x{:.2}",
            stats.move_speed_mult(),
            stats.build_dex_mult(),
            stats.craft_dex_mult()
        ),
        StatKind::Endurance => format!(
            "Salud máx: {} • Desgaste energía x{:.2}",
            stats.max_health(),
            stats.energy_drain_mult()
        ),
        StatKind::Charisma => format!(
            "Reuniones x{:.2} • Ánimo grupal +{}",
            stats.meeting_charisma_mult(),
            stats.meeting_mood_boost()
        ),
    }
}

pub(super) fn colonists_snapshot(core: &SimulationCore) -> VarArray {
    let mut out = VarArray::new();
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
        d.set("max_health", &c.max_health.to_variant());
        d.set("mood", &c.mood.to_variant());
        d.set("energy", &(c.energy.round() as i32).to_variant());
        d.set("walk_phase", &c.walk_phase.to_variant());
        d.set("work_swing_phase", &c.work_swing_phase.to_variant());
        let is_moving = !c.path.is_empty()
            || matches!(
                c.activity,
                ColonistActivity::Wandering
                    | ColonistActivity::Scouting
                    | ColonistActivity::MovingToTask
            );
        let is_working = matches!(
            c.activity,
            ColonistActivity::Chopping
                | ColonistActivity::Mining
                | ColonistActivity::Building
                | ColonistActivity::Carpentry
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
        d.set("helmet_slot", &GString::from(&c.inventory.helmet_slot).to_variant());
        d.set("armor_slot", &GString::from(&c.inventory.armor_slot).to_variant());
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

        let mut stats_dict = VarDictionary::new();
        let mut stat_xp_arr = VarArray::new();
        let mut stats_detail_arr = VarArray::new();
        for sk in StatKind::ALL {
            let val = c.stats.get(sk);
            let xp = c.stats.stat_xp[sk.index()];
            let xp_next = ColonistStats::xp_needed_for_next(val);
            stats_dict.set(sk.code(), &val.to_variant());
            stat_xp_arr.push(&xp.to_variant());

            let mut sd = VarDictionary::new();
            sd.set("code", &GString::from(sk.code()).to_variant());
            sd.set("short", &GString::from(sk.short_es()).to_variant());
            sd.set("label", &GString::from(sk.label_es()).to_variant());
            sd.set("value", &val.to_variant());
            sd.set("max_value", &MAX_STAT_VALUE.to_variant());
            sd.set("xp", &xp.to_variant());
            sd.set("xp_next", &xp_next.to_variant());
            sd.set(
                "summary",
                &GString::from(&stat_effect_summary_es(&c.stats, sk)).to_variant(),
            );
            stats_detail_arr.push(&sd.to_variant());
        }
        d.set("stat_int", &c.stats.get(StatKind::Intelligence).to_variant());
        d.set("stat_str", &c.stats.get(StatKind::Strength).to_variant());
        d.set("stat_dex", &c.stats.get(StatKind::Dexterity).to_variant());
        d.set("stat_end", &c.stats.get(StatKind::Endurance).to_variant());
        d.set("stat_cha", &c.stats.get(StatKind::Charisma).to_variant());
        d.set(
            "craft_tier_label",
            &GString::from(c.stats.craft_tier_label_es()).to_variant(),
        );
        d.set("stats", &stats_dict.to_variant());
        d.set("stat_xp", &stat_xp_arr.to_variant());
        d.set("stats_detail", &stats_detail_arr.to_variant());

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

pub(super) fn colonist_roles_detail(core: &SimulationCore, colonist_id: i32) -> VarArray {
    let mut out = VarArray::new();
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
            &GString::from(&effect_summary_es(r, lv)).to_variant(),
        );
        out.push(&rd.to_variant());
    }
    out
}

pub(super) fn world_snapshot(core: &mut SimulationCore, force_full: bool) -> VarDictionary {
    let mut out = VarDictionary::new();
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
    out.set(
        "meeting_partner_name",
        &GString::from(meeting_partner_name).to_variant(),
    );
    out.set("pending_briefing_count", &pending_count.to_variant());

    let mut inv = VarDictionary::new();
    inv.set("wood", &total_inv.wood.to_variant());
    inv.set("stone", &total_inv.stone.to_variant());
    inv.set("iron", &total_inv.iron.to_variant());
    inv.set("copper", &total_inv.copper.to_variant());
    inv.set("carrot_seeds", &total_inv.carrot_seeds.to_variant());
    inv.set("carrots", &total_inv.carrots.to_variant());
    inv.set("wood_planks", &total_inv.wood_planks.to_variant());
    inv.set("wood_buckets", &total_inv.wood_buckets.to_variant());
    inv.set("wood_doors", &total_inv.wood_doors.to_variant());
    inv.set("wood_crates", &total_inv.wood_crates.to_variant());
    inv.set("stored_wood", &stored_inv.wood.to_variant());
    inv.set("stored_carrots", &stored_inv.carrots.to_variant());
    out.set("inventory", &inv.to_variant());

    if core.world_dirty || force_full {
        core.world_dirty = false;
        out.set("props", &PackedInt32Array::from(core.props.as_slice()).to_variant());
        out.set(
            "blocked",
            &PackedInt32Array::from(core.blocked.as_slice()).to_variant(),
        );
        out.set("zones", &PackedInt32Array::from(core.zones.as_slice()).to_variant());
        out.set(
            "structures",
            &PackedInt32Array::from(core.structures.as_slice()).to_variant(),
        );

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

        let mut benches_arr = VarArray::new();
        for bench in &core.benches {
            let mut bd = VarDictionary::new();
            bd.set("id", &bench.id.to_variant());
            bd.set("x", &bench.x.to_variant());
            bd.set("y", &bench.y.to_variant());
            let mut q_arr = VarArray::new();
            for r in &bench.queue {
                q_arr.push(&GString::from(r.code()).to_variant());
            }
            bd.set("queue", &q_arr.to_variant());
            benches_arr.push(&bd.to_variant());
        }
        out.set("benches", &benches_arr.to_variant());
    }

    let mut sfx_arr = VarArray::new();
    for ev in core.pending_sfx.drain(..) {
        sfx_arr.push(&GString::from(ev).to_variant());
    }
    out.set("sfx_events", &sfx_arr.to_variant());

    out
}

pub(super) fn carpentry_bench_snapshot(
    core: &SimulationCore,
    tile_x: i32,
    tile_y: i32,
) -> VarDictionary {
    let mut out = VarDictionary::new();
    let Some(bench) = core.find_bench_near(tile_x, tile_y) else {
        out.set("found", &false.to_variant());
        return out;
    };

    let (total_inv, _) = core.compute_inventory();
    let max_carpenter_int = core
        .colonists
        .iter()
        .filter(|c| c.role_profile.has_role(ColonyRole::Carpenter))
        .map(|c| c.stats.get(StatKind::Intelligence))
        .max()
        .unwrap_or(0);

    let mut carpenters_arr = VarArray::new();
    for c in &core.colonists {
        if c.role_profile.has_role(ColonyRole::Carpenter) {
            let mut cd = VarDictionary::new();
            cd.set("id", &c.id.to_variant());
            cd.set("name", &GString::from(&c.name).to_variant());
            cd.set(
                "intelligence",
                &c.stats.get(StatKind::Intelligence).to_variant(),
            );
            cd.set(
                "level",
                &c.role_profile.level_of(ColonyRole::Carpenter).to_variant(),
            );
            carpenters_arr.push(&cd.to_variant());
        }
    }

    let mut recipes_arr = VarArray::new();
    for r in CarpentryRecipe::ALL {
        let (in_kind, in_amt) = r.input_cost();
        let (out_kind, out_amt) = r.output_item();
        let available_input = core.available_ground_item(in_kind);
        let mut rd = VarDictionary::new();
        rd.set("code", &GString::from(r.code()).to_variant());
        rd.set("label", &GString::from(r.label_es()).to_variant());
        rd.set("description", &GString::from(r.description_es()).to_variant());
        rd.set("min_intelligence", &r.min_intelligence().to_variant());
        rd.set("input_code", &GString::from(in_kind.code()).to_variant());
        rd.set("input_label", &GString::from(in_kind.label_es()).to_variant());
        rd.set("input_amount", &in_amt.to_variant());
        rd.set("output_code", &GString::from(out_kind.code()).to_variant());
        rd.set("output_label", &GString::from(out_kind.label_es()).to_variant());
        rd.set("output_amount", &out_amt.to_variant());
        rd.set("available_input", &available_input.to_variant());
        rd.set(
            "can_craft_by_colony",
            &(max_carpenter_int >= r.min_intelligence()).to_variant(),
        );
        recipes_arr.push(&rd.to_variant());
    }

    let mut queue_arr = VarArray::new();
    for (idx, r) in bench.queue.iter().enumerate() {
        let mut qd = VarDictionary::new();
        qd.set("index", &(idx as i32).to_variant());
        qd.set("code", &GString::from(r.code()).to_variant());
        qd.set("label", &GString::from(r.label_es()).to_variant());
        qd.set("min_intelligence", &r.min_intelligence().to_variant());
        queue_arr.push(&qd.to_variant());
    }

    let mut inv_dict = VarDictionary::new();
    inv_dict.set("wood", &total_inv.wood.to_variant());
    inv_dict.set("wood_planks", &total_inv.wood_planks.to_variant());
    inv_dict.set("wood_buckets", &total_inv.wood_buckets.to_variant());
    inv_dict.set("wood_doors", &total_inv.wood_doors.to_variant());
    inv_dict.set("wood_crates", &total_inv.wood_crates.to_variant());

    out.set("found", &true.to_variant());
    out.set("id", &bench.id.to_variant());
    out.set("x", &bench.x.to_variant());
    out.set("y", &bench.y.to_variant());
    out.set("max_carpenter_int", &max_carpenter_int.to_variant());
    out.set("carpenters", &carpenters_arr.to_variant());
    out.set("recipes", &recipes_arr.to_variant());
    out.set("queue", &queue_arr.to_variant());
    out.set("inventory", &inv_dict.to_variant());
    out
}
