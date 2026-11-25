//! Generador determinista de ruido 2D y números pseudoaleatorios para la Tierra procedural.
//!
//! # Lógica del módulo
//! Implementa un generador `SplitMix64` (independiente de la arquitectura de 32 o 64 bits)
//! y ruido fractal 2D (FBM) basado en interpolación quíntica. Permite crear elevación,
//! humedad y densidad mineral reproducibles a partir de cualquier semilla numérica.

/// Generador pseudoaleatorio determinista de 64 bits (SplitMix64).
#[derive(Clone, Debug)]
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    /// Inicializa el generador a partir de una semilla arbitraria.
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// Devuelve el siguiente entero pseudoaleatorio de 64 bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Devuelve un flotante uniforme en el rango `[0.0, 1.0)`.
    pub fn next_f32(&mut self) -> f32 {
        let bits = (self.next_u64() >> 40) as u32;
        (bits as f32) / ((1u32 << 24) as f32)
    }

    /// Devuelve un índice entero en `[0, upper)` (o `0` si `upper == 0`).
    pub fn next_usize(&mut self, upper: usize) -> usize {
        if upper == 0 {
            return 0;
        }
        (self.next_u64() as usize) % upper
    }

    /// Devuelve un entero en `[min, max]`.
    pub fn range_i32(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        let span = (max - min + 1) as usize;
        min + self.next_usize(span) as i32
    }
}

/// Mezcla determinista de coordenadas de grilla 2D y semilla en `[0.0, 1.0]`.
fn hash_2d(ix: i32, iy: i32, seed: u64) -> f32 {
    let mut h = seed
        ^ ((ix as i64 as u64).wrapping_mul(0x517C_C1B7_2722_0A95))
        ^ ((iy as i64 as u64).wrapping_mul(0x94D0_49BB_1331_11EB));
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    ((h >> 40) as f32) / ((1u32 << 24) as f32)
}

#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Ruido suave de valor 2D en `[0.0, 1.0]`.
pub fn value_noise_2d(x: f32, y: f32, seed: u64) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let sx = fade(x - x0 as f32);
    let sy = fade(y - y0 as f32);

    let n00 = hash_2d(x0, y0, seed);
    let n10 = hash_2d(x1, y0, seed);
    let n01 = hash_2d(x0, y1, seed);
    let n11 = hash_2d(x1, y1, seed);

    let ix0 = lerp(n00, n10, sx);
    let ix1 = lerp(n01, n11, sx);
    lerp(ix0, ix1, sy)
}

/// Ruido fractal (FBM) de múltiples octavas normalizado a `[0.0, 1.0]`.
pub fn fbm_2d(x: f32, y: f32, seed: u64, octaves: u8) -> f32 {
    let mut value = 0.0f32;
    let mut amplitude = 0.5f32;
    let mut frequency = 1.0f32;
    let mut norm = 0.0f32;

    for i in 0..octaves {
        let octave_seed = seed.wrapping_add((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        value += amplitude * value_noise_2d(x * frequency, y * frequency, octave_seed);
        norm += amplitude;
        amplitude *= 0.5;
        frequency *= 2.02;
    }

    if norm > 0.0 {
        (value / norm).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_is_deterministic() {
        let mut a = SeededRng::new(12345);
        let mut b = SeededRng::new(12345);
        for _ in 0..16 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn fbm_stays_within_unit_interval() {
        for x in 0..20 {
            for y in 0..20 {
                let n = fbm_2d(x as f32 * 0.13, y as f32 * 0.13, 99, 4);
                assert!((0.0..=1.0).contains(&n));
            }
        }
    }
}
