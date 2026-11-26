//! Punto de entrada raíz de la extensión nativa en Rust (`colonia_core`) para Godot 4.7.
//!
//! # Lógica del módulo
//! Conecta Godot Engine 4.7 con los módulos independientes en Rust:
//! - `noise`: ruido procedural determinista multiplataforma (32 y 64 bits).
//! - `world_gen`: generación del terreno terrestre, árboles, arbolitos, rocas y piedras.
//! - `tools`: catálogo de herramientas de cobre (hacha, pico, pala, azada y espada).
//! - `roles`: sistema de 9 roles (máx. 5 por colono) y progresión de niveles 1 al 10.
//! - `stats`: estadísticas personales (INT, FUE, DES, RES, CAR) con requisitos de fabricación.
//! - `colony_types`: modelos puros de zonas, cultivos, construcciones, ítems y órdenes.
//! - `animals`: fauna terrestre por bioma (cerditos, pollos, ovejas y vacas), carnes y huesos.
//! - `pathfinding`: búsqueda de rutas A* sobre el mapa dinámico.
//! - `colonist_sim`: simulación de colonos, líder de colonia, reuniones y persistencia.

mod animals;
mod colonist_sim;
mod colony_types;
mod noise;
mod pathfinding;
mod roles;
mod stats;
mod tools;
mod world_gen;

use godot::classes::{INode, Node};
use godot::prelude::*;

pub use colonist_sim::ColonySimulation;
pub use world_gen::WorldGenerator;

/// Estructura principal que implementa el ciclo de vida de la librería GDExtension.
struct ColoniaExtension;

#[gdextension]
unsafe impl ExtensionLibrary for ColoniaExtension {}

/// Nodo estructural para validar que Rust responde dentro de Godot 4.7.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct ColoniaBootstrap {
    base: Base<Node>,
}

#[godot_api]
impl INode for ColoniaBootstrap {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        godot_print!("colonia_core (Rust 1.100 / GDExtension API 4.7) inicializado.");
    }
}

#[godot_api]
impl ColoniaBootstrap {
    /// Devuelve la versión del núcleo nativo para verificación desde GDScript.
    #[func]
    pub fn get_core_version(&self) -> GString {
        GString::from("0.2.0-rust1.100-godot4.7")
    }
}
