mod palette;
mod blocks;

pub use palette::SelectBlock;
pub use palette::draw_ui_palette;
pub use palette::hit_test_palette;
pub use palette::draw_hover_tooltip;
pub use palette::draw_coordinates;

use macroquad::prelude::*;
use crate::constants::{CELL_SIZE, CHECKER_EVEN_COLOR, CHECKER_ODD_COLOR, CHUNK_GRID_COLOR, CHUNK_SIZE, CULLING_MARGIN, UNLOADED_CHUNK_COLOR};
use crate::world::World;
use blocks::draw_block;

pub struct Camera {
    pub offset_x: f32,
    pub offset_y: f32,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Camera { offset_x: 0.0, offset_y: 0.0, zoom: 1.0 }
    }

    pub fn world_to_screen(&self, wx: i32, wy: i32) -> (f32, f32) {
        (wx as f32 * CELL_SIZE * self.zoom + self.offset_x,
         wy as f32 * CELL_SIZE * self.zoom + self.offset_y)
    }

    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (i32, i32) {
        (((sx - self.offset_x) / (CELL_SIZE * self.zoom)).floor() as i32,
         ((sy - self.offset_y) / (CELL_SIZE * self.zoom)).floor() as i32)
    }

    pub fn screen_to_world_f32(&self, sx: f32, sy: f32) -> (f32, f32) {
        ((sx - self.offset_x) / (CELL_SIZE * self.zoom),
         (sy - self.offset_y) / (CELL_SIZE * self.zoom))
    }

    pub fn cell_size(&self) -> f32 { CELL_SIZE * self.zoom }
}

fn rgba(c: (u8, u8, u8, u8)) -> Color {
    Color::from_rgba(c.0, c.1, c.2, c.3)
}

pub fn draw_world(world: &World, camera: &Camera, screen_w: f32, screen_h: f32) {
    let cs = camera.cell_size();
    let (min_wx, min_wy) = camera.screen_to_world(0.0, 0.0);
    let (max_wx, max_wy) = camera.screen_to_world(screen_w, screen_h);

    let draw_min_x = min_wx;
    let draw_min_y = min_wy;
    let draw_max_x = max_wx + 1;
    let draw_max_y = max_wy + 1;

    for wy in draw_min_y..draw_max_y {
        for wx in draw_min_x..draw_max_x {
            let (sx, sy) = camera.world_to_screen(wx, wy);
            if sx + cs < -CULLING_MARGIN || sx > screen_w + CULLING_MARGIN || sy + cs < -CULLING_MARGIN || sy > screen_h + CULLING_MARGIN {
                continue;
            }

            if world.in_bounds(wx, wy) {
                if (wx + wy) % 2 == 0 {
                    draw_rectangle(sx, sy, cs, cs, rgba(CHECKER_EVEN_COLOR));
                } else {
                    draw_rectangle(sx, sy, cs, cs, rgba(CHECKER_ODD_COLOR));
                }
                if let Some(block) = world.get(wx, wy) {
                    draw_block(world, wx, wy, block, sx, sy, cs);
                }
            } else {
                draw_rectangle(sx, sy, cs, cs, rgba(UNLOADED_CHUNK_COLOR));
            }
        }
    }

    draw_chunk_grid(camera, draw_min_x, draw_min_y, draw_max_x, draw_max_y, world);
}

fn draw_chunk_grid(camera: &Camera, min_wx: i32, min_wy: i32, max_wx: i32, max_wy: i32, _world: &World) {
    let chunk = CHUNK_SIZE as i32;
    let sw = screen_width();
    let sh = screen_height();

    let (_, top_sy) = camera.world_to_screen(min_wx, min_wy);
    let (_, bot_sy) = camera.world_to_screen(min_wx, max_wy);
    let top_sy = top_sy.max(-CULLING_MARGIN).min(sh + CULLING_MARGIN);
    let bot_sy = bot_sy.max(-CULLING_MARGIN).min(sh + CULLING_MARGIN);

    let first_cx = min_wx.div_euclid(chunk);
    let last_cx = (max_wx - 1).div_euclid(chunk);
    for cx in first_cx..=last_cx {
        let wx = cx * chunk;
        let (sx, _) = camera.world_to_screen(wx, min_wy);
        if sx < -CULLING_MARGIN || sx > sw + CULLING_MARGIN { continue; }
        draw_line(sx, top_sy, sx, bot_sy, 1.0, rgba(CHUNK_GRID_COLOR));
    }

    let (left_sx, _) = camera.world_to_screen(min_wx, min_wy);
    let (right_sx, _) = camera.world_to_screen(max_wx, min_wy);
    let left_sx = left_sx.max(-CULLING_MARGIN).min(sw + CULLING_MARGIN);
    let right_sx = right_sx.max(-CULLING_MARGIN).min(sw + CULLING_MARGIN);

    let first_cy = min_wy.div_euclid(chunk);
    let last_cy = (max_wy - 1).div_euclid(chunk);
    for cy in first_cy..=last_cy {
        let wy = cy * chunk;
        let (_, sy) = camera.world_to_screen(min_wx, wy);
        if sy < -CULLING_MARGIN || sy > sh + CULLING_MARGIN { continue; }
        draw_line(left_sx, sy, right_sx, sy, 1.0, rgba(CHUNK_GRID_COLOR));
    }
}
