use crate::block::*;
use crate::constants::{BUTTON_POWERED_SHIFT, CHUNK_SIZE, CHUNK_SIZE_I32};
use crate::world::World;
use super::power;

fn dir_from_neighbor(tx: i32, ty: i32, nx: i32, ny: i32) -> Direction {
    Direction::from_delta(tx - nx, ty - ny).unwrap_or(Direction::North)
}

fn check_repeater_locked(world: &World, x: i32, y: i32, dir: Direction) -> bool {
    for &(sx, sy) in &[
        (x + dir.rotate_ccw().dx(), y + dir.rotate_ccw().dy()),
        (x + dir.rotate_cw().dx(), y + dir.rotate_cw().dy()),
    ] {
        if let Some(block) = world.get(sx, sy) {
            let dir_to_us = dir_from_neighbor(x, y, sx, sy);
            if block.id == BlockId::Repeater {
                let rd = decode_repeater_dir(block.data);
                if decode_repeater_powered(block.data) && rd == dir_to_us {
                    return true;
                }
            }
            if block.id == BlockId::Comparator {
                let cd = decode_comparator_dir(block.data);
                if decode_comparator_powered(block.data) && cd == dir_to_us {
                    return true;
                }
            }
        }
    }
    false
}

pub fn update_components(world: &mut World) -> bool {
    let mut changed = false;
    let mut phase1: Vec<(i32, i32, u16)> = Vec::new();

    for (&(cx, cy), chunk) in &world.chunks {
        let base_x = cx * CHUNK_SIZE_I32;
        let base_y = cy * CHUNK_SIZE_I32;
        for ly in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let wx = base_x + lx as i32;
                let wy = base_y + ly as i32;
                let block = &chunk.blocks[ly][lx];
                if block.id != BlockId::Repeater {
                    continue;
                }
                let dir = decode_repeater_dir(block.data);
                let delay = decode_repeater_delay(block.data);
                let locked = check_repeater_locked(world, wx, wy, dir);
                let was_powered = decode_repeater_powered(block.data);
                let counter = decode_repeater_counter(block.data);

                let (new_powered, new_counter) = if locked {
                    (was_powered, counter)
                } else {
                    let back_x = wx + dir.opposite().dx();
                    let back_y = wy + dir.opposite().dy();
                    let input_power = power::get_input_power_toward(world, back_x, back_y, dir, false);
                    let should_be_powered = input_power > 0;

                    if counter > 0 {
                        if counter > 1 {
                            (was_powered, counter - 1)
                        } else {
                            (!was_powered, 0)
                        }
                    } else if should_be_powered != was_powered {
                        if delay == 0 {
                            (should_be_powered, 0)
                        } else {
                            (was_powered, delay + 1)
                        }
                    } else {
                        (was_powered, 0)
                    }
                };

                let new_data = encode_repeater(dir, delay, locked, new_powered, new_counter);
                if new_data != block.data {
                    phase1.push((wx, wy, new_data));
                }
            }
        }
    }

    for (wx, wy, data) in &phase1 {
        if let Some(block) = world.get_mut(*wx, *wy) {
            block.data = *data;
            changed = true;
        }
    }

    let mut phase2_data: Vec<(i32, i32, u16)> = Vec::new();
    let mut phase2_power: Vec<(i32, i32, u8)> = Vec::new();

    for (&(cx, cy), chunk) in &world.chunks {
        let base_x = cx * CHUNK_SIZE_I32;
        let base_y = cy * CHUNK_SIZE_I32;
        for ly in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let wx = base_x + lx as i32;
                let wy = base_y + ly as i32;
                let block = &chunk.blocks[ly][lx];

                match block.id {
                    BlockId::RedstoneTorch => {
                        let lit = decode_torch_lit(block.data);
                        let on_wall = decode_torch_on_wall(block.data);
                        let dir = decode_torch_dir(block.data);
                        let should_be_lit = !power::torch_is_blocked(world, wx, wy);
                        if lit != should_be_lit {
                            phase2_data.push((wx, wy, encode_torch(should_be_lit, on_wall, dir)));
                        }
                    }

                    BlockId::Repeater => {}

                    BlockId::Comparator => {
                        let dir = decode_comparator_dir(block.data);
                        let mode = decode_comparator_mode(block.data);
                        let back_x = wx + dir.opposite().dx();
                        let back_y = wy + dir.opposite().dy();
                        let input_power = power::get_input_power_toward(world, back_x, back_y, dir, true);

                        let side_left_dir = dir.rotate_ccw();
                        let side_right_dir = dir.rotate_cw();
                        let side_left_x = wx + side_left_dir.dx();
                        let side_left_y = wy + side_left_dir.dy();
                        let side_right_x = wx + side_right_dir.dx();
                        let side_right_y = wy + side_right_dir.dy();
                        let left_power =
                            power::comparator_side_power(world, side_left_x, side_left_y, side_left_dir);
                        let right_power = power::comparator_side_power(
                            world,
                            side_right_x,
                            side_right_y,
                            side_right_dir,
                        );
                        let side_power = left_power.max(right_power);

                        let output_power = match mode {
                            ComparatorMode::Compare => {
                                if input_power >= side_power { input_power } else { 0 }
                            }
                            ComparatorMode::Subtract => input_power.saturating_sub(side_power),
                        };

                        phase2_power.push((wx, wy, output_power));

                        let was_powered = decode_comparator_powered(block.data);
                        let should_be_powered = output_power > 0;

                        if should_be_powered != was_powered {
                            phase2_data.push((wx, wy, encode_comparator(dir, mode, should_be_powered)));
                        }
                    }

                    BlockId::Button => {
                        let dir = decode_lever_dir(block.data);
                        let powered = ((block.data >> BUTTON_POWERED_SHIFT) & 1) != 0;
                        let counter = decode_button_counter(block.data);
                        if powered && counter > 0 {
                            let (new_powered, new_counter) = if counter > 1 {
                                (true, counter - 1)
                            } else {
                                (false, 0)
                            };
                            let new_data = encode_button(dir, new_powered, new_counter);
                            if new_data != block.data {
                                phase2_data.push((wx, wy, new_data));
                            }
                        }
                    }

                    BlockId::RedstoneLamp => {
                        let lit = decode_lamp_lit(block.data);
                        let should_be_lit = power::is_block_powered(world, wx, wy);
                        if lit != should_be_lit {
                            phase2_data.push((wx, wy, encode_lamp(should_be_lit)));
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    for (wx, wy, power) in &phase2_power {
        if let Some(block) = world.get_mut(*wx, *wy) {
            block.power = *power;
            changed = true;
        }
    }
    for (wx, wy, data) in &phase2_data {
        if let Some(block) = world.get_mut(*wx, *wy) {
            block.data = *data;
            changed = true;
        }
    }

    changed
}
