//! Tipos de datos puros para zonas, cultivos, construcciones, ítems y órdenes.
//!
//! # Lógica del módulo
//! Centraliza las estructuras de dominio de la colonia (recursos en el piso,
//! bloques de pared de madera, zonas de almacenamiento y cultivo de zanahorias,
//! órdenes pendientes de reunión con el líder e inventario) manteniendo tipos
//! deterministas compatibles con 32 y 64 bits.

use crate::roles::ColonistRoleProfile;
use crate::stats::ColonistStats;
use crate::tools::CopperTool;

/// Recursos físicos que pueden caer al piso y transportarse al almacén.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ItemKind {
    WoodLogs = 1,
    StoneChunk = 2,
    IronOre = 3,
    CopperOre = 4,
    CarrotSeed = 5,
    Carrot = 6,
    WoodPlank = 7,
    WoodBucket = 8,
    WoodDoorItem = 9,
    WoodCrate = 10,
}

impl ItemKind {
    pub fn code(self) -> &'static str {
        match self {
            Self::WoodLogs => "wood_logs",
            Self::StoneChunk => "stone_chunk",
            Self::IronOre => "iron_ore",
            Self::CopperOre => "copper_ore",
            Self::CarrotSeed => "carrot_seed",
            Self::Carrot => "carrot",
            Self::WoodPlank => "wood_plank",
            Self::WoodBucket => "wood_bucket",
            Self::WoodDoorItem => "wood_door_item",
            Self::WoodCrate => "wood_crate",
        }
    }

    pub fn label_es(self) -> &'static str {
        match self {
            Self::WoodLogs => "Troncos de madera",
            Self::StoneChunk => "Bloques de piedra",
            Self::IronOre => "Mena de hierro",
            Self::CopperOre => "Mena de cobre",
            Self::CarrotSeed => "Semillas de zanahoria",
            Self::Carrot => "Zanahorias",
            Self::WoodPlank => "Tablones de madera",
            Self::WoodBucket => "Cubo de madera",
            Self::WoodDoorItem => "Puerta de madera",
            Self::WoodCrate => "Arcón de madera",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "wood_logs" => Some(Self::WoodLogs),
            "stone_chunk" => Some(Self::StoneChunk),
            "iron_ore" => Some(Self::IronOre),
            "copper_ore" => Some(Self::CopperOre),
            "carrot_seed" => Some(Self::CarrotSeed),
            "carrot" => Some(Self::Carrot),
            "wood_plank" => Some(Self::WoodPlank),
            "wood_bucket" => Some(Self::WoodBucket),
            "wood_door_item" => Some(Self::WoodDoorItem),
            "wood_crate" => Some(Self::WoodCrate),
            _ => None,
        }
    }
}

/// Recurso suelto en una casilla del mundo 2D.
#[derive(Clone, Debug)]
pub struct GroundItem {
    pub id: i32,
    pub kind: ItemKind,
    pub x: i32,
    pub y: i32,
    pub amount: i32,
    pub stored: bool,
}

/// Zonas designadas por el jugador en el mapa.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ZoneKind {
    None = 0,
    Storage = 1,
    Farm = 2,
}

/// Estructuras y planos modulares sobre casillas.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StructureKind {
    None = 0,
    WoodWallBlueprint = 1,
    WoodWall = 2,
    WoodDoorBlueprint = 3,
    WoodDoor = 4,
    CarpenterBenchBlueprint = 5,
    CarpenterBench = 6,
}

/// Recetas disponibles en la Mesa de Carpintero, ordenadas por complejidad (Inteligencia mínima).
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CarpentryRecipe {
    Planks = 1,
    Bucket = 2,
    Door = 3,
    Crate = 4,
}

impl CarpentryRecipe {
    pub const ALL: [Self; 4] = [Self::Planks, Self::Bucket, Self::Door, Self::Crate];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Planks => "planks",
            Self::Bucket => "bucket",
            Self::Door => "door",
            Self::Crate => "crate",
        }
    }

    pub const fn label_es(self) -> &'static str {
        match self {
            Self::Planks => "Tablones de madera",
            Self::Bucket => "Cubo de madera",
            Self::Door => "Puerta de madera",
            Self::Crate => "Arcón de madera",
        }
    }

    pub const fn description_es(self) -> &'static str {
        match self {
            Self::Planks => "Aserra troncos en tablones cepillados para carpintería.",
            Self::Bucket => "Recipiente de madera con aros metálicos (queda en el piso/almacén).",
            Self::Door => "Hoja de puerta lista para instalar en muros de madera.",
            Self::Crate => "Arcón reforzado de ebanistería compleja con herrajes.",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "planks" => Some(Self::Planks),
            "bucket" => Some(Self::Bucket),
            "door" => Some(Self::Door),
            "crate" => Some(Self::Crate),
            _ => None,
        }
    }

    /// Requisito mínimo de Inteligencia (`INT`) necesario para fabricar esta receta.
    pub const fn min_intelligence(self) -> i32 {
        match self {
            Self::Planks => 1,
            Self::Bucket => 2,
            Self::Door => 3,
            Self::Crate => 4,
        }
    }

    /// Ingrediente necesario: `(ItemKind, cantidad)`.
    pub const fn input_cost(self) -> (ItemKind, i32) {
        match self {
            Self::Planks => (ItemKind::WoodLogs, 2),
            Self::Bucket => (ItemKind::WoodPlank, 3),
            Self::Door => (ItemKind::WoodPlank, 4),
            Self::Crate => (ItemKind::WoodPlank, 6),
        }
    }

    /// Producto generado: `(ItemKind, cantidad)`.
    pub const fn output_item(self) -> (ItemKind, i32) {
        match self {
            Self::Planks => (ItemKind::WoodPlank, 4),
            Self::Bucket => (ItemKind::WoodBucket, 1),
            Self::Door => (ItemKind::WoodDoorItem, 1),
            Self::Crate => (ItemKind::WoodCrate, 1),
        }
    }

    /// Duración base en segundos de trabajo en la mesa.
    pub const fn work_duration(self) -> f32 {
        match self {
            Self::Planks => 1.8,
            Self::Bucket => 2.4,
            Self::Door => 2.8,
            Self::Crate => 3.3,
        }
    }
}

/// Estado persistente de una Mesa de Carpintero construida en el mundo.
#[derive(Clone, Debug)]
pub struct CarpentryBenchState {
    pub id: i32,
    pub x: i32,
    pub y: i32,
    pub queue: Vec<CarpentryRecipe>,
}

/// Etapas visuales y lógicas del cultivo de zanahorias.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CropStage {
    None = 0,
    Tilled = 1,
    Seeded = 2,
    Sprout = 3,
    Growing = 4,
    Ready = 5,
}

/// Celda de cultivo activa dentro de una zona de cultivo.
#[derive(Clone, Debug)]
pub struct CropCell {
    pub x: i32,
    pub y: i32,
    pub stage: CropStage,
    pub progress: f32,
    pub tend_cooldown: f32,
}

/// Tipo de orden solicitada por el jugador para que el líder la transmita.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrderKind {
    ChopTree = 1,
    MineRock = 2,
    BuildWoodWall = 3,
    FarmCell = 4,
    HaulItem = 5,
    BuildWoodDoor = 6,
    BuildCarpenterBench = 7,
    CarpentryJob = 8,
}

/// Estado de aprobación de una orden respecto al Líder de Colonia.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrderStatus {
    /// El jugador marcó la zona, pero el líder aún debe reunirse y dar la orden.
    PendingBriefing = 0,
    /// El líder ya dio la instrucción y cualquier colono puede ejecutarla.
    Approved = 1,
}

/// Orden individual sobre una casilla u objetivo.
#[derive(Clone, Debug)]
pub struct ColonyOrder {
    pub id: i32,
    pub kind: OrderKind,
    pub x: i32,
    pub y: i32,
    pub status: OrderStatus,
    pub assigned_colonist_id: i32,
}

/// Actividad actual de alto nivel de un colono.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ColonistActivity {
    Resting,
    Wandering,
    Scouting,
    Sleeping,
    ConveningMeeting,
    AttendingMeeting,
    MovingToTask,
    Chopping,
    Mining,
    Building,
    Carpentry,
    Farming,
    Botanizing,
    Hauling,
}

impl ColonistActivity {
    pub fn label_es(self) -> &'static str {
        match self {
            Self::Resting => "Observando el claro",
            Self::Wandering => "Recorriendo la colonia",
            Self::Scouting => "Explorando e inspeccionando",
            Self::Sleeping => "Durmiendo en el suelo",
            Self::ConveningMeeting => "Reuniendo colonos para dar órdenes",
            Self::AttendingMeeting => "Escuchando instrucciones del líder",
            Self::MovingToTask => "En camino a su tarea",
            Self::Chopping => "Talando madera con hacha de cobre",
            Self::Mining => "Picando roca con pico de cobre",
            Self::Building => "Construyendo estructura de madera",
            Self::Carpentry => "Fabricando piezas en mesa de carpintero",
            Self::Farming => "Labrando o cosechando zanahorias",
            Self::Botanizing => "Sembrando y cuidando cultivos",
            Self::Hauling => "Transportando recursos al almacén",
        }
    }
}

/// Tarea activa asignada al colono tras la reunión con el líder.
#[derive(Clone, Debug)]
pub struct ActiveTask {
    pub order_id: i32,
    pub kind: OrderKind,
    pub target_x: i32,
    pub target_y: i32,
    pub item_id: i32,
    pub work_timer: f32,
    pub phase: i32, // 0 = ir al objetivo/recoger, 1 = ejecutar/entregar, o índice de receta en carpintería
}

/// Inventario personal y ranuras de equipamiento del colono:
/// - `helmet_slot`: ranura reservada para futuros cascos.
/// - `armor_slot`: ranura reservada para futuras armaduras.
/// - `back_tool`: herramienta o arma llevada en la espalda (pico, azada, espada, etc.).
/// - `main_hand`: herramienta o arma empuñada actualmente en la mano principal.
#[derive(Clone, Debug)]
pub struct ColonistInventory {
    pub helmet_slot: String,
    pub armor_slot: String,
    pub back_tool: CopperTool,
    pub main_hand: CopperTool,
}

impl ColonistInventory {
    pub fn default_for_role(role: &str, is_leader: bool) -> Self {
        let main_hand = CopperTool::default_for_role(role, is_leader);
        let back_tool = if is_leader {
            CopperTool::CopperAxe
        } else {
            match role {
                "Leñador" | "Carpintero" => CopperTool::CopperShovel,
                "Minero" => CopperTool::CopperShovel,
                "Constructor" => CopperTool::CopperAxe,
                "Botánico" => CopperTool::CopperShovel,
                "Explorador" => CopperTool::CopperPickaxe,
                _ => CopperTool::CopperHoe,
            }
        };
        Self {
            helmet_slot: String::new(),
            armor_slot: String::new(),
            back_tool,
            main_hand,
        }
    }

    /// Equipa una herramienta en la mano principal intercambiándola con la espalda si hace falta.
    pub fn equip_main_hand(&mut self, tool: CopperTool) {
        if self.main_hand == tool {
            return;
        }
        if self.back_tool == tool && self.main_hand != CopperTool::None {
            self.back_tool = self.main_hand;
            self.main_hand = tool;
            return;
        }
        if self.main_hand != CopperTool::None && self.back_tool == CopperTool::None {
            self.back_tool = self.main_hand;
        }
        self.main_hand = tool;
    }

    /// Guarda la herramienta de la mano principal en la espalda (p.ej. al dormir o cargar).
    pub fn stow_main_hand(&mut self) {
        if self.main_hand != CopperTool::None {
            if self.back_tool == CopperTool::None {
                self.back_tool = self.main_hand;
            }
            self.main_hand = CopperTool::None;
        }
    }
}

/// Estado completo de un colono en simulación.
#[derive(Clone, Debug)]
pub struct Colonist {
    pub id: i32,
    pub name: String,
    pub role: String,
    pub is_leader: bool,
    pub x: f32,
    pub y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub path: Vec<(i32, i32)>,
    pub facing_x: f32,
    pub facing_y: f32,
    pub speed: f32,
    pub health: i32,
    pub max_health: i32,
    pub mood: i32,
    pub energy: f32,
    pub wait_timer: f32,
    pub walk_phase: f32,
    pub work_swing_phase: f32,
    pub body_style: i32,
    pub hair_style: i32,
    pub skin_color: String,
    pub hair_color: String,
    pub outfit_color: String,
    pub activity: ColonistActivity,
    pub role_profile: ColonistRoleProfile,
    pub stats: ColonistStats,
    pub inventory: ColonistInventory,
    pub carrying_kind: Option<ItemKind>,
    pub carrying_amount: i32,
    pub active_task: Option<ActiveTask>,
}

/// Inventario acumulado y disponible en la colonia.
#[derive(Clone, Debug, Default)]
pub struct ColonyInventory {
    pub wood: i32,
    pub stone: i32,
    pub iron: i32,
    pub copper: i32,
    pub carrot_seeds: i32,
    pub carrots: i32,
    pub wood_planks: i32,
    pub wood_buckets: i32,
    pub wood_doors: i32,
    pub wood_crates: i32,
}
