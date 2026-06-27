use crate::block::*;
use crate::world::World;
use std::collections::VecDeque;

const MAX_ITERATIONS: usize = 20;

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

struct SourceEntry {
    x: i32,
    y: i32,
    power: u8,
}

fn calculate_wire_power(world: &mut World) {
    for block in world.blocks.iter_mut() {
        if block.id == BlockId::RedstoneWire {
            block.power = 0;
        }
    }

    let mut sources: Vec<SourceEntry> = Vec::new();

    for y in 0..world.height {
        for x in 0..world.width {
            let bx = x as i32;
            let by = y as i32;
            let idx = y * world.width + x;
            let block = &world.blocks[idx];

            if let Some(power) = get_emitted_power(block) {
                let dirs = get_emission_dirs(block).unwrap_or_else(|| Direction::ALL.to_vec());
                for dir in dirs {
                    sources.push(SourceEntry {
                        x: bx + dir.dx(),
                        y: by + dir.dy(),
                        power,
                    });
                }
            }

            let strong_power = block_is_strongly_powered(world, bx, by);
            if matches!(
                block.id,
                BlockId::SolidBlock | BlockId::Target | BlockId::RedstoneLamp | BlockId::Barrel
            ) && strong_power > 0
            {
                for dir in Direction::ALL {
                    sources.push(SourceEntry {
                        x: bx + dir.dx(),
                        y: by + dir.dy(),
                        power: strong_power,
                    });
                }
            }
        }
    }

    let mut queue: VecDeque<(i32, i32, u8)> = VecDeque::new();

    for source in &sources {
        if let Some(neighbor) = world.get_mut_local(source.x, source.y) {
            if neighbor.id == BlockId::RedstoneWire && source.power > neighbor.power {
                neighbor.power = source.power;
                queue.push_back((source.x, source.y, source.power));
            }
        }
    }

    while let Some((x, y, power)) = queue.pop_front() {
        if power <= 1 {
            continue;
        }
        let new_power = power - 1;

        for dir in Direction::ALL {
            let (nx, ny) = (x + dir.dx(), y + dir.dy());
            if !world.in_bounds_local(nx, ny) {
                continue;
            }
            let ni = ny as usize * world.width + nx as usize;

            if world.blocks[ni].id == BlockId::RedstoneWire && new_power > world.blocks[ni].power {
                world.blocks[ni].power = new_power;
                queue.push_back((nx, ny, new_power));
            }
        }
    }

    for y in 0..world.height {
        for x in 0..world.width {
            let idx = y * world.width + x;
            let block = &world.blocks[idx];
            if matches!(
                block.id,
                BlockId::SolidBlock | BlockId::Target | BlockId::RedstoneLamp | BlockId::Barrel
            ) {
                let bx = x as i32;
                let by = y as i32;
                let sp = block_is_strongly_powered(world, bx, by);
                if sp > 0 {
                    world.blocks[idx].power = sp;
                } else {
                    let wp = block_get_power(world, bx, by);
                    world.blocks[idx].power = wp;
                }
            }
        }
    }
}

fn get_emitted_power(block: &Block) -> Option<u8> {
    match block.id {
        BlockId::RedstoneTorch => {
            if decode_torch_lit(block.data) {
                Some(15)
            } else {
                None
            }
        }
        BlockId::RedstoneBlock => Some(15),
        BlockId::Lever => {
            if decode_lever_powered(block.data) {
                Some(15)
            } else {
                None
            }
        }
        BlockId::Button => {
            if ((block.data >> 2) & 1) != 0 {
                Some(15)
            } else {
                None
            }
        }
        BlockId::Repeater => {
            if decode_repeater_powered(block.data) {
                Some(15)
            } else {
                None
            }
        }
        BlockId::Comparator => {
            if decode_comparator_powered(block.data) {
                Some(block.power)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_emission_dirs(block: &Block) -> Option<Vec<Direction>> {
    match block.id {
        BlockId::RedstoneTorch => {
            if decode_torch_lit(block.data) && decode_torch_on_wall(block.data) {
                let attach = decode_torch_dir(block.data);
                let dirs: Vec<Direction> = Direction::ALL
                    .iter()
                    .copied()
                    .filter(|d| *d != attach)
                    .collect();
                Some(dirs)
            } else {
                None
            }
        }
        BlockId::Repeater => {
            if decode_repeater_powered(block.data) {
                Some(vec![decode_repeater_dir(block.data)])
            } else {
                None
            }
        }
        BlockId::Comparator => {
            if decode_comparator_powered(block.data) {
                Some(vec![decode_comparator_dir(block.data)])
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn block_is_strongly_powered(world: &World, x: i32, y: i32) -> u8 {
    for dir in Direction::ALL {
        let nx = x + dir.dx();
        let ny = y + dir.dy();
        let Some(neighbor) = world.get_local(nx, ny) else {
            continue;
        };
        let dir_from_neighbor = dir.opposite();

        match neighbor.id {
            BlockId::RedstoneTorch => {
                if decode_torch_lit(neighbor.data)
                    && (!decode_torch_on_wall(neighbor.data)
                        || decode_torch_dir(neighbor.data) != dir_from_neighbor)
                {
                    return 15;
                }
            }
            BlockId::RedstoneBlock => {}
            BlockId::Repeater => {
                let rd = decode_repeater_dir(neighbor.data);
                if decode_repeater_powered(neighbor.data) && rd == dir_from_neighbor {
                    return 15;
                }
            }
            BlockId::Comparator => {
                let cd = decode_comparator_dir(neighbor.data);
                if decode_comparator_powered(neighbor.data) && cd == dir_from_neighbor {
                    return neighbor.power;
                }
            }
            _ => {}
        }
    }
    0
}

pub fn wire_connects_in_dir(world: &World, x: i32, y: i32, dir: Direction) -> bool {
    let (nx, ny) = (x + dir.dx(), y + dir.dy());
    let Some(neighbor) = world.get_local(nx, ny) else {
        return false;
    };
    match neighbor.id {
        BlockId::RedstoneWire => true,
        BlockId::RedstoneTorch => true,
        BlockId::RedstoneBlock => true,
        BlockId::Repeater => {
            let rd = decode_repeater_dir(neighbor.data);
            let dir_to_wire = dir.opposite();
            rd == dir_to_wire || rd == dir_to_wire.opposite()
        }
        BlockId::Comparator => true,
        BlockId::Lever => true,
        BlockId::Button => true,
        BlockId::Target => true,
        _ => false,
    }
}

fn wire_powers_toward(world: &World, x: i32, y: i32, dir: Direction) -> bool {
    let conns: Vec<Direction> = Direction::ALL
        .iter()
        .copied()
        .filter(|&d| wire_connects_in_dir(world, x, y, d))
        .collect();

    match conns.len() {
        0 => true,
        1 => {
            let d = conns[0];
            dir == d || dir == d.opposite()
        }
        _ => wire_connects_in_dir(world, x, y, dir),
    }
}

fn block_has_power(world: &World, x: i32, y: i32, exclude: Option<(i32, i32)>) -> bool {
    for dir in Direction::ALL {
        let (nx, ny) = (x + dir.dx(), y + dir.dy());
        if exclude == Some((nx, ny)) {
            continue;
        }
        let Some(neighbor) = world.get_local(nx, ny) else {
            continue;
        };

        match neighbor.id {
            BlockId::RedstoneWire => {
                if neighbor.power > 0 && wire_powers_toward(world, nx, ny, dir.opposite()) {
                    return true;
                }
            }
            BlockId::RedstoneTorch => {
                if decode_torch_lit(neighbor.data)
                    && (!decode_torch_on_wall(neighbor.data)
                        || decode_torch_dir(neighbor.data) != dir.opposite())
                {
                    return true;
                }
            }
            BlockId::RedstoneBlock => {
                return true;
            }
            BlockId::Lever => {
                if decode_lever_powered(neighbor.data) {
                    return true;
                }
            }
            BlockId::Button => {
                if ((neighbor.data >> 2) & 1) != 0 {
                    return true;
                }
            }
            BlockId::Repeater => {
                let rd = decode_repeater_dir(neighbor.data);
                if decode_repeater_powered(neighbor.data) && rd == dir.opposite() {
                    return true;
                }
            }
            BlockId::Comparator => {
                let cd = decode_comparator_dir(neighbor.data);
                if decode_comparator_powered(neighbor.data) && cd == dir.opposite() {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn block_get_power(world: &World, x: i32, y: i32) -> u8 {
    let mut max_power = 0u8;
    for dir in Direction::ALL {
        let (nx, ny) = (x + dir.dx(), y + dir.dy());
        let Some(neighbor) = world.get_local(nx, ny) else {
            continue;
        };

        let p = match neighbor.id {
            BlockId::RedstoneWire => {
                if neighbor.power > 0 && wire_powers_toward(world, nx, ny, dir.opposite()) {
                    neighbor.power
                } else {
                    0
                }
            }
            BlockId::RedstoneTorch => {
                if decode_torch_lit(neighbor.data) {
                    if !decode_torch_on_wall(neighbor.data)
                        || decode_torch_dir(neighbor.data) != dir.opposite()
                    {
                        15
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            BlockId::RedstoneBlock => 15,
            BlockId::Lever => {
                if decode_lever_powered(neighbor.data) {
                    15
                } else {
                    0
                }
            }
            BlockId::Button => {
                if ((neighbor.data >> 2) & 1) != 0 {
                    15
                } else {
                    0
                }
            }
            BlockId::Repeater => {
                let rd = decode_repeater_dir(neighbor.data);
                if decode_repeater_powered(neighbor.data) && rd == dir.opposite() {
                    15
                } else {
                    0
                }
            }
            BlockId::Comparator => {
                let cd = decode_comparator_dir(neighbor.data);
                if decode_comparator_powered(neighbor.data) && cd == dir.opposite() {
                    neighbor.power
                } else {
                    0
                }
            }
            BlockId::SolidBlock | BlockId::Target | BlockId::Barrel => 0,
            _ => 0,
        };
        if p > max_power {
            max_power = p;
        }
    }
    max_power
}

fn torch_is_blocked(world: &World, x: i32, y: i32) -> bool {
    let Some(block) = world.get_local(x, y) else {
        return false;
    };
    let on_wall = decode_torch_on_wall(block.data);
    let dir = decode_torch_dir(block.data);

    if on_wall {
        let (ax, ay) = (x + dir.dx(), y + dir.dy());
        block_has_power(world, ax, ay, Some((x, y)))
    } else {
        false
    }
}

fn get_input_power(world: &World, x: i32, y: i32, read_containers: bool) -> u8 {
    let Some(block) = world.get_local(x, y) else {
        return 0;
    };

    match block.id {
        BlockId::Air => 0,
        BlockId::RedstoneWire => block.power,
        BlockId::RedstoneTorch => {
            if decode_torch_lit(block.data) {
                15
            } else {
                0
            }
        }
        BlockId::RedstoneBlock => 15,
        BlockId::Lever => {
            if decode_lever_powered(block.data) {
                15
            } else {
                0
            }
        }
        BlockId::Button => {
            if ((block.data >> 2) & 1) != 0 {
                15
            } else {
                0
            }
        }
        BlockId::Repeater => {
            if decode_repeater_powered(block.data) {
                15
            } else {
                0
            }
        }
        BlockId::Comparator => {
            if decode_comparator_powered(block.data) {
                block.power
            } else {
                0
            }
        }
        BlockId::SolidBlock | BlockId::Target | BlockId::RedstoneLamp => {
            block_get_power(world, x, y)
        }
        BlockId::Barrel => {
            let p = block_get_power(world, x, y);
            if read_containers {
                p.max(decode_barrel_strength(block.data))
            } else {
                p
            }
        }
        _ => 0,
    }
}

fn get_input_power_from_side(world: &World, x: i32, y: i32, from_caller: Direction) -> u8 {
    let Some(block) = world.get_local(x, y) else {
        return 0;
    };
    match block.id {
        BlockId::RedstoneWire => block.power,
        BlockId::RedstoneTorch => {
            if decode_torch_lit(block.data) {
                15
            } else {
                0
            }
        }
        BlockId::RedstoneBlock => 15,
        BlockId::Lever => {
            if decode_lever_powered(block.data) {
                15
            } else {
                0
            }
        }
        BlockId::Button => {
            if ((block.data >> 2) & 1) != 0 {
                15
            } else {
                0
            }
        }
        BlockId::Repeater => {
            if decode_repeater_powered(block.data)
                && decode_repeater_dir(block.data) == from_caller.opposite()
            {
                15
            } else {
                0
            }
        }
        BlockId::Comparator => {
            if decode_comparator_powered(block.data)
                && decode_comparator_dir(block.data) == from_caller.opposite()
            {
                block.power
            } else {
                0
            }
        }
        BlockId::SolidBlock | BlockId::Target | BlockId::RedstoneLamp | BlockId::Barrel => {
            block_is_strongly_powered(world, x, y)
        }
        _ => 0,
    }
}

fn dir_from_neighbor(tx: i32, ty: i32, nx: i32, ny: i32) -> Direction {
    Direction::from_delta(tx - nx, ty - ny).unwrap_or(Direction::North)
}

fn update_components(world: &mut World) -> bool {
    let mut changed = false;

    // Phase 1: Repeaters (must update before comparators)
    for y in 0..world.height {
        for x in 0..world.width {
            let bx = x as i32;
            let by = y as i32;
            let idx = y * world.width + x;
            let block = world.blocks[idx];
            if block.id != BlockId::Repeater {
                continue;
            }
            let dir = decode_repeater_dir(block.data);
            let delay = decode_repeater_delay(block.data);
            let locked = check_repeater_locked(world, bx, by, dir);
            let was_powered = decode_repeater_powered(block.data);
            let counter = decode_repeater_counter(block.data);

            let (new_powered, new_counter) = if locked {
                (was_powered, counter)
            } else {
                let back_x = bx + dir.opposite().dx();
                let back_y = by + dir.opposite().dy();
                let input_power = get_input_power(world, back_x, back_y, false);
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
                world.blocks[idx].data = new_data;
                changed = true;
            }
        }
    }

    // Phase 2: Everything else (comparators, torches, lamps, etc.)
    for y in 0..world.height {
        for x in 0..world.width {
            let bx = x as i32;
            let by = y as i32;
            let idx = y * world.width + x;
            let block = world.blocks[idx];

            match block.id {
                BlockId::RedstoneTorch => {
                    let lit = decode_torch_lit(block.data);
                    let on_wall = decode_torch_on_wall(block.data);
                    let dir = decode_torch_dir(block.data);
                    let should_be_lit = !torch_is_blocked(world, bx, by);
                    if lit != should_be_lit {
                        world.blocks[idx].data = encode_torch(should_be_lit, on_wall, dir);
                        changed = true;
                    }
                }

                BlockId::Repeater => {}

                BlockId::Comparator => {
                    let dir = decode_comparator_dir(block.data);
                    let mode = decode_comparator_mode(block.data);
                    let back_x = bx + dir.opposite().dx();
                    let back_y = by + dir.opposite().dy();
                    let input_power = get_input_power(world, back_x, back_y, true);

                    let side_left_dir = dir.rotate_ccw();
                    let side_right_dir = dir.rotate_cw();
                    let side_left_x = bx + side_left_dir.dx();
                    let side_left_y = by + side_left_dir.dy();
                    let side_right_x = bx + side_right_dir.dx();
                    let side_right_y = by + side_right_dir.dy();
                    let left_power =
                        get_input_power_from_side(world, side_left_x, side_left_y, side_left_dir);
                    let right_power = get_input_power_from_side(
                        world,
                        side_right_x,
                        side_right_y,
                        side_right_dir,
                    );
                    let side_power = left_power.max(right_power);

                    let output_power = match mode {
                        ComparatorMode::Compare => {
                            if input_power >= side_power {
                                input_power
                            } else {
                                0
                            }
                        }
                        ComparatorMode::Subtract => input_power.saturating_sub(side_power),
                    };

                    world.blocks[idx].power = output_power;

                    let was_powered = decode_comparator_powered(block.data);
                    let should_be_powered = output_power > 0;

                    if should_be_powered != was_powered {
                        world.blocks[idx].data = encode_comparator(dir, mode, should_be_powered);
                        changed = true;
                    }
                }

                BlockId::Button => {
                    let dir = decode_lever_dir(block.data);
                    let powered = ((block.data >> 2) & 1) != 0;
                    let counter = decode_button_counter(block.data);
                    if powered && counter > 0 {
                        let (new_powered, new_counter) = if counter > 1 {
                            (true, counter - 1)
                        } else {
                            (false, 0)
                        };
                        let new_data = encode_button(dir, new_powered, new_counter);
                        if new_data != block.data {
                            world.blocks[idx].data = new_data;
                            changed = true;
                        }
                    }
                }

                BlockId::RedstoneLamp => {
                    let lit = decode_lamp_lit(block.data);
                    let should_be_lit = is_block_powered(world, bx, by);
                    if lit != should_be_lit {
                        world.blocks[idx].data = encode_lamp(should_be_lit);
                        changed = true;
                    }
                }

                _ => {}
            }
        }
    }

    changed
}

fn is_block_powered(world: &World, x: i32, y: i32) -> bool {
    for dir in Direction::ALL {
        let (nx, ny) = (x + dir.dx(), y + dir.dy());
        let Some(neighbor) = world.get_local(nx, ny) else {
            continue;
        };

        match neighbor.id {
            BlockId::RedstoneWire => {
                if neighbor.power > 0 && wire_powers_toward(world, nx, ny, dir.opposite()) {
                    return true;
                }
            }
            BlockId::RedstoneTorch => {
                if decode_torch_lit(neighbor.data)
                    && (!decode_torch_on_wall(neighbor.data)
                        || decode_torch_dir(neighbor.data) != dir.opposite())
                {
                    return true;
                }
            }
            BlockId::RedstoneBlock => {
                return true;
            }
            BlockId::Lever => {
                if decode_lever_powered(neighbor.data) {
                    return true;
                }
            }
            BlockId::Button => {
                if ((neighbor.data >> 2) & 1) != 0 {
                    return true;
                }
            }
            BlockId::Repeater => {
                let rd = decode_repeater_dir(neighbor.data);
                if decode_repeater_powered(neighbor.data) && rd == dir.opposite() {
                    return true;
                }
            }
            BlockId::Comparator => {
                let cd = decode_comparator_dir(neighbor.data);
                if decode_comparator_powered(neighbor.data) && cd == dir.opposite() {
                    return true;
                }
            }
            BlockId::SolidBlock | BlockId::Target | BlockId::Barrel => {
                if block_is_strongly_powered(world, nx, ny) > 0 {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn check_repeater_locked(world: &World, x: i32, y: i32, dir: Direction) -> bool {
    for &(sx, sy) in &[
        (x + dir.rotate_ccw().dx(), y + dir.rotate_ccw().dy()),
        (x + dir.rotate_cw().dx(), y + dir.rotate_cw().dy()),
    ] {
        if let Some(block) = world.get_local(sx, sy) {
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
