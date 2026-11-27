//! Motor de neuroevolución ligero en Rust puro (sin dependencias externas).
//!
//! # Lógica del módulo
//! Implementa un perceptrón multicapa evolutivo (`BrainGenome`: 24 entradas ->
//! 20 neuronas ocultas `tanh` -> 16 acciones macro) junto a operadores genéticos
//! deterministas (`SplitMix64`):
//! - Inicialización tipo Xavier escalada para evitar saturación inicial.
//! - Evaluación hacia adelante (`forward`) y selección de acción (`select_action`).
//! - Selección por torneo, cruce uniforme y mutación gaussiana aproximada.
//! - Serialización y carga en JSON propio para guardar y reutilizar `best_brain.json`
//!   en caché entre ejecuciones manuales de GitHub Actions.

pub const INPUT_COUNT: usize = 24;
pub const HIDDEN_COUNT: usize = 20;
pub const OUTPUT_COUNT: usize = 16;

/// Generador pseudoaleatorio determinista SplitMix64 (compatible 32/64 bits).
#[derive(Clone, Debug)]
pub struct Rng64 {
    state: u64,
}

impl Rng64 {
    pub fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn next_f32(&mut self) -> f32 {
        let bits = (self.next_u64() >> 40) as u32;
        (bits as f32) / ((1u32 << 24) as f32)
    }

    pub fn next_usize(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            return 0;
        }
        (self.next_u64() % (upper as u64)) as usize
    }

    /// Aproximación rápida de distribución normal N(0, 1) mediante suma de 6 uniformes.
    pub fn next_normal_approx(&mut self) -> f32 {
        let mut sum = 0.0f32;
        for _ in 0..6 {
            sum += self.next_f32();
        }
        (sum - 3.0) * 1.414_213_5
    }
}

/// Genoma neuronal que controla las decisiones de alto nivel de la colonia.
#[derive(Clone, Debug)]
pub struct BrainGenome {
    pub id: u64,
    pub generation: usize,
    pub fitness: f32,
    pub w1: Vec<f32>,
    pub b1: Vec<f32>,
    pub w2: Vec<f32>,
    pub b2: Vec<f32>,
}

impl BrainGenome {
    pub fn random(id: u64, generation: usize, rng: &mut Rng64) -> Self {
        let scale1 = (2.0 / (INPUT_COUNT + HIDDEN_COUNT) as f32).sqrt();
        let scale2 = (2.0 / (HIDDEN_COUNT + OUTPUT_COUNT) as f32).sqrt();

        let w1 = (0..INPUT_COUNT * HIDDEN_COUNT)
            .map(|_| rng.next_normal_approx() * scale1)
            .collect();
        let b1 = (0..HIDDEN_COUNT)
            .map(|_| rng.next_normal_approx() * 0.1)
            .collect();
        let w2 = (0..HIDDEN_COUNT * OUTPUT_COUNT)
            .map(|_| rng.next_normal_approx() * scale2)
            .collect();
        let b2 = (0..OUTPUT_COUNT)
            .map(|_| rng.next_normal_approx() * 0.1)
            .collect();

        Self {
            id,
            generation,
            fitness: f32::NEG_INFINITY,
            w1,
            b1,
            w2,
            b2,
        }
    }

    /// Calcula las puntuaciones de salida para las 16 acciones a partir de las 24 entradas.
    pub fn forward(&self, inputs: &[f32; INPUT_COUNT]) -> [f32; OUTPUT_COUNT] {
        let mut hidden = [0.0f32; HIDDEN_COUNT];
        for h in 0..HIDDEN_COUNT {
            let mut acc = self.b1[h];
            let row_offset = h * INPUT_COUNT;
            for i in 0..INPUT_COUNT {
                acc += inputs[i] * self.w1[row_offset + i];
            }
            hidden[h] = acc.tanh();
        }

        let mut outputs = [0.0f32; OUTPUT_COUNT];
        for o in 0..OUTPUT_COUNT {
            let mut acc = self.b2[o];
            let row_offset = o * HIDDEN_COUNT;
            for h in 0..HIDDEN_COUNT {
                acc += hidden[h] * self.w2[row_offset + h];
            }
            outputs[o] = acc;
        }
        outputs
    }

    /// Devuelve el índice de acción con mayor puntuación.
    pub fn select_action(&self, inputs: &[f32; INPUT_COUNT]) -> usize {
        let outputs = self.forward(inputs);
        let mut best_idx = 0usize;
        let mut best_val = outputs[0];
        for (idx, &val) in outputs.iter().enumerate().skip(1) {
            if val > best_val {
                best_val = val;
                best_idx = idx;
            }
        }
        best_idx
    }

    /// Cruza dos genomas y aplica mutación gaussiana al descendiente.
    pub fn reproduce(
        parent_a: &Self,
        parent_b: &Self,
        child_id: u64,
        generation: usize,
        mutation_rate: f32,
        mutation_sigma: f32,
        rng: &mut Rng64,
    ) -> Self {
        let mut w1 = Vec::with_capacity(parent_a.w1.len());
        for i in 0..parent_a.w1.len() {
            let mut v = if rng.next_f32() < 0.5 {
                parent_a.w1[i]
            } else {
                parent_b.w1[i]
            };
            if rng.next_f32() < mutation_rate {
                v = (v + rng.next_normal_approx() * mutation_sigma).clamp(-4.0, 4.0);
            }
            w1.push(v);
        }

        let mut b1 = Vec::with_capacity(parent_a.b1.len());
        for i in 0..parent_a.b1.len() {
            let mut v = if rng.next_f32() < 0.5 {
                parent_a.b1[i]
            } else {
                parent_b.b1[i]
            };
            if rng.next_f32() < mutation_rate {
                v = (v + rng.next_normal_approx() * mutation_sigma).clamp(-4.0, 4.0);
            }
            b1.push(v);
        }

        let mut w2 = Vec::with_capacity(parent_a.w2.len());
        for i in 0..parent_a.w2.len() {
            let mut v = if rng.next_f32() < 0.5 {
                parent_a.w2[i]
            } else {
                parent_b.w2[i]
            };
            if rng.next_f32() < mutation_rate {
                v = (v + rng.next_normal_approx() * mutation_sigma).clamp(-4.0, 4.0);
            }
            w2.push(v);
        }

        let mut b2 = Vec::with_capacity(parent_a.b2.len());
        for i in 0..parent_a.b2.len() {
            let mut v = if rng.next_f32() < 0.5 {
                parent_a.b2[i]
            } else {
                parent_b.b2[i]
            };
            if rng.next_f32() < mutation_rate {
                v = (v + rng.next_normal_approx() * mutation_sigma).clamp(-4.0, 4.0);
            }
            b2.push(v);
        }

        Self {
            id: child_id,
            generation,
            fitness: f32::NEG_INFINITY,
            w1,
            b1,
            w2,
            b2,
        }
    }

    /// Serializa el genoma completo a JSON legible para persistencia en caché/artefactos.
    pub fn to_json(&self) -> String {
        fn fmt_slice(vals: &[f32]) -> String {
            let parts: Vec<String> = vals.iter().map(|v| format!("{v:.6}")).collect();
            format!("[{}]", parts.join(","))
        }
        format!(
            "{{\n  \"id\": {},\n  \"generation\": {},\n  \"fitness\": {:.4},\n  \"input_count\": {},\n  \"hidden_count\": {},\n  \"output_count\": {},\n  \"w1\": {},\n  \"b1\": {},\n  \"w2\": {},\n  \"b2\": {}\n}}\n",
            self.id,
            self.generation,
            if self.fitness.is_finite() {
                self.fitness
            } else {
                0.0
            },
            INPUT_COUNT,
            HIDDEN_COUNT,
            OUTPUT_COUNT,
            fmt_slice(&self.w1),
            fmt_slice(&self.b1),
            fmt_slice(&self.w2),
            fmt_slice(&self.b2),
        )
    }

    /// Carga un genoma previamente guardado en JSON validando dimensiones de la red.
    pub fn from_json(raw: &str) -> Option<Self> {
        let id = extract_u64(raw, "\"id\"").unwrap_or(1);
        let generation = extract_u64(raw, "\"generation\"").unwrap_or(0) as usize;
        let fitness = extract_f32(raw, "\"fitness\"").unwrap_or(0.0);
        let in_c = extract_u64(raw, "\"input_count\"")? as usize;
        let hid_c = extract_u64(raw, "\"hidden_count\"")? as usize;
        let out_c = extract_u64(raw, "\"output_count\"")? as usize;
        if in_c != INPUT_COUNT || hid_c != HIDDEN_COUNT || out_c != OUTPUT_COUNT {
            return None;
        }
        let w1 = extract_f32_array(raw, "\"w1\"")?;
        let b1 = extract_f32_array(raw, "\"b1\"")?;
        let w2 = extract_f32_array(raw, "\"w2\"")?;
        let b2 = extract_f32_array(raw, "\"b2\"")?;
        if w1.len() != INPUT_COUNT * HIDDEN_COUNT
            || b1.len() != HIDDEN_COUNT
            || w2.len() != HIDDEN_COUNT * OUTPUT_COUNT
            || b2.len() != OUTPUT_COUNT
        {
            return None;
        }
        Some(Self {
            id,
            generation,
            fitness,
            w1,
            b1,
            w2,
            b2,
        })
    }
}

fn extract_u64(raw: &str, key: &str) -> Option<u64> {
    let pos = raw.find(key)?;
    let rest = &raw[pos + key.len()..];
    let colon = rest.find(':')?;
    let after = rest[colon + 1..].trim_start();
    let end = after
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after.len());
    after[..end].parse::<u64>().ok()
}

fn extract_f32(raw: &str, key: &str) -> Option<f32> {
    let pos = raw.find(key)?;
    let rest = &raw[pos + key.len()..];
    let colon = rest.find(':')?;
    let after = rest[colon + 1..].trim_start();
    let end = after
        .find(|c: char| !(c.is_ascii_digit() || c == '-' || c == '.' || c == 'e' || c == 'E' || c == '+'))
        .unwrap_or(after.len());
    after[..end].parse::<f32>().ok()
}

fn extract_f32_array(raw: &str, key: &str) -> Option<Vec<f32>> {
    let pos = raw.find(key)?;
    let rest = &raw[pos + key.len()..];
    let open = rest.find('[')?;
    let close = rest[open + 1..].find(']')?;
    let inner = &rest[open + 1..open + 1 + close];
    let mut out = Vec::new();
    for token in inner.split(',') {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        out.push(t.parse::<f32>().ok()?);
    }
    Some(out)
}

/// Selecciona un padre por torneo entre `k` candidatos aleatorios.
pub fn tournament_select<'a>(
    population: &'a [BrainGenome],
    k: usize,
    rng: &mut Rng64,
) -> &'a BrainGenome {
    let mut best = &population[rng.next_usize(population.len())];
    for _ in 1..k.max(1) {
        let cand = &population[rng.next_usize(population.len())];
        if cand.fitness > best.fitness {
            best = cand;
        }
    }
    best
}

/// Genera la siguiente población manteniendo élite y reproduciendo el resto.
pub fn evolve_population(
    mut population: Vec<BrainGenome>,
    next_generation: usize,
    elite_count: usize,
    mutation_rate: f32,
    mutation_sigma: f32,
    next_id_counter: &mut u64,
    rng: &mut Rng64,
) -> Vec<BrainGenome> {
    population.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));
    let pop_size = population.len();
    let elites = elite_count.clamp(1, pop_size);
    let mut next_pop = Vec::with_capacity(pop_size);

    for elite in population.iter().take(elites) {
        let mut copy = elite.clone();
        copy.generation = next_generation;
        next_pop.push(copy);
    }

    let pool_limit = (pop_size / 2).max(elites).min(pop_size);
    let breeding_pool = &population[..pool_limit];

    while next_pop.len() < pop_size {
        let parent_a = tournament_select(breeding_pool, 3, rng);
        let parent_b = tournament_select(breeding_pool, 3, rng);
        let id = *next_id_counter;
        *next_id_counter += 1;
        next_pop.push(BrainGenome::reproduce(
            parent_a,
            parent_b,
            id,
            next_generation,
            mutation_rate,
            mutation_sigma,
            rng,
        ));
    }

    next_pop
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genome_forward_and_json_roundtrip_preserve_weights() {
        let mut rng = Rng64::new(12345);
        let mut brain = BrainGenome::random(7, 3, &mut rng);
        brain.fitness = 142.5;
        let inputs = [0.25f32; INPUT_COUNT];
        let out_before = brain.forward(&inputs);

        let json = brain.to_json();
        let restored = BrainGenome::from_json(&json).expect("JSON válido");
        assert_eq!(restored.id, 7);
        assert_eq!(restored.generation, 3);
        assert!((restored.fitness - 142.5).abs() < 0.01);

        let out_after = restored.forward(&inputs);
        for (a, b) in out_before.iter().zip(out_after.iter()) {
            assert!((a - b).abs() < 0.001);
        }
    }

    #[test]
    fn evolve_population_preserves_size_and_best_elite() {
        let mut rng = Rng64::new(99);
        let mut pop: Vec<BrainGenome> = (0..8)
            .map(|i| {
                let mut g = BrainGenome::random(i + 1, 0, &mut rng);
                g.fitness = (i as f32) * 10.0;
                g
            })
            .collect();
        let mut next_id = 100;
        pop = evolve_population(pop, 1, 2, 0.15, 0.25, &mut next_id, &mut rng);
        assert_eq!(pop.len(), 8);
        assert_eq!(pop[0].id, 8);
        assert!((pop[0].fitness - 70.0).abs() < 0.001);
    }
}
