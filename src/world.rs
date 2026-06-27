use crate::block::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct World {
    pub width: usize,
    pub height: usize,
    pub blocks: Vec<Block>,
    #[serde(default)]
    pub offset_x: i32,
    #[serde(default)]
    pub offset_y: i32,
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        World {
            width,
            height,
            blocks: vec![Block::air(); width * height],
            offset_x: 0,
            offset_y: 0,
        }
    }

    pub fn idx(&self, x: i32, y: i32) -> Option<usize> {
        let lx = x.checked_sub(self.offset_x)?;
        let ly = y.checked_sub(self.offset_y)?;
        if lx >= 0 && ly >= 0 && (lx as usize) < self.width && (ly as usize) < self.height {
            Some(ly as usize * self.width + lx as usize)
        } else {
            None
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&Block> {
        self.idx(x, y).map(|i| &self.blocks[i])
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut Block> {
        self.idx(x, y).map(move |i| &mut self.blocks[i])
    }

    pub fn get_local(&self, lx: i32, ly: i32) -> Option<&Block> {
        if lx >= 0 && (lx as usize) < self.width && ly >= 0 && (ly as usize) < self.height {
            Some(&self.blocks[ly as usize * self.width + lx as usize])
        } else {
            None
        }
    }

    pub fn get_mut_local(&mut self, lx: i32, ly: i32) -> Option<&mut Block> {
        if lx >= 0 && (lx as usize) < self.width && ly >= 0 && (ly as usize) < self.height {
            Some(&mut self.blocks[ly as usize * self.width + lx as usize])
        } else {
            None
        }
    }

    pub fn in_bounds_local(&self, lx: i32, ly: i32) -> bool {
        lx >= 0 && (lx as usize) < self.width && ly >= 0 && (ly as usize) < self.height
    }

    pub const CHUNK_SIZE: usize = 16;

    pub fn chunk_at(&self, x: i32, y: i32) -> (i32, i32) {
        (x.div_euclid(Self::CHUNK_SIZE as i32), y.div_euclid(Self::CHUNK_SIZE as i32))
    }

    pub fn expand_to_chunk(&mut self, cx: i32, cy: i32) {
        let cs = Self::CHUNK_SIZE as i32;
        let target_min_x = cx * cs;
        let target_min_y = cy * cs;
        let target_max_x = (cx + 1) * cs;
        let target_max_y = (cy + 1) * cs;

        let cur_min_x = self.offset_x;
        let cur_min_y = self.offset_y;
        let cur_max_x = self.offset_x + self.width as i32;
        let cur_max_y = self.offset_y + self.height as i32;

        if target_min_x >= cur_min_x && target_max_x <= cur_max_x
            && target_min_y >= cur_min_y && target_max_y <= cur_max_y
        { return; }

        let need_left = if target_min_x < cur_min_x { (cur_min_x - target_min_x) as usize } else { 0 };
        let need_top = if target_min_y < cur_min_y { (cur_min_y - target_min_y) as usize } else { 0 };
        let need_right = if target_max_x > cur_max_x { (target_max_x - cur_max_x) as usize } else { 0 };
        let need_bot = if target_max_y > cur_max_y { (target_max_y - cur_max_y) as usize } else { 0 };

        let new_w = self.width + need_left + need_right;
        let new_h = self.height + need_top + need_bot;
        self.offset_x -= need_left as i32;
        self.offset_y -= need_top as i32;

        let mut new_blocks = vec![Block::air(); new_w * new_h];
        for row in 0..self.height {
            let src = row * self.width;
            let dst = (row + need_top) * new_w + need_left;
            new_blocks[dst..dst + self.width].copy_from_slice(&self.blocks[src..src + self.width]);
        }
        self.width = new_w;
        self.height = new_h;
        self.blocks = new_blocks;
    }

    pub fn set(&mut self, x: i32, y: i32, block: Block) {
        if let Some(idx) = self.idx(x, y) {
            self.blocks[idx] = block;
        }
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        let lx = x - self.offset_x;
        let ly = y - self.offset_y;
        lx >= 0 && (lx as usize) < self.width && ly >= 0 && (ly as usize) < self.height
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, &Block),
    {
        for y in 0..self.height {
            for x in 0..self.width {
                f(x as i32, y as i32, &self.blocks[y * self.width + x]);
            }
        }
    }

    pub fn for_each_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(i32, i32, &mut Block),
    {
        for y in 0..self.height {
            for x in 0..self.width {
                f(x as i32, y as i32, &mut self.blocks[y * self.width + x]);
            }
        }
    }

    pub fn clear(&mut self) {
        for block in self.blocks.iter_mut() {
            *block = Block::air();
        }
    }

    pub fn place_test_circuit(&mut self) {
        let cx = self.width as i32 / 2 - 3;
        let cy = self.height as i32 / 2;

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

        let cx = self.width as i32 - 11;
        let cy = self.height as i32 / 2;
        self.set(cx, cy, Block::wire());
        self.set(cx + 1, cy, Block::solid());
        self.set(cx, cy + 1, Block::comparator(Direction::South, ComparatorMode::Compare, false));
        self.set(cx + 1, cy + 1, Block::comparator(Direction::North, ComparatorMode::Compare, false));
        self.set(cx, cy + 2, Block::wire());
        self.set(cx + 1, cy + 2, Block::wire());
        self.set(cx, cy + 3, Block::lever(Direction::North, false));
    }
}
