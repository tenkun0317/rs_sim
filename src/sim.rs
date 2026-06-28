pub mod power;
pub mod components;

use crate::constants::MAX_ITERATIONS;
use crate::world::World;
use power::calculate_wire_power;
use components::update_components;

pub fn update_simulation(world: &mut World) -> bool {
    let mut any_changed = false;
    for _ in 0..MAX_ITERATIONS {
        if step_simulation(world) {
            any_changed = true;
        } else {
            break;
        }
    }
    any_changed
}

pub fn step_simulation(world: &mut World) -> bool {
    calculate_wire_power(world);
    update_components(world)
}
