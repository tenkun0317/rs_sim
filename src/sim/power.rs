use crate::block::*;
use crate::constants::{
    BUTTON_POWERED_SHIFT, CHUNK_SIZE, CHUNK_SIZE_I32, MAX_POWER, WIRE_POWER_DECAY,
};
use crate::world::World;
use std::collections::VecDeque;

struct SourceEntry {
    x: i32,
    y: i32,
    power: u8,
}

fn get_emitted_power(block: &Block) -> Option<u8> {
    match block.id {
        BlockId::RedstoneTorch => {
            if decode_torch_lit(block.data) {
                Some(MAX_POWER)
            } else {
                None
            }
        }
        BlockId::RedstoneBlock => Some(MAX_POWER),
        BlockId::Lever => {
            if decode_lever_powered(block.data) {
                Some(MAX_POWER)
            } else {
                None
            }
        }
        BlockId::Button => {
            if ((block.data >> BUTTON_POWERED_SHIFT) & 1) != 0 {
                Some(MAX_POWER)
            } else {
                None
            }
        }
        BlockId::Repeater => {
            if decode_repeater_powered(block.data) {
                Some(MAX_POWER)
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
        let Some(neighbor) = world.get(nx, ny) else {
            continue;
        };
        let dir_from_neighbor = dir.opposite();

        match neighbor.id {
            BlockId::RedstoneTorch => {
                if decode_torch_lit(neighbor.data)
                    && (!decode_torch_on_wall(neighbor.data)
                        || decode_torch_dir(neighbor.data) != dir_from_neighbor)
                {
                    return MAX_POWER;
                }
            }
            BlockId::Repeater => {
                let rd = decode_repeater_dir(neighbor.data);
                if decode_repeater_powered(neighbor.data) && rd == dir_from_neighbor {
                    return MAX_POWER;
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
    let Some(neighbor) = world.get(nx, ny) else {
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
        let Some(neighbor) = world.get(nx, ny) else {
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
            BlockId::Lever => {
                if decode_lever_powered(neighbor.data) {
                    return true;
                }
            }
            BlockId::Button => {
                if ((neighbor.data >> BUTTON_POWERED_SHIFT) & 1) != 0 {
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
        let Some(neighbor) = world.get(nx, ny) else {
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
                        MAX_POWER
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            BlockId::Lever => {
                if decode_lever_powered(neighbor.data) {
                    MAX_POWER
                } else {
                    0
                }
            }
            BlockId::Button => {
                if ((neighbor.data >> BUTTON_POWERED_SHIFT) & 1) != 0 {
                    MAX_POWER
                } else {
                    0
                }
            }
            BlockId::Repeater => {
                let rd = decode_repeater_dir(neighbor.data);
                if decode_repeater_powered(neighbor.data) && rd == dir.opposite() {
                    MAX_POWER
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

pub fn torch_is_blocked(world: &World, x: i32, y: i32) -> bool {
    let Some(block) = world.get(x, y) else {
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

pub fn get_input_power(world: &World, x: i32, y: i32, read_containers: bool) -> u8 {
    let Some(block) = world.get(x, y) else {
        return 0;
    };

    match block.id {
        BlockId::Air => 0,
        BlockId::RedstoneWire => block.power,
        BlockId::RedstoneTorch => {
            if decode_torch_lit(block.data) {
                MAX_POWER
            } else {
                0
            }
        }
        BlockId::RedstoneBlock => MAX_POWER,
        BlockId::Lever => {
            if decode_lever_powered(block.data) {
                MAX_POWER
            } else {
                0
            }
        }
        BlockId::Button => {
            if ((block.data >> BUTTON_POWERED_SHIFT) & 1) != 0 {
                MAX_POWER
            } else {
                0
            }
        }
        BlockId::Repeater => {
            if decode_repeater_powered(block.data) {
                MAX_POWER
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
    }
}

pub fn get_input_power_toward(world: &World, x: i32, y: i32, toward: Direction, read_containers: bool) -> u8 {
    let Some(block) = world.get(x, y) else { return 0 };
    match block.id {
        BlockId::Air => 0,
        BlockId::RedstoneWire => block.power,
        BlockId::RedstoneTorch => {
            if decode_torch_lit(block.data) { MAX_POWER } else { 0 }
        }
        BlockId::RedstoneBlock => MAX_POWER,
        BlockId::Lever => {
            if decode_lever_powered(block.data) { MAX_POWER } else { 0 }
        }
        BlockId::Button => {
            if ((block.data >> BUTTON_POWERED_SHIFT) & 1) != 0 { MAX_POWER } else { 0 }
        }
        BlockId::Repeater => {
            if decode_repeater_powered(block.data) && decode_repeater_dir(block.data) == toward {
                MAX_POWER
            } else {
                0
            }
        }
        BlockId::Comparator => {
            if decode_comparator_powered(block.data) && decode_comparator_dir(block.data) == toward {
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
    }
}

pub fn get_input_power_from_side(world: &World, x: i32, y: i32, from_caller: Direction) -> u8 {
    let Some(block) = world.get(x, y) else {
        return 0;
    };
    match block.id {
        BlockId::RedstoneWire => block.power,
        BlockId::RedstoneTorch => {
            if decode_torch_lit(block.data) {
                MAX_POWER
            } else {
                0
            }
        }
        BlockId::RedstoneBlock => MAX_POWER,
        BlockId::Lever => {
            if decode_lever_powered(block.data) {
                MAX_POWER
            } else {
                0
            }
        }
        BlockId::Button => {
            if ((block.data >> BUTTON_POWERED_SHIFT) & 1) != 0 {
                MAX_POWER
            } else {
                0
            }
        }
        BlockId::Repeater => {
            if decode_repeater_powered(block.data)
                && decode_repeater_dir(block.data) == from_caller.opposite()
            {
                MAX_POWER
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

pub fn comparator_side_power(world: &World, x: i32, y: i32, from_side: Direction) -> u8 {
    let Some(block) = world.get(x, y) else { return 0 };
    match block.id {
        BlockId::RedstoneWire => block.power,
        BlockId::RedstoneBlock => MAX_POWER,
        BlockId::Repeater => {
            if decode_repeater_powered(block.data)
                && decode_repeater_dir(block.data) == from_side.opposite()
            {
                MAX_POWER
            } else {
                0
            }
        }
        BlockId::Comparator => {
            if decode_comparator_powered(block.data)
                && decode_comparator_dir(block.data) == from_side.opposite()
            {
                block.power
            } else {
                0
            }
        }
        _ => 0,
    }
}

pub fn calculate_wire_power(world: &mut World) {
    for chunk in world.chunks.values_mut() {
        for ly in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                if chunk.blocks[ly][lx].id == BlockId::RedstoneWire {
                    chunk.blocks[ly][lx].power = 0;
                }
            }
        }
    }

    let mut sources: Vec<SourceEntry> = Vec::new();

    for (&(cx, cy), chunk) in &world.chunks {
        let base_x = cx * CHUNK_SIZE_I32;
        let base_y = cy * CHUNK_SIZE_I32;
        for ly in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let wx = base_x + lx as i32;
                let wy = base_y + ly as i32;
                let block = &chunk.blocks[ly][lx];

                if let Some(power) = get_emitted_power(block) {
                    let dirs = get_emission_dirs(block).unwrap_or_else(|| Direction::ALL.to_vec());
                    for dir in dirs {
                        sources.push(SourceEntry {
                            x: wx + dir.dx(),
                            y: wy + dir.dy(),
                            power,
                        });
                    }
                }

                let strong_power = block_is_strongly_powered(world, wx, wy);
                if matches!(
                    block.id,
                    BlockId::SolidBlock | BlockId::Target | BlockId::RedstoneLamp | BlockId::Barrel
                ) && strong_power > 0
                {
                    for dir in Direction::ALL {
                        sources.push(SourceEntry {
                            x: wx + dir.dx(),
                            y: wy + dir.dy(),
                            power: strong_power,
                        });
                    }
                }
            }
        }
    }

    let mut queue: VecDeque<(i32, i32, u8)> = VecDeque::new();

    for source in &sources {
        if let Some(neighbor) = world.get_mut(source.x, source.y) {
            if neighbor.id == BlockId::RedstoneWire && source.power > neighbor.power {
                neighbor.power = source.power;
                queue.push_back((source.x, source.y, source.power));
            }
        }
    }

    while let Some((x, y, power)) = queue.pop_front() {
        if power <= WIRE_POWER_DECAY {
            continue;
        }
        let new_power = power - WIRE_POWER_DECAY;

        for dir in Direction::ALL {
            let (nx, ny) = (x + dir.dx(), y + dir.dy());
            let Some(neighbor) = world.get_mut(nx, ny) else { continue; };
            if neighbor.id == BlockId::RedstoneWire && new_power > neighbor.power {
                neighbor.power = new_power;
                queue.push_back((nx, ny, new_power));
            }
        }
    }

    let mut solid_powers: Vec<(i32, i32, u8)> = Vec::new();
    for (&(cx, cy), chunk) in &world.chunks {
        let base_x = cx * CHUNK_SIZE_I32;
        let base_y = cy * CHUNK_SIZE_I32;
        for ly in 0..CHUNK_SIZE {
            for lx in 0..CHUNK_SIZE {
                let wx = base_x + lx as i32;
                let wy = base_y + ly as i32;
                let block = &chunk.blocks[ly][lx];
                if matches!(
                    block.id,
                    BlockId::SolidBlock | BlockId::Target | BlockId::RedstoneLamp | BlockId::Barrel
                ) {
                    let sp = block_is_strongly_powered(world, wx, wy);
                    let power = if sp > 0 { sp } else { block_get_power(world, wx, wy) };
                    solid_powers.push((wx, wy, power));
                }
            }
        }
    }
    for (wx, wy, power) in solid_powers {
        if let Some(block) = world.get_mut(wx, wy) {
            block.power = power;
        }
    }
}

pub fn is_block_powered(world: &World, x: i32, y: i32) -> bool {
    for dir in Direction::ALL {
        let (nx, ny) = (x + dir.dx(), y + dir.dy());
        let Some(neighbor) = world.get(nx, ny) else {
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
                if ((neighbor.data >> BUTTON_POWERED_SHIFT) & 1) != 0 {
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
            BlockId::SolidBlock | BlockId::Target | BlockId::Barrel | BlockId::RedstoneLamp => {
                if block_is_strongly_powered(world, nx, ny) > 0 || block_get_power(world, nx, ny) > 0 {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}
