/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

/**
 * Componente raíz mínimo para que el entorno web compile y ejecute sin errores.
 *
 * Lógica: Renderiza únicamente "Hello World" en pantalla completa centrada,
 * dejando la arquitectura del juego móvil 2D (Godot 4.7.2 + Rust 1.100.0)
 * aislada en `/godot` y `/rust`.
 */
import { getHelloWorld } from './utils/greeting';

export default function App() {
  return (
    <main className="min-h-screen flex items-center justify-center bg-neutral-950 text-neutral-100">
      <h1 className="text-3xl font-semibold tracking-tight">{getHelloWorld()}</h1>
    </main>
  );
}
