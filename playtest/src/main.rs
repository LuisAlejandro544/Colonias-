//! Punto de entrada CLI del entrenador neuroevolutivo para Colonia Centinela.
//!
//! # Lógica del módulo
//! Ejecuta el bucle de evolución genética sobre `SimulationCore` (modo headless):
//! - Permite cargar opcionalmente un cerebro previo desde caché (`--load-brain`)
//!   para continuar mejorando generaciones futuras.
//! - Evalúa cada individuo en los 3 biomas terrestres del juego.
//! - Guarda al finalizar:
//!   - `best_brain.json`: pesos de la mejor red neuronal encontrada.
//!   - `playtest_report.json`: telemetría detallada por bioma.
//!   - `gemma_prompt_input.txt`: resumen compacto listo para Gemma 4 E2B.

mod evaluator;
mod neuro;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use evaluator::{build_gemma_input_text, build_report_json, evaluate_genome_suite};
use neuro::{evolve_population, BrainGenome, Rng64};

struct CliConfig {
    generations: usize,
    population: usize,
    days_per_eval: i32,
    seed: u64,
    load_brain: Option<PathBuf>,
    out_dir: PathBuf,
}

fn parse_args() -> CliConfig {
    let mut cfg = CliConfig {
        generations: 16,
        population: 20,
        days_per_eval: 4,
        seed: 202611,
        load_brain: None,
        out_dir: PathBuf::from("playtest_output"),
    };

    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--generations" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<usize>() {
                    cfg.generations = v.clamp(1, 200);
                }
                i += 2;
            }
            "--population" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<usize>() {
                    cfg.population = v.clamp(4, 200);
                }
                i += 2;
            }
            "--days" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<i32>() {
                    cfg.days_per_eval = v.clamp(1, 14);
                }
                i += 2;
            }
            "--seed" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<u64>() {
                    cfg.seed = v;
                }
                i += 2;
            }
            "--load-brain" if i + 1 < args.len() => {
                cfg.load_brain = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--out-dir" if i + 1 < args.len() => {
                cfg.out_dir = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }
    cfg
}

fn main() {
    let cfg = parse_args();
    let started = Instant::now();
    let mut rng = Rng64::new(cfg.seed);
    let mut next_id = 1u64;

    let mut population: Vec<BrainGenome> = Vec::with_capacity(cfg.population);

    if let Some(ref brain_path) = cfg.load_brain {
        if let Ok(raw) = fs::read_to_string(brain_path) {
            if let Some(mut loaded) = BrainGenome::from_json(&raw) {
                println!(
                    "[NeuroPlaytest] Cerebro cargado desde caché: {} (gen {}, fitness previo {:.2})",
                    brain_path.display(),
                    loaded.generation,
                    loaded.fitness
                );
                loaded.id = next_id;
                next_id += 1;
                population.push(loaded.clone());
                // Crear variantes mutadas del cerebro cacheado para explorar mejoras rápidas.
                while population.len() < cfg.population / 2 {
                    let id = next_id;
                    next_id += 1;
                    population.push(BrainGenome::reproduce(
                        &loaded,
                        &loaded,
                        id,
                        0,
                        0.22,
                        0.20,
                        &mut rng,
                    ));
                }
            } else {
                println!(
                    "[NeuroPlaytest] Aviso: {} no coincide con la arquitectura actual; iniciando desde cero.",
                    brain_path.display()
                );
            }
        }
    }

    while population.len() < cfg.population {
        let id = next_id;
        next_id += 1;
        population.push(BrainGenome::random(id, 0, &mut rng));
    }

    let mut best_overall: Option<BrainGenome> = None;
    let mut fitness_history: Vec<f32> = Vec::with_capacity(cfg.generations);

    for gen_idx in 0..cfg.generations {
        let gen_seed = cfg.seed.wrapping_add((gen_idx as u64) * 31);
        for genome in &mut population {
            let (fit, _) = evaluate_genome_suite(genome, gen_seed, cfg.days_per_eval);
            genome.fitness = fit;
        }

        population.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));
        let gen_best = &population[0];
        fitness_history.push(gen_best.fitness);

        println!(
            "[Gen {:>2}/{}] Mejor fitness: {:>7.2} (Genoma #{}) | Media top-4: {:>7.2}",
            gen_idx + 1,
            cfg.generations,
            gen_best.fitness,
            gen_best.id,
            population
                .iter()
                .take(4.min(population.len()))
                .map(|g| g.fitness)
                .sum::<f32>()
                / (4.min(population.len()) as f32)
        );

        let is_better = match &best_overall {
            Some(prev) => gen_best.fitness > prev.fitness,
            None => true,
        };
        if is_better {
            best_overall = Some(gen_best.clone());
        }

        if gen_idx + 1 < cfg.generations {
            let elite_count = (cfg.population / 5).max(2);
            population = evolve_population(
                population,
                gen_idx + 1,
                elite_count,
                0.20,
                0.28,
                &mut next_id,
                &mut rng,
            );
        }
    }

    let mut final_best = best_overall.expect("La población debe tener al menos un genoma");
    // Re-evaluación determinista final sobre las semillas fijas de diagnóstico.
    let (final_fit, episodes) = evaluate_genome_suite(&final_best, cfg.seed, cfg.days_per_eval);
    final_best.fitness = final_fit;

    fs::create_dir_all(&cfg.out_dir).expect("No se pudo crear el directorio de salida");

    let brain_file = cfg.out_dir.join("best_brain.json");
    let report_file = cfg.out_dir.join("playtest_report.json");
    let gemma_input_file = cfg.out_dir.join("gemma_prompt_input.txt");

    fs::write(&brain_file, final_best.to_json()).expect("Error guardando best_brain.json");
    fs::write(
        &report_file,
        build_report_json(
            cfg.generations,
            cfg.population,
            cfg.days_per_eval,
            &fitness_history,
            &final_best,
            &episodes,
        ),
    )
    .expect("Error guardando playtest_report.json");
    fs::write(
        &gemma_input_file,
        build_gemma_input_text(
            cfg.generations,
            cfg.population,
            cfg.days_per_eval,
            &fitness_history,
            &final_best,
            &episodes,
        ),
    )
    .expect("Error guardando gemma_prompt_input.txt");

    println!(
        "[NeuroPlaytest] Completado en {:.2}s. Mejor fitness final: {:.2}. Archivos en {}",
        started.elapsed().as_secs_f32(),
        final_best.fitness,
        cfg.out_dir.display()
    );
}
