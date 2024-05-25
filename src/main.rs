use slimes::{functions::Functions, slime::WorldState};

fn main() {
    let func = Functions::<20>::make_from_name("ackley");
    let mut world = WorldState::new(100, func.get_bounds(), 1000, func, 0.02);
    world.do_all_iters();
    println!("{}", world.best_solution_value);
}
