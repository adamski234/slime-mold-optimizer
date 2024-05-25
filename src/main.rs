use slimes::{functions::Functions, slime::MultiSwarmWorldState};

fn main() {
    let func = Functions::<20>::Schwefel2;
    //let mut world = WorldState::new(100, func.get_bounds(), 1000, func, 0.02);
    let mut world = MultiSwarmWorldState::new(5, 0.01, 20, func.get_bounds(), 1000, func, 0.02);
    world.do_all_iters();
    println!("{}", world.best_solution_value);
}
