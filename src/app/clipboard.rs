use crate::block::*;

#[derive(Clone)]
pub struct ClipboardData {
    pub rows: Vec<Vec<Block>>,
    pub width: usize,
    pub height: usize,
}

impl super::AppState {
    pub fn copy_selection(&mut self) {
        if let (Some(s), Some(e)) = (self.select_start, self.select_end) {
            let x0 = s.0.min(e.0);
            let x1 = s.0.max(e.0);
            let y0 = s.1.min(e.1);
            let y1 = s.1.max(e.1);
            let w = (x1 - x0 + 1) as usize;
            let h = (y1 - y0 + 1) as usize;
            let mut rows = vec![vec![Block::air(); w]; h];
            for y in 0..h {
                for x in 0..w {
                    let wx = x0 + x as i32;
                    let wy = y0 + y as i32;
                    if let Some(b) = self.world.get(wx, wy) {
                        rows[y][x] = *b;
                    }
                }
            }
            self.clipboard = Some(ClipboardData {
                rows,
                width: w,
                height: h,
            });
        }
    }

    pub fn cut_selection(&mut self) {
        if let (Some(s), Some(e)) = (self.select_start, self.select_end) {
            let x0 = s.0.min(e.0);
            let x1 = s.0.max(e.0);
            let y0 = s.1.min(e.1);
            let y1 = s.1.max(e.1);
            let w = (x1 - x0 + 1) as usize;
            let h = (y1 - y0 + 1) as usize;
            let mut rows = vec![vec![Block::air(); w]; h];
            self.edit_begin();
            for y in 0..h {
                for x in 0..w {
                    let wx = x0 + x as i32;
                    let wy = y0 + y as i32;
                    if let Some(b) = self.world.get(wx, wy) {
                        rows[y][x] = *b;
                    }
                    self.set_block(wx, wy, Block::air());
                }
            }
            self.edit_end();
            self.clipboard = Some(ClipboardData {
                rows,
                width: w,
                height: h,
            });
            self.select_start = None;
            self.select_end = None;
        }
    }

    pub fn paste_clipboard(&mut self, wx: i32, wy: i32) {
        let clip = match self.clipboard.clone() {
            Some(c) => c,
            None => return,
        };
        self.edit_begin();
        for y in 0..clip.height {
            for x in 0..clip.width {
                let bx = wx + x as i32;
                let by = wy + y as i32;
                if self.world.in_bounds(bx, by) && clip.rows[y][x].id != BlockId::Air {
                    self.set_block(bx, by, clip.rows[y][x]);
                }
            }
        }
        self.edit_end();
    }

    pub fn get_selection_size(&self) -> Option<(i32, i32)> {
        let (s, e) = (self.select_start?, self.select_end?);
        let x0 = s.0.min(e.0);
        let x1 = s.0.max(e.0);
        let y0 = s.1.min(e.1);
        let y1 = s.1.max(e.1);
        Some((x1 - x0 + 1, y1 - y0 + 1))
    }
}
