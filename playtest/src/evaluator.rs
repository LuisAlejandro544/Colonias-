//! Evaluador headless de partidas de Colonia Centinela para neuroevolución y QA.
//!
//! # Lógica del módulo
//! Conecta `BrainGenome` directamente con `SimulationCore` y `generate_world_data`
//! en Rust puro (sin Godot ni autoclicks):
//! - Traduce el estado de la colonia a 24 sensores normalizados (`extract_sensors`).
//! - Ejecuta la acción macro seleccionada (`execute_commander_action`) llamando
//!   a `designate_area`, `queue_carpentry_recipe` y `queue_masonry_recipe`.
//! - Simula episodios completos en los 3 biomas terrestres detectando atascos de
//!   colonos, deshidratación, saturación de órdenes y hitos alcanzados.
//! - Genera tanto `playtest_report.json` como `gemma_prompt_input.txt` para el
//!   análisis con Gemma 4 E2B en modo de razonamiento.

use std::collections::HashMap;

use colonia_core::colony_types::{
    CarpentryRecipe, ColonistActivity, MasonryRecipe, StructureKind, WorkshopKind, ZoneKind,
};
use colonia_core::{generate_world_data, SimulationCore};

use crate::neuro::{BrainGenome, INPUT_COUNT, OUTPUT_COUNT};

pub const ACTION_NAMES: [&str; OUTPUT_COUNT] = [
    "Esperar",
    "CrearAlmacen",
    "TalarArboles",
    "MinarRocas",
    "CultivoZanahoria",
    "CultivoMixto",
    "ConstruirCarpintero",
    "EncolarTablones",
    "EncolarCubo",
    "EncolarVaso",
    "ConstruirPozo",
    "ConstruirMason",
    "EncolarLadrillos",
    "ConstruirHorno",
    "EncolarLingoteCobre",
    "CazarFauna",
];

/// Métricas detalladas recopiladas durante una simulación de colonia.
#[derive(Clone, Debug)]
pub struct EpisodeTelemetry {
    pub biome: &'static str,
    pub seed: u64,
    pub days_simulated: i32,
    pub fitness: f32,
    pub min_hydration_seen: f32,
    pub avg_mood_end: f32,
    pub dehydration_ticks: usize,
    pub stuck_colonist_events: usize,
    pub max_pending_orders: usize,
    pub final_pending_orders: usize,
    pub storage_tiles: usize,
    pub farm_tiles: usize,
    pub has_carpenter_bench: bool,
    pub has_water_well: bool,
    pub has_mason_bench: bool,
    pub has_stone_furnace: bool,
    pub wood_end: i32,
    pub stone_end: i32,
    pub planks_end: i32,
    pub buckets_end: i32,
    pub cups_end: i32,
    pub bricks_end: i32,
    pub copper_ingots_end: i32,
    pub crops_harvested_end: i32,
    pub meat_end: i32,
    pub effective_actions: usize,
    pub rejected_actions: usize,
    pub action_counts: [usize; OUTPUT_COUNT],
    pub anomalies: Vec<String>,
}

#[derive(Clone, Debug)]
struct StuckTracker {
    last_tx: i32,
    last_ty: i32,
    same_tile_secs: f32,
    reported: bool,
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// Convierte el estado actual de `SimulationCore` en 24 entradas normalizadas `0..=1`.
pub fn extract_sensors(core: &SimulationCore) -> [f32; INPUT_COUNT] {
    let (inv, _) = core.compute_inventory();

    let storage_tiles = core
        .zones
        .iter()
        .filter(|&&z| z == ZoneKind::Storage as i32)
        .count();
    let farm_tiles = core
        .zones
        .iter()
        .filter(|&&z| z == ZoneKind::Farm as i32)
        .count();

    let has_carpenter = !core.benches.is_empty()
        || core
            .structures
            .iter()
            .any(|&s| s == StructureKind::CarpenterBenchBlueprint as i32);
    let has_well = core.structures.iter().any(|&s| {
        s == StructureKind::WaterWell as i32 || s == StructureKind::WaterWellBlueprint as i32
    });
    let has_mason = core
        .workshops
        .iter()
        .any(|w| w.kind == WorkshopKind::MasonBench)
        || core
            .structures
            .iter()
            .any(|&s| s == StructureKind::MasonBenchBlueprint as i32);
    let has_furnace = core
        .workshops
        .iter()
        .any(|w| w.kind == WorkshopKind::StoneFurnace)
        || core
            .structures
            .iter()
            .any(|&s| s == StructureKind::StoneFurnaceBlueprint as i32);

    let col_count = core.colonists.len().max(1) as f32;
    let avg_energy = core.colonists.iter().map(|c| c.energy).sum::<f32>() / col_count;
    let min_hydration = core
        .colonists
        .iter()
        .map(|c| c.hydration)
        .fold(100.0f32, f32::min);
    let avg_mood = core.colonists.iter().map(|c| c.mood as f32).sum::<f32>() / col_count;
    let idle_count = core
        .colonists
        .iter()
        .filter(|c| {
            c.active_task.is_none()
                && matches!(
                    c.activity,
                    ColonistActivity::Resting
                        | ColonistActivity::Wandering
                        | ColonistActivity::Scouting
                )
        })
        .count() as f32;

    let food_total = inv.carrots + inv.potatoes + inv.wheat + inv.beets + inv.meat_cuts;
    let seeds_total = inv.carrot_seeds + inv.potato_seeds + inv.wheat_seeds + inv.beet_seeds;

    [
        clamp01(core.day_count as f32 / 10.0),
        if core.is_crop_season_active() { 1.0 } else { 0.0 },
        clamp01(inv.wood as f32 / 40.0),
        clamp01(inv.stone as f32 / 40.0),
        clamp01(inv.copper as f32 / 20.0),
        clamp01(inv.coal as f32 / 20.0),
        clamp01(inv.copper_ingots as f32 / 10.0),
        clamp01(inv.wood_planks as f32 / 20.0),
        clamp01(inv.wood_buckets as f32 / 5.0),
        clamp01(inv.wood_cups as f32 / 8.0),
        clamp01(inv.stone_bricks as f32 / 20.0),
        clamp01(food_total as f32 / 30.0),
        clamp01(seeds_total as f32 / 60.0),
        clamp01(storage_tiles as f32 / 12.0),
        clamp01(farm_tiles as f32 / 16.0),
        if has_carpenter { 1.0 } else { 0.0 },
        if has_well { 1.0 } else { 0.0 },
        if has_mason { 1.0 } else { 0.0 },
        if has_furnace { 1.0 } else { 0.0 },
        clamp01(avg_energy / 100.0),
        clamp01(min_hydration / 100.0),
        clamp01(avg_mood / 100.0),
        clamp01(idle_count / col_count),
        clamp01(core.orders.len() as f32 / 25.0),
    ]
}

fn find_nearest_props(core: &SimulationCore, allowed_props: &[i32], limit: usize) -> Vec<(i32, i32)> {
    let cx = core.map_width / 2;
    let cy = core.map_height / 2;
    let mut candidates: Vec<(i32, i32, i32)> = Vec::new();
    for y in 2..core.map_height - 2 {
        for x in 2..core.map_width - 2 {
            let idx = (y * core.map_width + x) as usize;
            let p = core.props[idx];
            if allowed_props.contains(&p) {
                let already_ordered = core.orders.iter().any(|o| o.x == x && o.y == y);
                if !already_ordered {
                    let dist = (x - cx).abs() + (y - cy).abs();
                    candidates.push((dist, x, y));
                }
            }
        }
    }
    candidates.sort_by_key(|c| c.0);
    candidates
        .into_iter()
        .take(limit)
        .map(|(_, x, y)| (x, y))
        .collect()
}

fn find_free_building_spot(core: &SimulationCore, min_radius: i32, max_radius: i32) -> Option<(i32, i32)> {
    let cx = core.map_width / 2;
    let cy = core.map_height / 2;
    let mut best: Option<(i32, i32, i32)> = None;
    for y in (cy - max_radius).max(2)..=(cy + max_radius).min(core.map_height - 3) {
        for x in (cx - max_radius).max(2)..=(cx + max_radius).min(core.map_width - 3) {
            let dist = (x - cx).abs() + (y - cy).abs();
            if dist < min_radius {
                continue;
            }
            let idx = (y * core.map_width + x) as usize;
            if core.blocked[idx] == 0
                && core.ground[idx] != 5
                && core.structures[idx] == StructureKind::None as i32
                && core.zones[idx] == ZoneKind::None as i32
            {
                let clear_neighbors = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .filter(|(ox, oy)| {
                        let nidx = ((y + oy) * core.map_width + (x + ox)) as usize;
                        core.blocked.get(nidx).copied().unwrap_or(1) == 0
                    })
                    .count();
                if clear_neighbors >= 2 {
                    match best {
                        Some((bd, _, _)) if dist >= bd => {}
                        _ => best = Some((dist, x, y)),
                    }
                }
            }
        }
    }
    best.map(|(_, x, y)| (x, y))
}

fn find_free_patch_2x2(core: &SimulationCore, prefer_fertile: bool) -> Option<(i32, i32)> {
    let cx = core.map_width / 2;
    let cy = core.map_height / 2;
    let mut best: Option<(i32, i32, i32)> = None;
    for y in (cy - 12).max(2)..=(cy + 11).min(core.map_height - 4) {
        for x in (cx - 12).max(2)..=(cx + 11).min(core.map_width - 4) {
            let mut valid = true;
            let mut fertile_hits = 0i32;
            for dy in 0..2 {
                for dx in 0..2 {
                    let idx = ((y + dy) * core.map_width + (x + dx)) as usize;
                    let g = core.ground[idx];
                    if core.blocked[idx] != 0
                        || core.structures[idx] != StructureKind::None as i32
                        || core.zones[idx] != ZoneKind::None as i32
                        || g == 4
                        || g == 5
                    {
                        valid = false;
                        break;
                    }
                    if g == 1 {
                        fertile_hits += 1;
                    }
                }
                if !valid {
                    break;
                }
            }
            if !valid {
                continue;
            }
            let dist = (x - cx).abs() + (y - cy).abs();
            let score = dist - if prefer_fertile { fertile_hits * 3 } else { 0 };
            match best {
                Some((bs, _, _)) if score >= bs => {}
                _ => best = Some((score, x, y)),
            }
        }
    }
    best.map(|(_, x, y)| (x, y))
}

/// Ejecuta una de las 16 acciones macro sobre `SimulationCore` y devuelve `true` si tuvo efecto válido.
pub fn execute_commander_action(core: &mut SimulationCore, action_idx: usize, step_idx: usize) -> bool {
    // Evitar saturar al líder con decenas de órdenes sin aprobar al mismo tiempo.
    if action_idx != 0 && core.orders.len() >= 18 {
        return false;
    }

    match action_idx {
        0 => true, // Esperar
        1 => {
            let storage_count = core
                .zones
                .iter()
                .filter(|&&z| z == ZoneKind::Storage as i32)
                .count();
            if storage_count >= 12 {
                return false;
            }
            if let Some((x, y)) = find_free_patch_2x2(core, false) {
                core.designate_area("storage", x, y, x + 1, y + 1) > 0
            } else {
                false
            }
        }
        2 => {
            let targets = find_nearest_props(core, &[1, 2], 3);
            let mut total = 0;
            for (x, y) in targets {
                total += core.designate_area("chop", x, y, x, y);
            }
            total > 0
        }
        3 => {
            let targets = find_nearest_props(core, &[4, 5, 7, 8, 9, 10, 11], 3);
            let mut total = 0;
            for (x, y) in targets {
                total += core.designate_area("mine", x, y, x, y);
            }
            total > 0
        }
        4 => {
            let farm_count = core
                .zones
                .iter()
                .filter(|&&z| z == ZoneKind::Farm as i32)
                .count();
            if farm_count >= 16 {
                return false;
            }
            if let Some((x, y)) = find_free_patch_2x2(core, true) {
                core.designate_area("farm_carrot", x, y, x + 1, y + 1) > 0
            } else {
                false
            }
        }
        5 => {
            let farm_count = core
                .zones
                .iter()
                .filter(|&&z| z == ZoneKind::Farm as i32)
                .count();
            if farm_count >= 16 {
                return false;
            }
            let mode = match step_idx % 3 {
                0 => "farm_potato",
                1 => "farm_wheat",
                _ => "farm_beet",
            };
            if let Some((x, y)) = find_free_patch_2x2(core, true) {
                core.designate_area(mode, x, y, x + 1, y + 1) > 0
            } else {
                false
            }
        }
        6 => {
            let exists = !core.benches.is_empty()
                || core
                    .structures
                    .iter()
                    .any(|&s| s == StructureKind::CarpenterBenchBlueprint as i32);
            if exists {
                return false;
            }
            if let Some((x, y)) = find_free_building_spot(core, 2, 7) {
                core.designate_area("build_carpenter_bench", x, y, x, y) > 0
            } else {
                false
            }
        }
        7 => {
            let Some(bench_id) = core.benches.first().map(|b| b.id) else {
                return false;
            };
            let queue_len = core.benches.first().map(|b| b.queue.len()).unwrap_or(99);
            if queue_len >= 4 {
                return false;
            }
            core.queue_carpentry_recipe(bench_id, CarpentryRecipe::Planks)
                .is_ok()
        }
        8 => {
            let Some(bench_id) = core.benches.first().map(|b| b.id) else {
                return false;
            };
            let queue_len = core.benches.first().map(|b| b.queue.len()).unwrap_or(99);
            if queue_len >= 4 {
                return false;
            }
            core.queue_carpentry_recipe(bench_id, CarpentryRecipe::Bucket)
                .is_ok()
        }
        9 => {
            let Some(bench_id) = core.benches.first().map(|b| b.id) else {
                return false;
            };
            let queue_len = core.benches.first().map(|b| b.queue.len()).unwrap_or(99);
            if queue_len >= 4 {
                return false;
            }
            core.queue_carpentry_recipe(bench_id, CarpentryRecipe::Cup)
                .is_ok()
        }
        10 => {
            let exists = core.structures.iter().any(|&s| {
                s == StructureKind::WaterWell as i32
                    || s == StructureKind::WaterWellBlueprint as i32
            });
            if exists {
                return false;
            }
            if let Some((x, y)) = find_free_building_spot(core, 2, 7) {
                core.designate_area("build_well", x, y, x, y) > 0
            } else {
                false
            }
        }
        11 => {
            let exists = core
                .workshops
                .iter()
                .any(|w| w.kind == WorkshopKind::MasonBench)
                || core
                    .structures
                    .iter()
                    .any(|&s| s == StructureKind::MasonBenchBlueprint as i32);
            if exists {
                return false;
            }
            if let Some((x, y)) = find_free_building_spot(core, 3, 8) {
                core.designate_area("build_mason_bench", x, y, x, y) > 0
            } else {
                false
            }
        }
        12 => {
            let Some(ws_id) = core
                .workshops
                .iter()
                .find(|w| w.kind == WorkshopKind::MasonBench && w.queue.len() < 4)
                .map(|w| w.id)
            else {
                return false;
            };
            core.queue_masonry_recipe(ws_id, MasonryRecipe::Bricks)
                .is_ok()
        }
        13 => {
            let exists = core
                .workshops
                .iter()
                .any(|w| w.kind == WorkshopKind::StoneFurnace)
                || core
                    .structures
                    .iter()
                    .any(|&s| s == StructureKind::StoneFurnaceBlueprint as i32);
            if exists {
                return false;
            }
            if let Some((x, y)) = find_free_building_spot(core, 3, 9) {
                core.designate_area("build_furnace", x, y, x, y) > 0
            } else {
                false
            }
        }
        14 => {
            let Some(ws_id) = core
                .workshops
                .iter()
                .find(|w| w.kind == WorkshopKind::StoneFurnace && w.queue.len() < 4)
                .map(|w| w.id)
            else {
                return false;
            };
            core.queue_masonry_recipe(ws_id, MasonryRecipe::CopperIngot)
                .is_ok()
        }
        15 => {
            let cx = core.map_width as f32 * 0.5;
            let cy = core.map_height as f32 * 0.5;
            let target = core
                .animals
                .iter()
                .filter(|a| !a.marked_for_hunt)
                .min_by(|a, b| {
                    let da = (a.x - cx).powi(2) + (a.y - cy).powi(2);
                    let db = (b.x - cx).powi(2) + (b.y - cy).powi(2);
                    da.total_cmp(&db)
                })
                .map(|a| (a.x.floor() as i32, a.y.floor() as i32));
            if let Some((ax, ay)) = target {
                core.designate_area("hunt", ax, ay, ax, ay) > 0
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Evalúa un genoma en un bioma y semilla concretos durante `days` días de juego.
pub fn evaluate_episode(
    brain: &BrainGenome,
    seed: u64,
    biome: &'static str,
    days: i32,
) -> EpisodeTelemetry {
    let world = generate_world_data(seed, 48, 48, biome);
    let mut core = SimulationCore::new(
        seed,
        world.width,
        world.height,
        world.ground_tiles,
        world.prop_tiles,
        world.blocked_tiles,
    );
    core.biome_code = biome.to_string();
    core.spawn_initial(4);

    let decision_interval_secs = 5.0f32;
    let dt = 0.05f32;
    let substeps = (decision_interval_secs / dt).round() as usize;
    let total_decisions = ((days.max(1) as f32 * 90.0) / decision_interval_secs).round() as usize;

    let mut min_hydration_seen = 100.0f32;
    let mut dehydration_ticks = 0usize;
    let mut stuck_colonist_events = 0usize;
    let mut max_pending_orders = 0usize;
    let mut effective_actions = 0usize;
    let mut rejected_actions = 0usize;
    let mut action_counts = [0usize; OUTPUT_COUNT];
    let mut anomalies: Vec<String> = Vec::new();
    let mut trackers: HashMap<i32, StuckTracker> = HashMap::new();

    for step_idx in 0..total_decisions {
        let sensors = extract_sensors(&core);
        let action = brain.select_action(&sensors);
        action_counts[action] += 1;

        if action != 0 {
            if execute_commander_action(&mut core, action, step_idx) {
                effective_actions += 1;
            } else {
                rejected_actions += 1;
            }
        }

        for _ in 0..substeps {
            core.step(dt);
        }

        max_pending_orders = max_pending_orders.max(core.orders.len());

        for c in &core.colonists {
            if c.hydration < min_hydration_seen {
                min_hydration_seen = c.hydration;
            }
            if c.hydration <= 5.0 {
                dehydration_ticks += 1;
            }
            let tx = c.x.floor() as i32;
            let ty = c.y.floor() as i32;
            let entry = trackers.entry(c.id).or_insert(StuckTracker {
                last_tx: tx,
                last_ty: ty,
                same_tile_secs: 0.0,
                reported: false,
            });
            if c.active_task.is_some()
                && !matches!(
                    c.activity,
                    ColonistActivity::Sleeping
                        | ColonistActivity::ConveningMeeting
                        | ColonistActivity::AttendingMeeting
                )
                && tx == entry.last_tx
                && ty == entry.last_ty
            {
                entry.same_tile_secs += decision_interval_secs;
                if entry.same_tile_secs >= 100.0 && !entry.reported {
                    stuck_colonist_events += 1;
                    entry.reported = true;
                    if anomalies.len() < 8 {
                        anomalies.push(format!(
                            "Colono {} ({}) permaneció >100s en ({},{}) durante {:?}",
                            c.name, c.id, tx, ty, c.activity
                        ));
                    }
                }
            } else {
                entry.last_tx = tx;
                entry.last_ty = ty;
                entry.same_tile_secs = 0.0;
                entry.reported = false;
            }
        }
    }

    let (inv, _) = core.compute_inventory();
    let storage_tiles = core
        .zones
        .iter()
        .filter(|&&z| z == ZoneKind::Storage as i32)
        .count();
    let farm_tiles = core
        .zones
        .iter()
        .filter(|&&z| z == ZoneKind::Farm as i32)
        .count();
    let has_carpenter_bench = !core.benches.is_empty();
    let has_water_well = core
        .structures
        .iter()
        .any(|&s| s == StructureKind::WaterWell as i32);
    let has_mason_bench = core
        .workshops
        .iter()
        .any(|w| w.kind == WorkshopKind::MasonBench);
    let has_stone_furnace = core
        .workshops
        .iter()
        .any(|w| w.kind == WorkshopKind::StoneFurnace);

    let avg_mood_end =
        core.colonists.iter().map(|c| c.mood as f32).sum::<f32>() / core.colonists.len().max(1) as f32;
    let crops_harvested_end = inv.carrots + inv.potatoes + inv.wheat + inv.beets;

    if !has_water_well && min_hydration_seen <= 18.0 && anomalies.len() < 8 {
        anomalies.push(format!(
            "Colonia sin Pozo de Agua en {biome}: hidratación mínima cayó a {min_hydration_seen:.1}%"
        ));
    }
    if core.orders.len() >= 15 && anomalies.len() < 8 {
        anomalies.push(format!(
            "Acumulación alta de órdenes sin resolver al final ({}) en {biome}",
            core.orders.len()
        ));
    }

    // Función de aptitud multi-hito diseñada para premiar supervivencia real y progresión.
    let mut fitness = 0.0f32;
    fitness += (storage_tiles.min(12) as f32) * 1.5;
    fitness += (farm_tiles.min(16) as f32) * 1.6;
    if has_carpenter_bench {
        fitness += 28.0;
    }
    if has_water_well {
        fitness += 55.0;
    }
    if has_mason_bench {
        fitness += 35.0;
    }
    if has_stone_furnace {
        fitness += 45.0;
    }
    fitness += (inv.wood.min(40) as f32) * 0.45;
    fitness += (inv.stone.min(40) as f32) * 0.50;
    fitness += (inv.wood_planks.min(20) as f32) * 1.8;
    fitness += (inv.wood_buckets.min(4) as f32) * 10.0;
    fitness += (inv.wood_cups.min(6) as f32) * 3.5;
    fitness += (inv.stone_bricks.min(20) as f32) * 2.2;
    fitness += (inv.copper_ingots.min(8) as f32) * 12.0;
    fitness += (crops_harvested_end.min(40) as f32) * 2.4;
    fitness += (inv.meat_cuts.min(16) as f32) * 1.8;
    fitness += min_hydration_seen * 0.55;
    fitness += avg_mood_end * 0.45;
    fitness += (effective_actions.min(30) as f32) * 0.35;

    fitness -= (dehydration_ticks as f32) * 2.5;
    fitness -= (stuck_colonist_events as f32) * 18.0;
    fitness -= (core.orders.len() as f32) * 1.1;
    fitness -= (rejected_actions as f32) * 0.12;

    EpisodeTelemetry {
        biome,
        seed,
        days_simulated: core.day_count,
        fitness,
        min_hydration_seen,
        avg_mood_end,
        dehydration_ticks,
        stuck_colonist_events,
        max_pending_orders,
        final_pending_orders: core.orders.len(),
        storage_tiles,
        farm_tiles,
        has_carpenter_bench,
        has_water_well,
        has_mason_bench,
        has_stone_furnace,
        wood_end: inv.wood,
        stone_end: inv.stone,
        planks_end: inv.wood_planks,
        buckets_end: inv.wood_buckets,
        cups_end: inv.wood_cups,
        bricks_end: inv.stone_bricks,
        copper_ingots_end: inv.copper_ingots,
        crops_harvested_end,
        meat_end: inv.meat_cuts,
        effective_actions,
        rejected_actions,
        action_counts,
        anomalies,
    }
}

/// Evalúa un genoma promediando su desempeño en los 3 biomas del juego.
pub fn evaluate_genome_suite(brain: &BrainGenome, base_seed: u64, days: i32) -> (f32, Vec<EpisodeTelemetry>) {
    let biomes = [
        ("temperate_forest", base_seed),
        ("fertile_meadow", base_seed.wrapping_add(101)),
        ("rocky_highlands", base_seed.wrapping_add(202)),
    ];
    let mut episodes = Vec::with_capacity(biomes.len());
    let mut sum_fit = 0.0f32;
    for (biome, s) in biomes {
        let ep = evaluate_episode(brain, s, biome, days);
        sum_fit += ep.fitness;
        episodes.push(ep);
    }
    (sum_fit / episodes.len() as f32, episodes)
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Construye el reporte JSON completo para artefactos de CI.
pub fn build_report_json(
    generations: usize,
    population_size: usize,
    days_per_eval: i32,
    best_fitness_history: &[f32],
    best: &BrainGenome,
    episodes: &[EpisodeTelemetry],
) -> String {
    let hist_str: Vec<String> = best_fitness_history
        .iter()
        .map(|f| format!("{f:.2}"))
        .collect();

    let ep_blocks: Vec<String> = episodes
        .iter()
        .map(|ep| {
            let anom_list: Vec<String> = ep
                .anomalies
                .iter()
                .map(|a| format!("\"{}\"", json_escape(a)))
                .collect();
            let top_actions: Vec<String> = ep
                .action_counts
                .iter()
                .enumerate()
                .filter(|&(_, &cnt)| cnt > 0)
                .map(|(i, cnt)| format!("\"{}\": {}", ACTION_NAMES[i], cnt))
                .collect();
            format!(
                "    {{\n      \"biome\": \"{}\",\n      \"seed\": {},\n      \"days_simulated\": {},\n      \"fitness\": {:.2},\n      \"min_hydration_seen\": {:.1},\n      \"avg_mood_end\": {:.1},\n      \"dehydration_ticks\": {},\n      \"stuck_colonist_events\": {},\n      \"max_pending_orders\": {},\n      \"final_pending_orders\": {},\n      \"storage_tiles\": {},\n      \"farm_tiles\": {},\n      \"has_carpenter_bench\": {},\n      \"has_water_well\": {},\n      \"has_mason_bench\": {},\n      \"has_stone_furnace\": {},\n      \"wood_end\": {},\n      \"stone_end\": {},\n      \"planks_end\": {},\n      \"buckets_end\": {},\n      \"cups_end\": {},\n      \"bricks_end\": {},\n      \"copper_ingots_end\": {},\n      \"crops_harvested_end\": {},\n      \"meat_end\": {},\n      \"effective_actions\": {},\n      \"rejected_actions\": {},\n      \"actions_used\": {{{}}},\n      \"anomalies\": [{}]\n    }}",
                ep.biome,
                ep.seed,
                ep.days_simulated,
                ep.fitness,
                ep.min_hydration_seen,
                ep.avg_mood_end,
                ep.dehydration_ticks,
                ep.stuck_colonist_events,
                ep.max_pending_orders,
                ep.final_pending_orders,
                ep.storage_tiles,
                ep.farm_tiles,
                ep.has_carpenter_bench,
                ep.has_water_well,
                ep.has_mason_bench,
                ep.has_stone_furnace,
                ep.wood_end,
                ep.stone_end,
                ep.planks_end,
                ep.buckets_end,
                ep.cups_end,
                ep.bricks_end,
                ep.copper_ingots_end,
                ep.crops_harvested_end,
                ep.meat_end,
                ep.effective_actions,
                ep.rejected_actions,
                top_actions.join(", "),
                anom_list.join(", ")
            )
        })
        .collect();

    format!(
        "{{\n  \"generations\": {},\n  \"population_size\": {},\n  \"days_per_eval\": {},\n  \"best_genome_id\": {},\n  \"best_fitness\": {:.2},\n  \"fitness_history\": [{}],\n  \"episodes\": [\n{}\n  ]\n}}\n",
        generations,
        population_size,
        days_per_eval,
        best.id,
        best.fitness,
        hist_str.join(", "),
        ep_blocks.join(",\n")
    )
}

/// Construye un resumen compacto en texto plano optimizado para Gemma 4 E2B (CPU).
pub fn build_gemma_input_text(
    generations: usize,
    population_size: usize,
    days_per_eval: i32,
    best_fitness_history: &[f32],
    best: &BrainGenome,
    episodes: &[EpisodeTelemetry],
) -> String {
    let first_fit = best_fitness_history.first().copied().unwrap_or(0.0);
    let last_fit = best.fitness;
    let mut out = String::new();
    out.push_str("=== REPORTE DE NEUROEVOLUCION: COLONIA CENTINELA ===\n");
    out.push_str(&format!(
        "Generaciones: {generations} | Poblacion: {population_size} | Dias por episodio: {days_per_eval}\n"
    ));
    out.push_str(&format!(
        "Evolucion fitness: Gen 1 = {first_fit:.2} -> Mejor final = {last_fit:.2} (Genoma #{})\n\n",
        best.id
    ));

    for ep in episodes {
        let mut sorted_actions: Vec<(usize, usize)> = ep
            .action_counts
            .iter()
            .enumerate()
            .filter(|&(_, &c)| c > 0)
            .map(|(i, &c)| (i, c))
            .collect();
        sorted_actions.sort_by(|a, b| b.1.cmp(&a.1));
        let top_actions: Vec<String> = sorted_actions
            .into_iter()
            .take(6)
            .map(|(i, c)| format!("{}({})", ACTION_NAMES[i], c))
            .collect();

        out.push_str(&format!(
            "[Bioma: {} | Semilla: {}]\n",
            ep.biome, ep.seed
        ));
        out.push_str(&format!(
            "- Fitness: {:.2} | Hidratacion min: {:.1}% | Animo final: {:.1} | Deshidrataciones: {} | Colonos atascados: {}\n",
            ep.fitness, ep.min_hydration_seen, ep.avg_mood_end, ep.dehydration_ticks, ep.stuck_colonist_events
        ));
        out.push_str(&format!(
            "- Infraestructura: Almacen={} casillas, Cultivo={} casillas, Carpintero={}, Pozo={}, Mason={}, Horno={}\n",
            ep.storage_tiles,
            ep.farm_tiles,
            if ep.has_carpenter_bench { "SI" } else { "NO" },
            if ep.has_water_well { "SI" } else { "NO" },
            if ep.has_mason_bench { "SI" } else { "NO" },
            if ep.has_stone_furnace { "SI" } else { "NO" }
        ));
        out.push_str(&format!(
            "- Inventario final: madera={}, piedra={}, tablones={}, cubos={}, vasos={}, ladrillos={}, lingotes_cobre={}, cosechas={}, carne={}\n",
            ep.wood_end,
            ep.stone_end,
            ep.planks_end,
            ep.buckets_end,
            ep.cups_end,
            ep.bricks_end,
            ep.copper_ingots_end,
            ep.crops_harvested_end,
            ep.meat_end
        ));
        out.push_str(&format!(
            "- Ordenes: max={}, pendientes_fin={} | Acciones validas={}, rechazadas={} | Top acciones: {}\n",
            ep.max_pending_orders,
            ep.final_pending_orders,
            ep.effective_actions,
            ep.rejected_actions,
            top_actions.join(", ")
        ));
        if ep.anomalies.is_empty() {
            out.push_str("- Anomalias detectadas: Ninguna\n\n");
        } else {
            out.push_str(&format!(
                "- Anomalias detectadas: {}\n\n",
                ep.anomalies.join(" ; ")
            ));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neuro::Rng64;

    #[test]
    fn episode_evaluation_runs_cleanly_without_godot() {
        let mut rng = Rng64::new(42);
        let brain = BrainGenome::random(1, 0, &mut rng);
        let ep = evaluate_episode(&brain, 1234, "temperate_forest", 2);
        assert!(ep.days_simulated >= 2);
        assert_eq!(ep.biome, "temperate_forest");
    }
}
