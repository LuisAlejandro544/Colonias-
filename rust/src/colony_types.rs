//! Tipos de datos puros para zonas, cultivos, construcciones, ítems y órdenes.
//!
//! # Lógica del módulo
//! Centraliza las estructuras de dominio de la colonia (recursos en el piso,
//! bloques de pared de madera y piedra/ladrillo, suelos pavimentados, mesas de
//! carpintero/masón, hornos de piedra y ladrillo, braseros, pozos de agua,
//! riego agrícola, órdenes pendientes de reunión y colonos) manteniendo tipos
//! deterministas compatibles con 32 y 64 bits.

use crate::roles::ColonistRoleProfile;
use crate::stats::ColonistStats;
use crate::tools::CopperTool;

/// Especies de cultivos agrícolas soportados por la colonia.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CropKind {
    Carrot = 1,
    Potato = 2,
    Wheat = 3,
    Beet = 4,
}

impl CropKind {
    pub const ALL: [Self; 4] = [Self::Carrot, Self::Potato, Self::Wheat, Self::Beet];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Carrot => "carrot",
            Self::Potato => "potato",
            Self::Wheat => "wheat",
            Self::Beet => "beet",
        }
    }

    pub const fn label_es(self) -> &'static str {
        match self {
            Self::Carrot => "Zanahoria",
            Self::Potato => "Papa",
            Self::Wheat => "Trigo",
            Self::Beet => "Remolacha",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "carrot" => Some(Self::Carrot),
            "potato" => Some(Self::Potato),
            "wheat" => Some(Self::Wheat),
            "beet" => Some(Self::Beet),
            _ => None,
        }
    }

    pub const fn seed_item(self) -> ItemKind {
        match self {
            Self::Carrot => ItemKind::CarrotSeed,
            Self::Potato => ItemKind::PotatoSeed,
            Self::Wheat => ItemKind::WheatSeed,
            Self::Beet => ItemKind::BeetSeed,
        }
    }

    pub const fn harvest_item(self) -> ItemKind {
        match self {
            Self::Carrot => ItemKind::Carrot,
            Self::Potato => ItemKind::Potato,
            Self::Wheat => ItemKind::Wheat,
            Self::Beet => ItemKind::Beet,
        }
    }
}

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
    Coal = 11,
    CopperIngot = 12,
    PotatoSeed = 13,
    Potato = 14,
    WheatSeed = 15,
    Wheat = 16,
    BeetSeed = 17,
    Beet = 18,
    RawChickenBreast = 19,
    RawChickenThigh = 20,
    RawPorkBelly = 21,
    RawPorkRibs = 22,
    RawMuttonShoulder = 23,
    RawMuttonLeg = 24,
    RawBeefSirloin = 25,
    RawBeefBrisket = 26,
    AnimalBones = 27,
    WoodCup = 28,
    StoneBricks = 29,
    StoneMortar = 30,
    StoneMillstone = 31,
    StoneStatuette = 32,
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
            Self::Coal => "coal",
            Self::CopperIngot => "copper_ingot",
            Self::PotatoSeed => "potato_seed",
            Self::Potato => "potato",
            Self::WheatSeed => "wheat_seed",
            Self::Wheat => "wheat",
            Self::BeetSeed => "beet_seed",
            Self::Beet => "beet",
            Self::RawChickenBreast => "raw_chicken_breast",
            Self::RawChickenThigh => "raw_chicken_thigh",
            Self::RawPorkBelly => "raw_pork_belly",
            Self::RawPorkRibs => "raw_pork_ribs",
            Self::RawMuttonShoulder => "raw_mutton_shoulder",
            Self::RawMuttonLeg => "raw_mutton_leg",
            Self::RawBeefSirloin => "raw_beef_sirloin",
            Self::RawBeefBrisket => "raw_beef_brisket",
            Self::AnimalBones => "animal_bones",
            Self::WoodCup => "wood_cup",
            Self::StoneBricks => "stone_bricks",
            Self::StoneMortar => "stone_mortar",
            Self::StoneMillstone => "stone_millstone",
            Self::StoneStatuette => "stone_statuette",
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
            Self::Coal => "Carbón mineral",
            Self::CopperIngot => "Lingote de cobre",
            Self::PotatoSeed => "Semillas de papa",
            Self::Potato => "Papas",
            Self::WheatSeed => "Semillas de trigo",
            Self::Wheat => "Trigo",
            Self::BeetSeed => "Semillas de remolacha",
            Self::Beet => "Remolacha",
            Self::RawChickenBreast => "Pechuga de pollo cruda",
            Self::RawChickenThigh => "Muslo de pollo crudo",
            Self::RawPorkBelly => "Panceta de cerdo cruda",
            Self::RawPorkRibs => "Costillar de cerdo crudo",
            Self::RawMuttonShoulder => "Paletilla de oveja cruda",
            Self::RawMuttonLeg => "Pierna de oveja cruda",
            Self::RawBeefSirloin => "Solomillo de res crudo",
            Self::RawBeefBrisket => "Pecho de res crudo",
            Self::AnimalBones => "Huesos de animal",
            Self::WoodCup => "Vaso de madera",
            Self::StoneBricks => "Ladrillos de piedra",
            Self::StoneMortar => "Mortero de piedra",
            Self::StoneMillstone => "Piedra de molino",
            Self::StoneStatuette => "Estatuilla de piedra",
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
            "coal" => Some(Self::Coal),
            "copper_ingot" => Some(Self::CopperIngot),
            "potato_seed" => Some(Self::PotatoSeed),
            "potato" => Some(Self::Potato),
            "wheat_seed" => Some(Self::WheatSeed),
            "wheat" => Some(Self::Wheat),
            "beet_seed" => Some(Self::BeetSeed),
            "beet" => Some(Self::Beet),
            "raw_chicken_breast" => Some(Self::RawChickenBreast),
            "raw_chicken_thigh" => Some(Self::RawChickenThigh),
            "raw_pork_belly" => Some(Self::RawPorkBelly),
            "raw_pork_ribs" => Some(Self::RawPorkRibs),
            "raw_mutton_shoulder" => Some(Self::RawMuttonShoulder),
            "raw_mutton_leg" => Some(Self::RawMuttonLeg),
            "raw_beef_sirloin" => Some(Self::RawBeefSirloin),
            "raw_beef_brisket" => Some(Self::RawBeefBrisket),
            "animal_bones" => Some(Self::AnimalBones),
            "wood_cup" => Some(Self::WoodCup),
            "stone_bricks" => Some(Self::StoneBricks),
            "stone_mortar" => Some(Self::StoneMortar),
            "stone_millstone" => Some(Self::StoneMillstone),
            "stone_statuette" => Some(Self::StoneStatuette),
            _ => None,
        }
    }

    pub const fn is_meat_cut(self) -> bool {
        matches!(
            self,
            Self::RawChickenBreast
                | Self::RawChickenThigh
                | Self::RawPorkBelly
                | Self::RawPorkRibs
                | Self::RawMuttonShoulder
                | Self::RawMuttonLeg
                | Self::RawBeefSirloin
                | Self::RawBeefBrisket
        )
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
    WaterWellBlueprint = 7,
    WaterWell = 8,
    MasonBenchBlueprint = 9,
    MasonBench = 10,
    StoneFurnaceBlueprint = 11,
    StoneFurnace = 12,
    StoneWallBlueprint = 13,
    StoneWall = 14,
    StoneFloorBlueprint = 15,
    StoneFloor = 16,
    StoneBrazierBlueprint = 17,
    StoneBrazier = 18,
}

/// Recetas disponibles en la Mesa de Carpintero, ordenadas por complejidad (Inteligencia mínima).
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CarpentryRecipe {
    Planks = 1,
    Bucket = 2,
    Door = 3,
    Crate = 4,
    Cup = 5,
}

impl CarpentryRecipe {
    pub const ALL: [Self; 5] = [Self::Planks, Self::Cup, Self::Bucket, Self::Door, Self::Crate];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Planks => "planks",
            Self::Cup => "cup",
            Self::Bucket => "bucket",
            Self::Door => "door",
            Self::Crate => "crate",
        }
    }

    pub const fn label_es(self) -> &'static str {
        match self {
            Self::Planks => "Tablones de madera",
            Self::Cup => "Vaso de madera",
            Self::Bucket => "Cubo de madera",
            Self::Door => "Puerta de madera",
            Self::Crate => "Arcón de madera",
        }
    }

    pub const fn description_es(self) -> &'static str {
        match self {
            Self::Planks => "Aserra troncos en tablones cepillados para carpintería.",
            Self::Cup => "Vaso tallado en madera para beber con comodidad del pozo o del río.",
            Self::Bucket => "Cubo con aros metálicos necesario para pozos y cocina futura (no sirve para beber agua cruda del río).",
            Self::Door => "Hoja de puerta lista para instalar en muros de madera.",
            Self::Crate => "Arcón reforzado de ebanistería compleja con herrajes.",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "planks" => Some(Self::Planks),
            "cup" => Some(Self::Cup),
            "bucket" => Some(Self::Bucket),
            "door" => Some(Self::Door),
            "crate" => Some(Self::Crate),
            _ => None,
        }
    }

    /// Requisito mínimo de Inteligencia (`INT`) necesario para fabricar esta receta.
    pub const fn min_intelligence(self) -> i32 {
        match self {
            Self::Planks | Self::Cup => 1,
            Self::Bucket => 2,
            Self::Door => 3,
            Self::Crate => 4,
        }
    }

    /// Ingrediente necesario: `(ItemKind, cantidad)`.
    pub const fn input_cost(self) -> (ItemKind, i32) {
        match self {
            Self::Planks => (ItemKind::WoodLogs, 2),
            Self::Cup => (ItemKind::WoodPlank, 1),
            Self::Bucket => (ItemKind::WoodPlank, 3),
            Self::Door => (ItemKind::WoodPlank, 4),
            Self::Crate => (ItemKind::WoodPlank, 6),
        }
    }

    /// Producto generado: `(ItemKind, cantidad)`.
    pub const fn output_item(self) -> (ItemKind, i32) {
        match self {
            Self::Planks => (ItemKind::WoodPlank, 4),
            Self::Cup => (ItemKind::WoodCup, 2),
            Self::Bucket => (ItemKind::WoodBucket, 1),
            Self::Door => (ItemKind::WoodDoorItem, 1),
            Self::Crate => (ItemKind::WoodCrate, 1),
        }
    }

    /// Duración base en segundos de trabajo en la mesa.
    pub const fn work_duration(self) -> f32 {
        match self {
            Self::Planks => 1.8,
            Self::Cup => 1.4,
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

/// Tipo de taller de cantería o fundición operado por el Masón.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WorkshopKind {
    MasonBench = 1,
    StoneFurnace = 2,
}

impl WorkshopKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::MasonBench => "mason",
            Self::StoneFurnace => "furnace",
        }
    }

    pub const fn label_es(self) -> &'static str {
        match self {
            Self::MasonBench => "Mesa de Masón",
            Self::StoneFurnace => "Horno de Piedra y Ladrillo",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "mason" => Some(Self::MasonBench),
            "furnace" => Some(Self::StoneFurnace),
            _ => None,
        }
    }
}

/// Recetas de cantería y fundición trabajadas por el Masón.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MasonryRecipe {
    Bricks = 1,
    Mortar = 2,
    Millstone = 3,
    Statuette = 4,
    CopperIngot = 5,
}

impl MasonryRecipe {
    pub const ALL: [Self; 5] = [
        Self::Bricks,
        Self::Mortar,
        Self::Millstone,
        Self::Statuette,
        Self::CopperIngot,
    ];

    pub const fn station(self) -> WorkshopKind {
        match self {
            Self::CopperIngot => WorkshopKind::StoneFurnace,
            _ => WorkshopKind::MasonBench,
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Bricks => "bricks",
            Self::Mortar => "mortar",
            Self::Millstone => "millstone",
            Self::Statuette => "statuette",
            Self::CopperIngot => "copper_ingot",
        }
    }

    pub const fn label_es(self) -> &'static str {
        match self {
            Self::Bricks => "Ladrillos de piedra",
            Self::Mortar => "Mortero de piedra",
            Self::Millstone => "Piedra de molino",
            Self::Statuette => "Estatuilla de piedra",
            Self::CopperIngot => "Lingote de cobre",
        }
    }

    pub const fn description_es(self) -> &'static str {
        match self {
            Self::Bricks => "Corta bloques de piedra en ladrillos resistentes para muros, suelos y hornos.",
            Self::Mortar => "Cuenco pesado con majadero tallado en granito para moler grano y hierbas.",
            Self::Millstone => "Rueda tallada en piedra y ladrillo para molienda avanzada de la colonia.",
            Self::Statuette => "Figura decorativa del Centinela esculpida con ladrillo y piedra pulida.",
            Self::CopperIngot => "Funde mena de cobre con carbón mineral a alta temperatura en el horno.",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "bricks" => Some(Self::Bricks),
            "mortar" => Some(Self::Mortar),
            "millstone" => Some(Self::Millstone),
            "statuette" => Some(Self::Statuette),
            "copper_ingot" => Some(Self::CopperIngot),
            _ => None,
        }
    }

    pub const fn min_intelligence(self) -> i32 {
        match self {
            Self::Bricks => 1,
            Self::Mortar | Self::CopperIngot => 2,
            Self::Millstone => 3,
            Self::Statuette => 4,
        }
    }

    pub const fn input_cost_1(self) -> (ItemKind, i32) {
        match self {
            Self::Bricks => (ItemKind::StoneChunk, 2),
            Self::Mortar => (ItemKind::StoneChunk, 3),
            Self::Millstone => (ItemKind::StoneChunk, 4),
            Self::Statuette => (ItemKind::StoneBricks, 4),
            Self::CopperIngot => (ItemKind::CopperOre, 2),
        }
    }

    pub const fn input_cost_2(self) -> Option<(ItemKind, i32)> {
        match self {
            Self::Millstone => Some((ItemKind::StoneBricks, 2)),
            Self::Statuette => Some((ItemKind::StoneChunk, 1)),
            Self::CopperIngot => Some((ItemKind::Coal, 1)),
            _ => None,
        }
    }

    pub const fn output_item(self) -> (ItemKind, i32) {
        match self {
            Self::Bricks => (ItemKind::StoneBricks, 4),
            Self::Mortar => (ItemKind::StoneMortar, 1),
            Self::Millstone => (ItemKind::StoneMillstone, 1),
            Self::Statuette => (ItemKind::StoneStatuette, 1),
            Self::CopperIngot => (ItemKind::CopperIngot, 1),
        }
    }

    pub const fn work_duration(self) -> f32 {
        match self {
            Self::Bricks => 1.8,
            Self::Mortar => 2.3,
            Self::CopperIngot => 2.5,
            Self::Millstone => 2.9,
            Self::Statuette => 3.2,
        }
    }
}

/// Estado persistente de una Mesa de Masón o un Horno de Piedra y Ladrillo.
#[derive(Clone, Debug)]
pub struct WorkshopStationState {
    pub id: i32,
    pub kind: WorkshopKind,
    pub x: i32,
    pub y: i32,
    pub queue: Vec<MasonryRecipe>,
}

/// Etapas visuales y lógicas del cultivo agrícola.
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
    pub crop_kind: CropKind,
    pub stage: CropStage,
    pub progress: f32,
    pub tend_cooldown: f32,
    pub watered_timer: f32,
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
    HuntAnimal = 9,
    BuildWaterWell = 10,
    DrinkWater = 11,
    BuildMasonBench = 12,
    BuildStoneFurnace = 13,
    BuildStoneWall = 14,
    BuildStoneFloor = 15,
    BuildStoneBrazier = 16,
    MasonryJob = 17,
    WaterCrops = 18,
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
    pub target_id: i32,
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
    Masonry,
    Smelting,
    Farming,
    Botanizing,
    WateringCrops,
    Hauling,
    Hunting,
    DrinkingWell,
    DrinkingRiver,
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
            Self::Mining => "Picando roca y vetas minerales",
            Self::Building => "Construyendo estructura",
            Self::Carpentry => "Fabricando piezas en mesa de carpintero",
            Self::Masonry => "Labrando piedra en mesa de masón",
            Self::Smelting => "Fundiendo mineral en el horno",
            Self::Farming => "Labrando o cosechando cultivos",
            Self::Botanizing => "Sembrando y cuidando cultivos",
            Self::WateringCrops => "Regando cultivos con cubo de agua",
            Self::Hauling => "Transportando recursos al almacén",
            Self::Hunting => "Cazando y obteniendo cortes de carne",
            Self::DrinkingWell => "Bebiendo agua del pozo",
            Self::DrinkingRiver => "Bebiendo agua cruda del río",
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
                "Minero" | "Masón" => CopperTool::CopperShovel,
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
    pub hydration: f32,
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
    pub coal: i32,
    pub copper_ingots: i32,
    pub carrot_seeds: i32,
    pub carrots: i32,
    pub potato_seeds: i32,
    pub potatoes: i32,
    pub wheat_seeds: i32,
    pub wheat: i32,
    pub beet_seeds: i32,
    pub beets: i32,
    pub meat_cuts: i32,
    pub animal_bones: i32,
    pub wood_planks: i32,
    pub wood_buckets: i32,
    pub wood_cups: i32,
    pub wood_doors: i32,
    pub wood_crates: i32,
    pub stone_bricks: i32,
    pub stone_mortars: i32,
    pub stone_millstones: i32,
    pub stone_statuettes: i32,
}
