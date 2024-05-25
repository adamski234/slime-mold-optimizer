use std::ops::AddAssign;

pub struct BatchRunData {
    pub min_result: f64,
    pub max_result: f64,
    pub average: f64,
    pub run_count: usize,
}

impl BatchRunData {
    pub fn new() -> Self {
        return Self {
            min_result: f64::MAX,
            max_result: f64::MIN,
            average: 0.0,
            run_count: 0,
        };
    }
}

impl AddAssign for BatchRunData {
    fn add_assign(&mut self, other: Self) {
        if other.max_result > self.max_result {
            self.max_result = other.max_result;
        }
        if other.min_result < self.min_result {
            self.min_result = other.min_result;
        }
        let self_sum = self.average * self.run_count as f64;
        let other_sum = other.average * other.run_count as f64;
        self.run_count += other.run_count;
        self.average = (self_sum + other_sum) / self.run_count as f64;
    }
}

impl AddAssign<f64> for BatchRunData {
    fn add_assign(&mut self, rhs: f64) {
        if rhs > self.max_result {
            self.max_result = rhs;
        }
        if rhs < self.min_result {
            self.min_result = rhs;
        }
        let previous_sum = self.average * self.run_count as f64;
        self.run_count += 1;
        self.average = (previous_sum + rhs) / self.run_count as f64;
        
    }
}