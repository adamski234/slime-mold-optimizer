use rand_distr::{Distribution, Uniform};
use rand::prelude::*;

use crate::{functions::Functions, vector::VectorN};

#[derive(Debug, Clone)]
pub struct Slime<const N: usize> {
	position: VectorN<N>,
	weight: f64,
	optimization_function: Functions<N>,
	function_bounds: (f64, f64),
	function_value: f64,
	z_parameter: f64,
}

impl<const N: usize> PartialEq for Slime<N> {
	fn eq(&self, other: &Self) -> bool {
		return self.position == other.position;
	}
}

impl<const N: usize> PartialOrd for Slime<N> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		return self.function_value.partial_cmp(&other.function_value);
	}
}

impl<const N: usize> Slime<N> {
	pub fn new(function_bounds: (f64, f64), optimization_function: Functions<N>, z_parameter: f64, random_source: &mut ThreadRng) -> Self {
		let mut coords_array = [0.0; N];
		let range = Uniform::from(function_bounds.0..function_bounds.1);
		coords_array.fill_with(|| range.sample(random_source));
		let position = VectorN::new(coords_array);
		let function_value = optimization_function(position);

		return Self {
			function_bounds, function_value, optimization_function, position, z_parameter,
			weight: 0.0,
		};
	}

	pub fn move_slime(&mut self, a_param: f64, first_slime: &Self, second_slime: &Self, best_global_result: f64, iter_progress: f64, random_source: &mut ThreadRng) {
		// equation 2.7
		if random_source.gen::<f64>() < self.z_parameter {
			let range = Uniform::from(self.function_bounds.0..self.function_bounds.1);
			self.position.coordinates.fill_with(|| range.sample(random_source));
		} else {
			let p_value = (self.function_value - best_global_result).abs().tanh(); // equation 2.2
			if random_source.gen::<f64>() < p_value {
				let vb_param = random_source.gen_range(-a_param..a_param);
				self.position = first_slime.position + (second_slime.position * self.weight - first_slime.position) * vb_param;
			} else {
				let range_size = 1.0 - iter_progress;
				self.position *= random_source.gen_range(-range_size..range_size);
			}
		}

		self.position.clamp(self.function_bounds);

		self.function_value = (self.optimization_function)(self.position);
	}

	fn reset(&mut self, random_source: &mut ThreadRng) {
		let range = Uniform::from(self.function_bounds.0..self.function_bounds.1);
		self.position.coordinates.fill_with(|| range.sample(random_source));
		self.weight = 0.0;
		self.function_value = (self.optimization_function)(self.position);
	}
}

#[derive(Debug, Clone)]
pub struct WorldState<const N: usize> {
	population: Vec<Slime<N>>,
	pub best_solution_value: f64,
	pub best_solution: VectorN<N>,
	a_parameter: f64,
	iteration_count: usize,
	random_source: ThreadRng,
}

impl<const N: usize> WorldState<N> {
	pub fn new(pop_size: usize, function_bounds: (f64, f64), iteration_count: usize, optimization_function: Functions<N>, z_parameter: f64, mut rng_source: ThreadRng) -> Self {
		let mut population = Vec::with_capacity(pop_size);

		let mut best_solution = Default::default();
		let mut best_solution_value = f64::MAX;

		for _ in 0..pop_size {
			let candidate = Slime::new(function_bounds, optimization_function, z_parameter, &mut rng_source);
			if candidate.function_value < best_solution_value {
				best_solution = candidate.position;
				best_solution_value = candidate.function_value;
			}
			population.push(candidate);
		}


		let mut result = Self {
			population, best_solution, best_solution_value,
			a_parameter: 0.0,
			iteration_count,
			random_source: rng_source,
		};

		result.recalculate_a(0);
		result.recalculate_weights();

		return result;
	}

	fn recalculate_a(&mut self, iteration: usize) {
		// add one because original code uses matlab, with 1 as index start
		self.a_parameter = (-((iteration + 1) as f64 / self.iteration_count as f64) + 1.0).atanh();
	}

	fn recalculate_weights(&mut self) {
		let original_clone = self.population.clone();
		let mut sorted = self.population.iter_mut().collect::<Vec<_>>();
		sorted.sort_unstable_by(|first, second| {
			return first.function_value.partial_cmp(&second.function_value).unwrap();
		});
		let best_value_in_iter = sorted[0].function_value;
		let worst_value_in_iter = sorted[sorted.len() - 1].function_value;
		for (index, (sorted, original)) in sorted.into_iter().zip(&original_clone).enumerate() {
			let part = self.random_source.gen::<f64>() * (
				(best_value_in_iter - original.function_value) / (best_value_in_iter - worst_value_in_iter) + 1.0
			).log10(); // Should this be a log10?
			if index < original_clone.len() {
				sorted.weight = 1.0 + part;
			} else {
				sorted.weight = 1.0 - part;
			}
		}
	}

	fn update_best_solutions(&mut self) {
		for mold in self.population.iter() {
			if mold.function_value < self.best_solution_value {
				self.best_solution = mold.position;
				self.best_solution_value = mold.function_value;
			}
		}
	}

	fn do_iteration(&mut self, iter_number: usize) {
		let iter_progress = iter_number as f64 / self.iteration_count as f64;
		let original_clone = self.population.clone();
		for mold in self.population.iter_mut() {
			mold.move_slime(
				self.a_parameter,
				original_clone.choose(&mut self.random_source).unwrap(),
				original_clone.choose(&mut self.random_source).unwrap(),
				self.best_solution_value,
				iter_progress,
				&mut self.random_source
			);
		}
		self.update_best_solutions();
		self.recalculate_a(iter_number);
		self.recalculate_weights();
	}

	pub fn do_all_iters(&mut self) {
		for iter in 0..self.iteration_count {
			self.do_iteration(iter);
		}
	}

	pub fn reset(&mut self) {
		self.best_solution_value = f64::MAX;
		for mold in &mut self.population {
			mold.reset(&mut self.random_source);
			if mold.function_value < self.best_solution_value {
				self.best_solution_value = mold.function_value;
				self.best_solution = mold.position;
			}
		}
		self.recalculate_a(0);
		self.recalculate_weights()
	}

}

#[derive(Clone, Debug)]
pub struct MultiSwarmWorldState<const N: usize> {
	swarms: Vec<WorldState<N>>,
	iteration_count: usize,
	migration_threshold: f64,
	pub best_solution: VectorN<N>,
	pub best_solution_value: f64,
}

impl<const N: usize> MultiSwarmWorldState<N> {
	pub fn new(swarm_count: usize, migration_threshold: f64, pop_size: usize, function_bounds: (f64, f64), iteration_count: usize, optimization_function: Functions<N>, z_parameter: f64) -> Self {
		let mut swarms = Vec::with_capacity(swarm_count);
		let mut best_solution = VectorN::default();
		let mut best_solution_value = f64::MAX;
		for _ in 0..swarm_count {
			let world = WorldState::new(pop_size, function_bounds, iteration_count, optimization_function, z_parameter, thread_rng());
			if world.best_solution_value < best_solution_value {
				best_solution = world.best_solution;
				best_solution_value = world.best_solution_value;
			}
			swarms.push(world);
		}

		return Self {
			swarms, iteration_count, migration_threshold, best_solution, best_solution_value
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

	pub fn do_iteration(&mut self, iteration_number: usize) {
		for swarm in &mut self.swarms {
			swarm.do_iteration(iteration_number);
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
					let migration_count = (lambda * self.swarms[first_index].population.len() as f64) as usize;

					let mut sorted_first = self.swarms[first_index].population.clone();
					sorted_first.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap()); // Worst are first

					let mut sorted_second_reverse = self.swarms[second_index].population.clone();
					sorted_second_reverse.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap().reverse()); // Best are first
					
					let mut migrated_first = sorted_first.split_off(migration_count); // contains best from first
					let mut left_second = sorted_second_reverse.split_off(migration_count); // contains worst from second

					// sorted_first now contains the unmigrated part of first swarm, so does left_second for the second swarm
					// now merge the migrated and unmigrated parts

					migrated_first.append(&mut sorted_second_reverse);
					sorted_first.append(&mut left_second);

					migrated_first.shuffle(&mut thread_rng());
					sorted_first.shuffle(&mut thread_rng());

					self.swarms[first_index].population = migrated_first;
					self.swarms[second_index].population = sorted_first;

					// update both best solutions
					self.swarms[first_index].update_best_solutions();
					self.swarms[second_index].update_best_solutions();
				}
			}
		}

		self.update_best_solutions();
	}

	pub fn do_all_iters(&mut self) {
		for iter in 0..self.iteration_count {
			self.do_iteration(iter);
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