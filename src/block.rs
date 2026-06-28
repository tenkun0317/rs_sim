use crate::constants::{
    BARREL_MAX_STRENGTH, BARREL_STRENGTH_MASK, BUTTON_COUNTER_MASK, BUTTON_COUNTER_SHIFT,
    BUTTON_DEFAULT_TICKS, BUTTON_MAX_COUNTER, BUTTON_POWERED_SHIFT, COMPARATOR_MODE_SHIFT,
    COMPARATOR_POWERED_SHIFT, DIR_MASK, LEVER_POWERED_SHIFT, REPEATER_COUNTER_MASK,
    REPEATER_COUNTER_SHIFT, REPEATER_DELAY_MASK, REPEATER_DELAY_SHIFT, REPEATER_LOCKED_SHIFT,
    REPEATER_MAX_COUNTER, REPEATER_MAX_DELAY, REPEATER_POWERED_SHIFT, TORCH_DIR_SHIFT,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North = 0,
    South = 1,
    East = 2,
    West = 3,
}

impl Direction {
    pub const ALL: [Direction; 4] = [Direction::North, Direction::South, Direction::East, Direction::West];

    pub fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    pub fn dx(self) -> i32 {
        match self {
            Direction::East => 1,
            Direction::West => -1,
            _ => 0,
        }
    }

    pub fn dy(self) -> i32 {
        match self {
            Direction::South => 1,
            Direction::North => -1,
            _ => 0,
        }
    }

    pub fn from_delta(dx: i32, dy: i32) -> Option<Self> {
        match (dx, dy) {
            (0, -1) => Some(Direction::North),
            (0, 1) => Some(Direction::South),
            (1, 0) => Some(Direction::East),
            (-1, 0) => Some(Direction::West),
            _ => None,
        }
    }

    pub fn rotate_cw(self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    pub fn rotate_ccw(self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockId {
    Air,
    SolidBlock,
    RedstoneWire,
    RedstoneTorch,
    RedstoneBlock,
    Repeater,
    Comparator,
    Lever,
    Button,
    RedstoneLamp,
    Target,
    Barrel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparatorMode {
    Compare = 0,
    Subtract = 1,
}

pub fn encode_dir(d: Direction) -> u16 { d as u16 }

pub fn decode_dir(data: u16) -> Direction {
    match data & DIR_MASK {
        0 => Direction::North,
        1 => Direction::South,
        2 => Direction::East,
        3 => Direction::West,
        _ => Direction::North,
    }
}

pub fn encode_repeater(dir: Direction, delay: u8, locked: bool, powered: bool, counter: u8) -> u16 {
    encode_dir(dir)
        | ((delay as u16).min(REPEATER_MAX_DELAY as u16) << REPEATER_DELAY_SHIFT)
        | ((locked as u16) << REPEATER_LOCKED_SHIFT)
        | ((powered as u16) << REPEATER_POWERED_SHIFT)
        | ((counter as u16).min(REPEATER_MAX_COUNTER as u16) << REPEATER_COUNTER_SHIFT)
}

pub fn decode_repeater_dir(data: u16) -> Direction { decode_dir(data) }
pub fn decode_repeater_delay(data: u16) -> u8 { ((data >> REPEATER_DELAY_SHIFT) & REPEATER_DELAY_MASK as u16) as u8 }
pub fn decode_repeater_locked(data: u16) -> bool { ((data >> REPEATER_LOCKED_SHIFT) & 1) != 0 }
pub fn decode_repeater_powered(data: u16) -> bool { ((data >> REPEATER_POWERED_SHIFT) & 1) != 0 }
pub fn decode_repeater_counter(data: u16) -> u8 { ((data >> REPEATER_COUNTER_SHIFT) & REPEATER_COUNTER_MASK as u16) as u8 }

pub fn encode_comparator(dir: Direction, mode: ComparatorMode, powered: bool) -> u16 {
    encode_dir(dir) | ((mode as u16) << COMPARATOR_MODE_SHIFT) | ((powered as u16) << COMPARATOR_POWERED_SHIFT)
}

pub fn decode_comparator_dir(data: u16) -> Direction { decode_dir(data) }
pub fn decode_comparator_mode(data: u16) -> ComparatorMode {
    if ((data >> COMPARATOR_MODE_SHIFT) & 1) != 0 { ComparatorMode::Subtract } else { ComparatorMode::Compare }
}
pub fn decode_comparator_powered(data: u16) -> bool { ((data >> COMPARATOR_POWERED_SHIFT) & 1) != 0 }

pub fn encode_lever(dir: Direction, powered: bool) -> u16 {
    encode_dir(dir) | ((powered as u16) << LEVER_POWERED_SHIFT)
}
pub fn decode_lever_dir(data: u16) -> Direction { decode_dir(data) }
pub fn decode_lever_powered(data: u16) -> bool { ((data >> LEVER_POWERED_SHIFT) & 1) != 0 }

pub fn encode_torch(lit: bool, on_wall: bool, dir: Direction) -> u16 {
    (lit as u16) | ((on_wall as u16) << 1) | ((dir as u16) << TORCH_DIR_SHIFT)
}
pub fn decode_torch_lit(data: u16) -> bool { (data & 1) != 0 }
pub fn decode_torch_on_wall(data: u16) -> bool { ((data >> 1) & 1) != 0 }
pub fn decode_torch_dir(data: u16) -> Direction { decode_dir(data >> TORCH_DIR_SHIFT) }

pub fn encode_lamp(lit: bool) -> u16 { lit as u16 }
pub fn decode_lamp_lit(data: u16) -> bool { (data & 1) != 0 }

pub fn encode_button(dir: Direction, powered: bool, counter: u8) -> u16 {
    encode_dir(dir)
        | ((powered as u16) << BUTTON_POWERED_SHIFT)
        | ((counter as u16).min(BUTTON_MAX_COUNTER as u16) << BUTTON_COUNTER_SHIFT)
}
pub fn decode_button_counter(data: u16) -> u8 { ((data >> BUTTON_COUNTER_SHIFT) & BUTTON_COUNTER_MASK as u16) as u8 }

pub fn encode_barrel(strength: u8) -> u16 { (strength as u16).min(BARREL_MAX_STRENGTH as u16) }
pub fn decode_barrel_strength(data: u16) -> u8 { (data & BARREL_STRENGTH_MASK) as u8 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub power: u8,
    pub data: u16,
}

impl Block {
    pub const fn new(id: BlockId) -> Self {
        Block { id, power: 0, data: 0 }
    }

    pub const fn air() -> Self { Block::new(BlockId::Air) }
    pub const fn solid() -> Self { Block::new(BlockId::SolidBlock) }
    pub const fn wire() -> Self { Block::new(BlockId::RedstoneWire) }
    pub const fn redstone_block() -> Self { Block::new(BlockId::RedstoneBlock) }
    pub const fn lamp() -> Self { Block::new(BlockId::RedstoneLamp) }

    pub fn torch(lit: bool, on_wall: bool, dir: Direction) -> Self {
        Block { id: BlockId::RedstoneTorch, power: 0, data: encode_torch(lit, on_wall, dir) }
    }

    pub fn repeater(dir: Direction, delay: u8, locked: bool, powered: bool) -> Self {
        Block { id: BlockId::Repeater, power: 0, data: encode_repeater(dir, delay, locked, powered, 0) }
    }

    pub fn comparator(dir: Direction, mode: ComparatorMode, powered: bool) -> Self {
        Block { id: BlockId::Comparator, power: 0, data: encode_comparator(dir, mode, powered) }
    }

    pub fn lever(dir: Direction, powered: bool) -> Self {
        Block { id: BlockId::Lever, power: 0, data: encode_lever(dir, powered) }
    }

    pub fn button(dir: Direction, powered: bool) -> Self {
        let counter = if powered { BUTTON_DEFAULT_TICKS } else { 0u8 };
        Block { id: BlockId::Button, power: 0, data: encode_button(dir, powered, counter) }
    }

    pub fn target() -> Self {
        Block { id: BlockId::Target, power: 0, data: 0 }
    }

    pub fn barrel(strength: u8) -> Self {
        Block { id: BlockId::Barrel, power: 0, data: encode_barrel(strength) }
    }

    pub fn display_name(&self) -> &'static str {
        match self.id {
            BlockId::Air => "Air",
            BlockId::SolidBlock => "Block",
            BlockId::RedstoneWire => "Wire",
            BlockId::RedstoneTorch => "Torch",
            BlockId::RedstoneBlock => "Redstone Block",
            BlockId::Repeater => "Repeater",
            BlockId::Comparator => "Comparator",
            BlockId::Lever => "Lever",
            BlockId::Button => "Button",
            BlockId::RedstoneLamp => "Lamp",
            BlockId::Target => "Target",
            BlockId::Barrel => "Barrel",
        }
    }
}
