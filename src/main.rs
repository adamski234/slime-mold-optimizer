mod batch;

use batch::BatchRunData;
use clap::{Args, Parser, Subcommand};
use rand::thread_rng;
use slimes::{functions::Functions, particles, slime};

const FN_SIZE: usize = 5;


#[derive(Debug, Parser, Clone)]
struct Config {
    #[command(flatten)]
    multi_swarm_config: Option<MultiSwarmConfig>,
    #[arg(long = "functions", value_delimiter = ',', num_args = 1.., required = true)]
    functions: Vec<String>,
    #[arg(long = "try-count")]
    try_count: Option<usize>,
    #[arg(long = "iterations")]
    iterations: usize,
    #[arg(long = "pop-size")]
    population_size: usize,

    #[command(subcommand)]
    variant: OptimizationVariant
}

#[derive(Debug, Args, Clone, Copy)]
struct MultiSwarmConfig {
    #[arg(long = "migration-threshold", name = "migration-threshold", required = false)]
    migration_threshold: f64,
    #[arg(long = "swarm-count", name = "swarm-count", required = false)]
    swarm_count: usize,
}

#[derive(Debug, Clone, Subcommand, Copy)]
enum OptimizationVariant {
    Particles(ParticleConfig),

    Slime {
        #[arg(long = "z-parameter")]
        z_param: f64,
    }
}

#[derive(Debug, Args, Clone, Copy)]
struct ParticleConfig {
    #[arg(long = "social-coeff")]
    social_coeff: f64,
    #[arg(long = "cognitive-coeff")]
    cognitive_coeff: f64,
    #[arg(long = "inertia-coeff")]
    inertia_coeff: f64,
}

fn main() {
    let config = Config::parse();

    if config.functions.is_empty() {
        panic!("No functions given");
    }
    let test_functions = config.functions.into_iter().map(|s| {
        return (Functions::<FN_SIZE>::make_from_name(&s), s);
    }).collect::<Vec<_>>();

    if let Some(tries) = config.try_count {
        for (function, function_name) in test_functions {
            let bounds = function.get_bounds();
            let tries_per_thread = tries.div_ceil(num_cpus::get());
            let mut threads = Vec::with_capacity(num_cpus::get());
            for _ in 0..num_cpus::get() {
                threads.push(std::thread::spawn(move || {
                    let mut thread_stats = BatchRunData::new();
                    if let Some(MultiSwarmConfig { migration_threshold, swarm_count }) = config.multi_swarm_config {
                        match config.variant {
                            OptimizationVariant::Particles(ParticleConfig { social_coeff, cognitive_coeff, inertia_coeff }) => {
                                let mut world = particles::MultiSwarmWorldState::new(swarm_count, migration_threshold, config.population_size, function, bounds, social_coeff, cognitive_coeff, inertia_coeff);
                                for _ in 0..tries_per_thread {
                                    world.do_all_iters(config.iterations);
                                    thread_stats += world.best_solution_value;
                                    world.reset();
                                }
                            }
                            OptimizationVariant::Slime { z_param } => {
                                let mut world = slime::MultiSwarmWorldState::new(swarm_count, migration_threshold, config.population_size, bounds, config.iterations, function, z_param);
                                for _ in 0..tries_per_thread {
                                    world.do_all_iters();
                                    thread_stats += world.best_solution_value;
                                    world.reset();
                                }
                            }
                        }
                    } else {
                        match config.variant {
                            OptimizationVariant::Particles(ParticleConfig { social_coeff, cognitive_coeff, inertia_coeff }) => {
                                let mut world = particles::WorldState::new(config.population_size, function, bounds, social_coeff, cognitive_coeff, inertia_coeff);
                                for _ in 0..tries_per_thread {
                                    world.do_all_iterations(config.iterations);
                                    thread_stats += world.best_solution_value;
                                    world.reset();
                                }
                            }
                            OptimizationVariant::Slime { z_param } => {
                                let mut world = slime::WorldState::new(config.population_size, bounds, config.iterations, function, z_param, thread_rng());
                                for _ in 0..tries_per_thread {
                                    world.do_all_iters();
                                    thread_stats += world.best_solution_value;
                                    world.reset();
                                }
                            }
                        }
                    }
                    return thread_stats;
                }));
            }
            let result = threads.into_iter().map(|handle| handle.join().unwrap()).reduce(|mut a, b| {
				a += b;
				return a;
			}).unwrap();
            println!("{}: Finished {} runs. Max solution is {}. Average solution is {}. Min solution is {}.", function_name, result.run_count, result.max_result, result.average, result.min_result);
        }
    } else {
        let mut threads = Vec::new();
		for (function, function_name) in test_functions {
            let bounds = function.get_bounds();
            threads.push(std::thread::spawn(move || {
                if let Some(MultiSwarmConfig { migration_threshold, swarm_count }) = config.multi_swarm_config {
                    match config.variant {
                        OptimizationVariant::Particles(ParticleConfig { social_coeff, cognitive_coeff, inertia_coeff }) => {
                            let mut world = particles::MultiSwarmWorldState::new(swarm_count, migration_threshold, config.population_size, function, bounds, social_coeff, cognitive_coeff, inertia_coeff);
                            world.do_all_iters(config.iterations);
                            println!("{}: Found optimum at {:?} = {}", function_name, world.best_solution.coordinates, world.best_solution_value);
                        }
                        OptimizationVariant::Slime { z_param } => {
                            let mut world = slime::MultiSwarmWorldState::new(swarm_count, migration_threshold, config.population_size, bounds, config.iterations, function, z_param);
                            world.do_all_iters();
                            println!("{}: Found optimum at {:?} = {}", function_name, world.best_solution.coordinates, world.best_solution_value);
                        }
                    }
                } else {
                    match config.variant {
                        OptimizationVariant::Particles(ParticleConfig { social_coeff, cognitive_coeff, inertia_coeff }) => {
                            let mut world = particles::WorldState::new(config.population_size, function, bounds, social_coeff, cognitive_coeff, inertia_coeff);
                            world.do_all_iterations(config.iterations);
                            println!("{}: Found optimum at {:?} = {}", function_name, world.best_solution.coordinates, world.best_solution_value);
                        }
                        OptimizationVariant::Slime { z_param } => {
                            let mut world = slime::WorldState::new(config.population_size, bounds, config.iterations, function, z_param, thread_rng());
                            world.do_all_iters();
                            println!("{}: Found optimum at {:?} = {}", function_name, world.best_solution.coordinates, world.best_solution_value);
                        }
                    }
                }
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }
    }
}
