use crate::block::*;
use crate::constants::{CHUNK_SIZE, CHUNK_SIZE_I32, WORLD_CHUNKS_X};
use std::collections::HashMap;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Chunk {
    pub blocks: [[Block; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            blocks: [[Block::air(); CHUNK_SIZE]; CHUNK_SIZE],
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
}

impl World {
    pub fn new(chunks_x: usize, chunks_y: usize) -> Self {
        let mut chunks = HashMap::new();
        for cy in 0..chunks_y as i32 {
            for cx in 0..chunks_x as i32 {
                chunks.insert((cx, cy), Chunk::new());
            }
        }
        World { chunks }
    }

    pub fn chunk_at(&self, x: i32, y: i32) -> (i32, i32) {
        (x.div_euclid(CHUNK_SIZE_I32), y.div_euclid(CHUNK_SIZE_I32))
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&Block> {
        let (cx, cy) = self.chunk_at(x, y);
        let chunk = self.chunks.get(&(cx, cy))?;
        let lx = x.wrapping_sub(cx * CHUNK_SIZE_I32) as usize;
        let ly = y.wrapping_sub(cy * CHUNK_SIZE_I32) as usize;
        debug_assert!(lx < CHUNK_SIZE && ly < CHUNK_SIZE);
        Some(&chunk.blocks[ly][lx])
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut Block> {
        let (cx, cy) = self.chunk_at(x, y);
        let chunk = self.chunks.get_mut(&(cx, cy))?;
        let lx = x.wrapping_sub(cx * CHUNK_SIZE_I32) as usize;
        let ly = y.wrapping_sub(cy * CHUNK_SIZE_I32) as usize;
        debug_assert!(lx < CHUNK_SIZE && ly < CHUNK_SIZE);
        Some(&mut chunk.blocks[ly][lx])
    }

    pub fn set(&mut self, x: i32, y: i32, block: Block) {
        let (cx, cy) = self.chunk_at(x, y);
        let chunk = self.chunks.entry((cx, cy)).or_insert_with(Chunk::new);
        let lx = x.wrapping_sub(cx * CHUNK_SIZE_I32) as usize;
        let ly = y.wrapping_sub(cy * CHUNK_SIZE_I32) as usize;
        debug_assert!(lx < CHUNK_SIZE && ly < CHUNK_SIZE);
        chunk.blocks[ly][lx] = block;
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        let (cx, cy) = self.chunk_at(x, y);
        self.chunks.contains_key(&(cx, cy))
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, &Block),
    {
        for (&(cx, cy), chunk) in &self.chunks {
            let base_x = cx * CHUNK_SIZE_I32;
            let base_y = cy * CHUNK_SIZE_I32;
            for ly in 0..CHUNK_SIZE {
                for lx in 0..CHUNK_SIZE {
                    let wx = base_x + lx as i32;
                    let wy = base_y + ly as i32;
                    f(wx, wy, &chunk.blocks[ly][lx]);
                }
            }
        }
    }

    pub fn expand_to_chunk(&mut self, cx: i32, cy: i32) {
        self.chunks.entry((cx, cy)).or_insert_with(Chunk::new);
    }

    pub fn for_each_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(i32, i32, &mut Block),
    {
        let keys: Vec<(i32, i32)> = self.chunks.keys().copied().collect();
        for &(cx, cy) in &keys {
            let base_x = cx * CHUNK_SIZE_I32;
            let base_y = cy * CHUNK_SIZE_I32;
            if let Some(chunk) = self.chunks.get_mut(&(cx, cy)) {
                for ly in 0..CHUNK_SIZE {
                    for lx in 0..CHUNK_SIZE {
                        let wx = base_x + lx as i32;
                        let wy = base_y + ly as i32;
                        f(wx, wy, &mut chunk.blocks[ly][lx]);
                    }
                }
            }
        }
    }

    pub fn clear(&mut self) {
        for chunk in self.chunks.values_mut() {
            for row in chunk.blocks.iter_mut() {
                for block in row.iter_mut() {
                    *block = Block::air();
                }
            }
        }
    }

    pub fn place_test_circuit(&mut self) {
        let world_w = WORLD_CHUNKS_X as i32 * CHUNK_SIZE_I32;
        let cx = world_w / 2 - 3;
        let cy = world_w / 2;

        self.set(cx, cy, Block::torch(true, false, Direction::North));
        self.set(cx + 1, cy, Block::wire());
        self.set(cx + 2, cy, Block::wire());
        self.set(cx + 3, cy, Block::repeater(Direction::East, 0, false, false));
        self.set(cx + 4, cy, Block::wire());
        self.set(cx + 5, cy, Block::wire());
        self.set(cx + 6, cy, Block::lamp());

        self.set(cx, cy + 2, Block::lever(Direction::North, false));
        self.set(cx + 1, cy + 2, Block::wire());
        self.set(cx + 2, cy + 2, Block::target());
        self.set(cx + 3, cy + 2, Block::wire());
        self.set(cx + 4, cy + 2, Block::lamp());

        let cx = world_w - 11;
        let cy = world_w / 2;
        self.set(cx, cy, Block::wire());
        self.set(cx + 1, cy, Block::solid());
        self.set(cx, cy + 1, Block::comparator(Direction::South, ComparatorMode::Compare, false));
        self.set(cx + 1, cy + 1, Block::comparator(Direction::North, ComparatorMode::Compare, false));
        self.set(cx, cy + 2, Block::wire());
        self.set(cx + 1, cy + 2, Block::wire());
        self.set(cx, cy + 3, Block::lever(Direction::North, false));
    }
}
