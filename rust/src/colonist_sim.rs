//! Puente GDExtension para la simulación modular de colonos en Rust.
//!
//! # Lógica del módulo
//! Expone el nodo `ColonySimulation` hacia Godot 4.7 y orquesta los submódulos
//! independientes en `colonist_sim/`:
//! - `core`: estado `SimulationCore`, inicialización, bucle `step` y tests de integración.
//! - `designations`: marcado/cancelación de áreas, almacén e inventario del mundo.
//! - `leader`: Líder de Colonia, reuniones de órdenes y asignación de hasta 5 roles.
//! - `farming`: crecimiento de zanahorias y tareas de Agricultor/Botánico.
//! - `tasks`: selección de tareas, rutas A* y ejecución de tala, minería, construcción y acarreo.
//! - `persistence`: serialización/restauración de snapshots `VarDictionary` para guardado JSON.

mod core;
mod designations;
mod farming;
mod leader;
mod persistence;
mod tasks;

use godot::classes::{INode, Node};
use godot::prelude::*;

use self::core::SimulationCore;
use crate::roles::ColonyRole;

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
        let core = persistence::restore_colony_core(
            seed as u64,
            width,
            height,
            ground_tiles.to_vec(),
            prop_tiles.to_vec(),
            blocked_tiles.to_vec(),
            &saved_state,
        );
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
        let Some(core) = &self.core else {
            return VarArray::new();
        };
        persistence::colonists_snapshot(core)
    }

    /// Devuelve el detalle completo de los 9 roles y niveles (1..=10) de un colono para el panel de Trabajos.
    #[func]
    pub fn get_colonist_roles_detail(&self, colonist_id: i32) -> VarArray {
        let Some(core) = &self.core else {
            return VarArray::new();
        };
        persistence::colonist_roles_detail(core, colonist_id)
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
        let Some(core) = &mut self.core else {
            return VarDictionary::new();
        };
        persistence::world_snapshot(core, force_full)
    }

    /// Consulta la Mesa de Carpintero en una casilla (o vecina inmediata) para abrir su menú de fabricación.
    #[func]
    pub fn get_carpentry_bench_at(&self, tile_x: i32, tile_y: i32) -> VarDictionary {
        let Some(core) = &self.core else {
            let mut d = VarDictionary::new();
            d.set("found", &false.to_variant());
            return d;
        };
        persistence::carpentry_bench_snapshot(core, tile_x, tile_y)
    }

    /// Añade una receta a la cola de fabricación de una Mesa de Carpintero.
    #[func]
    pub fn queue_carpentry_recipe(&mut self, bench_id: i32, recipe_code: GString) -> VarDictionary {
        let mut res = VarDictionary::new();
        let Some(core) = &mut self.core else {
            res.set("ok", &false.to_variant());
            res.set("message", &GString::from("Simulación no iniciada.").to_variant());
            return res;
        };
        let Some(recipe) = crate::colony_types::CarpentryRecipe::from_code(&recipe_code.to_string()) else {
            res.set("ok", &false.to_variant());
            res.set("message", &GString::from("Receta desconocida.").to_variant());
            return res;
        };
        match core.queue_carpentry_recipe(bench_id, recipe) {
            Ok(()) => {
                res.set("ok", &true.to_variant());
                res.set("message", &GString::from("Pieza añadida a la cola.").to_variant());
            }
            Err(err) => {
                res.set("ok", &false.to_variant());
                res.set("message", &GString::from(err).to_variant());
            }
        }
        res
    }

    /// Cancela una pieza pendiente en la cola de la Mesa de Carpintero.
    #[func]
    pub fn cancel_carpentry_queue_item(&mut self, bench_id: i32, queue_index: i32) -> bool {
        let Some(core) = &mut self.core else {
            return false;
        };
        if queue_index < 0 {
            return false;
        }
        core.cancel_carpentry_queue_item(bench_id, queue_index as usize)
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
