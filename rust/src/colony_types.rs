//! Tipos de datos puros para zonas, cultivos, construcciones, ítems y órdenes.
//!
//! # Lógica del módulo
//! Centraliza las estructuras de dominio de la colonia (recursos en el piso,
//! bloques de pared de madera, zonas de almacenamiento y cultivo de zanahorias,
//! órdenes pendientes de reunión con el líder e inventario) manteniendo tipos
//! deterministas compatibles con 32 y 64 bits.

use crate::roles::ColonistRoleProfile;
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
            Self::Building => "Construyendo pared de madera",
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
    pub phase: i32, // 0 = ir al objetivo/recoger, 1 = ejecutar/entregar
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
                "Leñador" => CopperTool::CopperShovel,
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
}
