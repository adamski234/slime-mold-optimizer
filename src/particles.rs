use rand::prelude::*;

use crate::{functions::Functions, vector::VectorN};

#[derive(Debug, Clone)]
pub struct Particle<const N: usize> {
	pub current_speed: VectorN<N>,
	pub coordinates: VectorN<N>,
	pub best_found_solution: VectorN<N>, // of this particle
	best_found_solution_value: f64,
	pub bounds: (f64, f64), // lower, upper
	pub social_coefficient: f64,
	pub cognitive_coefficient: f64,
	pub inertia_coefficient: f64,
	function: Functions<N>,
	function_value: f64,
}

impl<const N: usize> Particle<N> {
	fn move_particle(&mut self, best_global_solution: VectorN<N>, random_source: &mut ThreadRng) {
		let inertia_part = self.current_speed * self.inertia_coefficient;
		let social_part = (best_global_solution - self.coordinates) * self.social_coefficient * random_source.gen::<f64>();
		let self_part = (self.best_found_solution - self.coordinates) * self.cognitive_coefficient * random_source.gen::<f64>();
		self.current_speed = inertia_part + social_part + self_part;
		self.coordinates += self.current_speed * 1.0;

		self.coordinates.clamp(self.bounds);

		self.function_value = (self.function)(self.coordinates);
	}
}

impl<const N: usize> PartialEq for Particle<N> {
	fn eq(&self, other: &Self) -> bool {
		return self.coordinates == other.coordinates;
	}
}

impl<const N: usize> PartialOrd for Particle<N> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		return self.function_value.partial_cmp(&other.function_value);
	}
}

#[derive(Debug, Clone)]
pub struct WorldState<const DIMENSIONS: usize> {
	pub particles: Vec<Particle<DIMENSIONS>>,
	pub function: Functions<DIMENSIONS>,
	pub best_solution: VectorN<DIMENSIONS>,
	pub best_solution_value: f64,
	bounds: (f64, f64),
	particle_count: usize,
	social_coefficient: f64,
	cognitive_coefficient: f64,
	inertia_coefficient: f64,
	random_generator: ThreadRng,
}

impl<const DIMENSIONS: usize> WorldState<DIMENSIONS> {
	pub fn new(particle_count: usize, function: Functions<DIMENSIONS>, bounds: (f64, f64), social_coefficient: f64, cognitive_coefficient: f64, inertia_coefficient: f64) -> Self {
		if bounds.0 >= bounds.1 {
			panic!("Incorrect order of bounds or zero size");
		}
		let mut result = Self {
			random_generator: thread_rng(),
			particles: Vec::with_capacity(particle_count),
			function,
			best_solution: VectorN::default(),
			best_solution_value: f64::INFINITY,
			bounds,
			particle_count,
			social_coefficient,
			cognitive_coefficient,
			inertia_coefficient,
		};

		result.create_particles();

		return result;
	}

	fn create_particles(&mut self) {
		let size = self.bounds.1 - self.bounds.0;
		let mut best_solution = f64::INFINITY;
		for _ in 0..self.particle_count {
			let mut coords = [0.0; DIMENSIONS];
			coords.fill_with(|| self.random_generator.gen::<f64>() * size + self.bounds.0);
			let value_at_coords = (self.function)(VectorN::new(coords));
			self.particles.push(Particle {
				current_speed: VectorN::default(),
				coordinates: VectorN::new(coords),
				best_found_solution: VectorN::new(coords),
				best_found_solution_value: value_at_coords,
				bounds: self.bounds,
				social_coefficient: self.social_coefficient,
				cognitive_coefficient: self.cognitive_coefficient,
				inertia_coefficient: self.inertia_coefficient,
				function: self.function,
				function_value: (self.function)(VectorN::new(coords)),
			});
			if value_at_coords < best_solution {
				best_solution = value_at_coords;
				self.best_solution = VectorN::new(coords);
			}
		}
	}

	pub fn reset(&mut self) {
		let size = self.bounds.1 - self.bounds.0;
		let mut best_solution = f64::INFINITY;
		for particle in &mut self.particles {
			let mut coords = [0.0; DIMENSIONS];
			coords.fill_with(|| self.random_generator.gen::<f64>() * size + self.bounds.0);
			particle.current_speed = VectorN::default();
			particle.coordinates = VectorN::new(coords);
			particle.best_found_solution = VectorN::new(coords);
			let particle_solution = (self.function)(VectorN::new(coords));
			if particle_solution < best_solution {
				best_solution = particle_solution;
				self.best_solution = VectorN::new(coords);
				self.best_solution_value = (self.function)(self.best_solution);
			}
		}
	}

	pub fn set_coeffs(&mut self, social_coefficient: f64, cognitive_coefficient: f64, inertia_coefficient: f64) {
		self.social_coefficient = social_coefficient;
		self.cognitive_coefficient = cognitive_coefficient;
		self.inertia_coefficient = inertia_coefficient;
		for particle in &mut self.particles {
			particle.social_coefficient = social_coefficient;
			particle.cognitive_coefficient = cognitive_coefficient;
			particle.inertia_coefficient = inertia_coefficient;
		}
	}

	pub fn get_coeffs(&self) -> (f64, f64, f64) { // same order as set_coeffs
		return (self.social_coefficient, self.cognitive_coefficient, self.inertia_coefficient);
	}

	pub fn update_best_solutions(&mut self) {
		for particle in &mut self.particles {
			let particle_solution = (self.function)(particle.coordinates);
			if particle_solution < self.best_solution_value {
				self.best_solution_value = particle_solution;
				self.best_solution = particle.coordinates;
			}
			if particle_solution < particle.best_found_solution_value {
				particle.best_found_solution = particle.coordinates;
				particle.best_found_solution_value = particle_solution;
			}
		}
	}

	pub fn move_particles(&mut self) {
		for particle in &mut self.particles {
			particle.move_particle(self.best_solution, &mut self.random_generator);
		}
	}

	pub fn do_iteration(&mut self) {
		self.move_particles();
		self.update_best_solutions();
	}

	pub fn do_all_iterations(&mut self, iteration_count: usize) {
		for _ in 0..iteration_count {
			self.do_iteration();
		}
	}
}

#[derive(Clone, Debug)]
pub struct MultiSwarmWorldState<const N: usize> {
	swarms: Vec<WorldState<N>>,
	migration_threshold: f64,
	pub best_solution: VectorN<N>,
	pub best_solution_value: f64,
}

impl<const N: usize> MultiSwarmWorldState<N> {
	pub fn new(swarm_count: usize, migration_threshold: f64, particle_count: usize, function: Functions<N>, bounds: (f64, f64), social_coefficient: f64, cognitive_coefficient: f64, inertia_coefficient: f64) -> Self {
		let mut swarms = Vec::with_capacity(swarm_count);
		let mut best_solution = VectorN::default();
		let mut best_solution_value = f64::MAX;
		for _ in 0..swarm_count {
			let world = WorldState::new(particle_count, function, bounds, social_coefficient, cognitive_coefficient, inertia_coefficient);
			if world.best_solution_value < best_solution_value {
				best_solution = world.best_solution;
				best_solution_value = world.best_solution_value;
			}
			swarms.push(world);
		}

		return Self {
			swarms, migration_threshold, best_solution, best_solution_value
		};
	}

	fn update_best_solutions(&mut self) {
		for swarm in self.swarms.iter() {
			if swarm.best_solution_value < self.best_solution_value {
				self.best_solution = swarm.best_solution;
				self.best_solution_value = swarm.best_solution_value;
			}
		}
	}

	pub fn do_iteration(&mut self) {
		for swarm in &mut self.swarms {
			swarm.do_iteration();
		}
		// migration
		let size = self.swarms.len();
		// iterate over distinct pairs
		// Can't use iterators as it requires mutable references to two swarm
		// Will migrate best from both parts to one of the swarms and worst to the other
		for first_index in 0..(size - 1) {
			for second_index in (first_index + 1)..size {
				let difference = (self.swarms[second_index].best_solution_value - self.swarms[first_index].best_solution_value).abs();
				if difference > self.migration_threshold {
					let lambda = difference / self.swarms[second_index].best_solution_value.max(self.swarms[first_index].best_solution_value);
					let migration_count = (lambda * self.swarms[first_index].particles.len() as f64) as usize;

					let mut sorted_first = self.swarms[first_index].particles.clone();
					sorted_first.sort_by(|a, b| a.partial_cmp(b).unwrap()); // Worst are first

					let mut sorted_second_reverse = self.swarms[second_index].particles.clone();
					sorted_second_reverse.sort_by(|a, b| a.partial_cmp(b).unwrap().reverse()); // Best are first
					
					let mut migrated_first = sorted_first.split_off(migration_count); // contains best from first
					let mut left_second = sorted_second_reverse.split_off(migration_count); // contains worst from second

					// sorted_first now contains the unmigrated part of first swarm, so does left_second for the second swarm
					// now merge the migrated and unmigrated parts

					migrated_first.append(&mut sorted_second_reverse);
					sorted_first.append(&mut left_second);

					migrated_first.shuffle(&mut thread_rng());
					sorted_first.shuffle(&mut thread_rng());

					self.swarms[first_index].particles = migrated_first;
					self.swarms[second_index].particles = sorted_first;

					// update both best solutions
					self.swarms[first_index].update_best_solutions();
					self.swarms[second_index].update_best_solutions();
				}
			}
		}

		self.update_best_solutions();
	}

	pub fn do_all_iters(&mut self, iterations: usize) {
		for _ in 0..iterations {
			self.do_iteration();
		}
	}

	pub fn reset(&mut self) {
		self.best_solution_value = f64::MAX;
		for swarm in &mut self.swarms {
			swarm.reset();
		}
		self.update_best_solutions();
	}
}