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
	pub fn new(pop_size: usize, function_bounds: (f64, f64), iteration_count: usize, optimization_function: Functions<N>, z_parameter: f64) -> Self {
		let mut population = Vec::with_capacity(pop_size);

		let mut best_solution = Default::default();
		let mut best_solution_value = f64::MAX;

		let mut rng_source = thread_rng();

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
		sorted.sort_by(|first, second| {
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
			if mold.function_value < self.best_solution_value {
				self.best_solution = mold.position;
				self.best_solution_value = mold.function_value;
			}
		}
		self.recalculate_a(iter_number);
		self.recalculate_weights();
	}

	pub fn do_all_iters(&mut self) {
		for iter in 0..self.iteration_count {
			self.do_iteration(iter);
		}
	}

}