//! Simulación modular de colonos terrestres en vista centinela (estilo RimWorld).
//!
//! # Lógica del módulo
//! Administra el estado, apariencia natural (piel, cabello, vestimenta), roles
//! terrestres y movimiento autónomo de cada colono respetando las casillas
//! bloqueadas por árboles y rocas. Expone snapshots a Godot para renderizar
//! los cuerpos por capas sin saturar el hilo principal en dispositivos móviles.

use godot::classes::{INode, Node};
use godot::prelude::*;

use crate::noise::SeededRng;

const FIRST_NAMES: &[&str] = &[
    "Mateo", "Lucía", "Valentina", "Santiago", "Camila", "Diego",
    "Elena", "Andrés", "Sofía", "Tomás", "Valeria", "Nicolás",
    "Clara", "Gabriel", "Paula", "Hugo",
];

const ROLES: &[&str] = &[
    "Leñador", "Minero", "Constructor", "Botánico", "Médico", "Explorador",
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

/// Estado de actividad actual de un colono.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ColonistActivity {
    Resting = 0,
    Wandering = 1,
    Scouting = 2,
}

impl ColonistActivity {
    pub fn label_es(self) -> &'static str {
        match self {
            Self::Resting => "Observando el claro",
            Self::Wandering => "Recorriendo la colonia",
            Self::Scouting => "Inspeccionando recursos",
        }
    }
}

/// Representación interna de un colono en simulación.
#[derive(Clone, Debug)]
pub struct Colonist {
    pub id: i32,
    pub name: String,
    pub role: String,
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub facing_x: f32,
    pub facing_y: f32,
    pub speed: f32,
    pub health: i32,
    pub mood: i32,
    pub wait_timer: f32,
    pub walk_phase: f32,
    pub body_style: i32,
    pub hair_style: i32,
    pub skin_color: String,
    pub hair_color: String,
    pub outfit_color: String,
    pub activity: ColonistActivity,
}

/// Estado completo de la simulación en Rust.
pub struct SimulationCore {
    pub map_width: i32,
    pub map_height: i32,
    pub blocked: Vec<i32>,
    pub colonists: Vec<Colonist>,
    pub rng: SeededRng,
}

impl SimulationCore {
    pub fn new(seed: u64, width: i32, height: i32, blocked: Vec<i32>) -> Self {
        Self {
            map_width: width.max(16),
            map_height: height.max(16),
            blocked,
            colonists: Vec::new(),
            rng: SeededRng::new(seed ^ 0xC010_414A_0001_0002),
        }
    }

    fn tile_blocked(map_width: i32, map_height: i32, blocked: &[i32], tx: i32, ty: i32) -> bool {
        if tx < 1 || ty < 1 || tx >= map_width - 1 || ty >= map_height - 1 {
            return true;
        }
        let idx = (ty * map_width + tx) as usize;
        blocked.get(idx).copied().unwrap_or(1) != 0
    }

    /// Crea los colonos iniciales distribuidos en el claro central del mapa.
    pub fn spawn_initial(&mut self, count: i32) {
        self.colonists.clear();
        let total = count.clamp(1, 10);
        let cx = self.map_width as f32 * 0.5;
        let cy = self.map_height as f32 * 0.5;

        for i in 0..total {
            let angle = (i as f32 / total as f32) * std::f32::consts::TAU;
            let radius = 1.6 + (i % 2) as f32 * 0.9;
            let sx = (cx + angle.cos() * radius).clamp(2.0, (self.map_width - 3) as f32);
            let sy = (cy + angle.sin() * radius).clamp(2.0, (self.map_height - 3) as f32);

            let name_idx = (self.rng.next_usize(FIRST_NAMES.len()) + i as usize) % FIRST_NAMES.len();
            let role_idx = i as usize % ROLES.len();
            let skin_idx = self.rng.next_usize(SKIN_HEX.len());
            let hair_idx = self.rng.next_usize(HAIR_HEX.len());
            let outfit_idx = self.rng.next_usize(OUTFIT_HEX.len());

            self.colonists.push(Colonist {
                id: i + 1,
                name: FIRST_NAMES[name_idx].to_string(),
                role: ROLES[role_idx].to_string(),
                x: sx,
                y: sy,
                target_x: sx,
                target_y: sy,
                facing_x: 0.0,
                facing_y: 1.0,
                speed: 2.15 + self.rng.next_f32() * 0.45,
                health: 100,
                mood: 82 + self.rng.range_i32(-8, 12),
                wait_timer: 0.4 + self.rng.next_f32() * 1.8,
                walk_phase: 0.0,
                body_style: self.rng.range_i32(0, 1),
                hair_style: self.rng.range_i32(0, 2),
                skin_color: SKIN_HEX[skin_idx].to_string(),
                hair_color: HAIR_HEX[hair_idx].to_string(),
                outfit_color: OUTFIT_HEX[outfit_idx].to_string(),
                activity: ColonistActivity::Resting,
            });
        }
    }

    /// Avanza un paso de simulación para todos los colonos.
    pub fn step(&mut self, delta: f32) {
        let dt = delta.clamp(0.0, 0.1);
        if dt <= 0.0 {
            return;
        }

        let w = self.map_width;
        let h = self.map_height;
        let blocked = &self.blocked;

        for colonist in &mut self.colonists {
            if colonist.wait_timer > 0.0 {
                colonist.wait_timer -= dt;
                colonist.activity = ColonistActivity::Resting;
                continue;
            }

            let dx = colonist.target_x - colonist.x;
            let dy = colonist.target_y - colonist.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 0.12 {
                // Elegimos un nuevo punto despejado cercano para patrullar el claro.
                let mut picked = false;
                for _ in 0..8 {
                    let ox = self.rng.range_i32(-6, 6);
                    let oy = self.rng.range_i32(-6, 6);
                    let nx = (colonist.x.round() as i32 + ox).clamp(2, w - 3);
                    let ny = (colonist.y.round() as i32 + oy).clamp(2, h - 3);
                    if !Self::tile_blocked(w, h, blocked, nx, ny) {
                        colonist.target_x = nx as f32 + 0.5;
                        colonist.target_y = ny as f32 + 0.5;
                        colonist.activity = if self.rng.next_f32() < 0.35 {
                            ColonistActivity::Scouting
                        } else {
                            ColonistActivity::Wandering
                        };
                        picked = true;
                        break;
                    }
                }
                if !picked {
                    colonist.wait_timer = 1.2;
                } else {
                    colonist.wait_timer = if self.rng.next_f32() < 0.4 {
                        1.0 + self.rng.next_f32() * 2.2
                    } else {
                        0.0
                    };
                }
                continue;
            }

            let nx = dx / dist;
            let ny = dy / dist;
            let step_dist = (colonist.speed * dt).min(dist);
            let next_x = colonist.x + nx * step_dist;
            let next_y = colonist.y + ny * step_dist;

            let tile_x = next_x.floor() as i32;
            let tile_y = next_y.floor() as i32;
            let hits_obstacle = Self::tile_blocked(w, h, blocked, tile_x, tile_y);

            if hits_obstacle {
                // Si tropieza con un árbol o roca, detiene este trayecto y elige otro luego.
                colonist.target_x = colonist.x;
                colonist.target_y = colonist.y;
                colonist.wait_timer = 0.6;
                colonist.activity = ColonistActivity::Resting;
            } else {
                colonist.x = next_x;
                colonist.y = next_y;
                colonist.facing_x = nx;
                colonist.facing_y = ny;
                colonist.walk_phase += dt * colonist.speed * 4.5;
            }
        }
    }
}

/// Nodo GDExtension que administra la simulación de colonos en tiempo real.
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
    /// Inicializa la simulación sobre el mapa de obstáculos y genera los colonos iniciales.
    #[func]
    pub fn setup_colony(
        &mut self,
        seed: i64,
        width: i32,
        height: i32,
        blocked_tiles: PackedInt32Array,
        colonist_count: i32,
    ) {
        let mut core = SimulationCore::new(seed as u64, width, height, blocked_tiles.to_vec());
        core.spawn_initial(colonist_count);
        self.core = Some(core);
    }

    /// Restaura una lista de colonos guardados (desde archivo JSON en `user://`).
    #[func]
    pub fn restore_colonists(
        &mut self,
        seed: i64,
        width: i32,
        height: i32,
        blocked_tiles: PackedInt32Array,
        saved_colonists: VarArray,
    ) {
        let mut core = SimulationCore::new(seed as u64, width, height, blocked_tiles.to_vec());
        for item in saved_colonists.iter_shared() {
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
                let x = dict.get("x").and_then(|v| v.try_to::<f32>().ok()).unwrap_or(width as f32 * 0.5);
                let y = dict.get("y").and_then(|v| v.try_to::<f32>().ok()).unwrap_or(height as f32 * 0.5);
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

                core.colonists.push(Colonist {
                    id,
                    name,
                    role,
                    x,
                    y,
                    target_x: x,
                    target_y: y,
                    facing_x: 0.0,
                    facing_y: 1.0,
                    speed: 2.25,
                    health,
                    mood,
                    wait_timer: 0.5,
                    walk_phase: 0.0,
                    body_style,
                    hair_style,
                    skin_color,
                    hair_color,
                    outfit_color,
                    activity: ColonistActivity::Resting,
                });
            }
        }
        self.core = Some(core);
    }

    /// Avanza la simulación un paso de tiempo `delta` (segundos escalados por velocidad).
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
            d.set("x", &c.x.to_variant());
            d.set("y", &c.y.to_variant());
            d.set("facing_x", &c.facing_x.to_variant());
            d.set("facing_y", &c.facing_y.to_variant());
            d.set("health", &c.health.to_variant());
            d.set("mood", &c.mood.to_variant());
            d.set("walk_phase", &c.walk_phase.to_variant());
            d.set("is_moving", &(c.activity != ColonistActivity::Resting).to_variant());
            d.set("activity", &GString::from(c.activity.label_es()).to_variant());
            d.set("body_style", &c.body_style.to_variant());
            d.set("hair_style", &c.hair_style.to_variant());
            d.set("skin_color", &GString::from(&c.skin_color).to_variant());
            d.set("hair_color", &GString::from(&c.hair_color).to_variant());
            d.set("outfit_color", &GString::from(&c.outfit_color).to_variant());
            out.push(&d.to_variant());
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_requested_number_of_colonists() {
        let blocked = vec![0i32; 32 * 32];
        let mut core = SimulationCore::new(100, 32, 32, blocked);
        core.spawn_initial(4);
        assert_eq!(core.colonists.len(), 4);
    }

    #[test]
    fn colonists_move_without_entering_blocked_tiles() {
        let mut blocked = vec![0i32; 32 * 32];
        // Rodeamos un anillo lejano como bloqueado
        for y in 0..32 {
            for x in 0..32 {
                if x < 10 || x > 22 || y < 10 || y > 22 {
                    blocked[y * 32 + x] = 1;
                }
            }
        }
        let mut core = SimulationCore::new(999, 32, 32, blocked);
        core.spawn_initial(3);
        for _ in 0..240 {
            core.step(0.05);
        }
        for c in &core.colonists {
            assert!(!SimulationCore::tile_blocked(
                core.map_width,
                core.map_height,
                &core.blocked,
                c.x.floor() as i32,
                c.y.floor() as i32
            ));
        }
    }
}
