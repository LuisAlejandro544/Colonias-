//! Catálogo de herramientas de cobre para Colonia Centinela.
//!
//! # Lógica del módulo
//! Define las 5 herramientas iniciales de cobre (Hacha, Pico, Pala, Azada y Espada),
//! sus identificadores de textura en Godot, velocidad de trabajo y asignación
//! automática según el rol o la tarea terrestre que ejecuta cada colono.

/// Herramientas de cobre disponibles en la colonia.
#[repr(i32)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CopperTool {
    None = 0,
    CopperAxe = 1,
    CopperPickaxe = 2,
    CopperShovel = 3,
    CopperHoe = 4,
    CopperSword = 5,
}

impl CopperTool {
    /// Código de texto estable para sincronización con Godot y guardado JSON.
    pub fn code(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::CopperAxe => "copper_axe",
            Self::CopperPickaxe => "copper_pickaxe",
            Self::CopperShovel => "copper_shovel",
            Self::CopperHoe => "copper_hoe",
            Self::CopperSword => "copper_sword",
        }
    }

    /// Nombre legible en español para el inspector táctil.
    pub fn label_es(self) -> &'static str {
        match self {
            Self::None => "Sin herramienta",
            Self::CopperAxe => "Hacha de cobre",
            Self::CopperPickaxe => "Pico de cobre",
            Self::CopperShovel => "Pala de cobre",
            Self::CopperHoe => "Azada de cobre",
            Self::CopperSword => "Espada de cobre",
        }
    }

    /// Reconstruye la variante desde su código guardado.
    pub fn from_code(code: &str) -> Self {
        match code {
            "copper_axe" => Self::CopperAxe,
            "copper_pickaxe" => Self::CopperPickaxe,
            "copper_shovel" => Self::CopperShovel,
            "copper_hoe" => Self::CopperHoe,
            "copper_sword" => Self::CopperSword,
            _ => Self::None,
        }
    }

    /// Selecciona la herramienta de cobre acorde al rol inicial del colono.
    pub fn default_for_role(role: &str, is_leader: bool) -> Self {
        if is_leader {
            return Self::CopperSword;
        }
        match role {
            "Leñador" => Self::CopperAxe,
            "Minero" => Self::CopperPickaxe,
            "Constructor" | "Transportador" => Self::CopperShovel,
            "Agricultor" | "Botánico" => Self::CopperHoe,
            "Explorador" | "Líder de Colonia" => Self::CopperSword,
            _ => Self::CopperShovel,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copper_tools_roundtrip_codes() {
        let all = [
            CopperTool::CopperAxe,
            CopperTool::CopperPickaxe,
            CopperTool::CopperShovel,
            CopperTool::CopperHoe,
            CopperTool::CopperSword,
        ];
        for tool in all {
            assert_eq!(CopperTool::from_code(tool.code()), tool);
            assert!(!tool.label_es().is_empty());
        }
    }
}
