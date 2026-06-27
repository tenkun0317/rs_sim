use macroquad::prelude::*;
use crate::block::*;
use crate::world::World;
use crate::sim;

pub const CELL_SIZE: f32 = 40.0;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectBlock {
    RedstoneWire, RedstoneTorch, RedstoneBlock, Repeater, Comparator,
    Lever, Button, SolidBlock, RedstoneLamp, Target, Barrel, Eraser,
}

impl SelectBlock {
    pub const ALL: [SelectBlock; 12] = [
        SelectBlock::RedstoneWire, SelectBlock::RedstoneTorch, SelectBlock::RedstoneBlock,
        SelectBlock::Repeater, SelectBlock::Comparator, SelectBlock::Lever, SelectBlock::Button,
        SelectBlock::SolidBlock, SelectBlock::RedstoneLamp, SelectBlock::Target,
        SelectBlock::Barrel,
        SelectBlock::Eraser,
    ];

    pub fn to_block(self, cursor_dir: Direction) -> Block {
        match self {
            SelectBlock::RedstoneWire => Block::wire(),
            SelectBlock::RedstoneTorch => Block::torch(true, true, cursor_dir),
            SelectBlock::RedstoneBlock => Block::redstone_block(),
            SelectBlock::Repeater => Block::repeater(cursor_dir, 0, false, false),
            SelectBlock::Comparator => Block::comparator(cursor_dir, ComparatorMode::Compare, false),
            SelectBlock::Lever => Block::lever(cursor_dir, false),
            SelectBlock::Button => Block::button(cursor_dir, false),
            SelectBlock::SolidBlock => Block::solid(),
            SelectBlock::RedstoneLamp => Block::lamp(),
            SelectBlock::Target => Block::target(),
            SelectBlock::Barrel => Block::barrel(0),
            SelectBlock::Eraser => Block::air(),
        }
    }

    fn palette_color(&self) -> Color {
        match self {
            SelectBlock::RedstoneWire => Color::from_rgba(200, 30, 30, 255),
            SelectBlock::RedstoneTorch => Color::from_rgba(255, 120, 20, 255),
            SelectBlock::RedstoneBlock => Color::from_rgba(180, 30, 30, 255),
            SelectBlock::Repeater => Color::from_rgba(160, 140, 120, 255),
            SelectBlock::Comparator => Color::from_rgba(150, 150, 160, 255),
            SelectBlock::Lever => Color::from_rgba(120, 100, 80, 255),
            SelectBlock::Button => Color::from_rgba(140, 140, 100, 255),
            SelectBlock::SolidBlock => Color::from_rgba(100, 100, 100, 255),
            SelectBlock::RedstoneLamp => Color::from_rgba(200, 200, 80, 255),
            SelectBlock::Target => Color::from_rgba(200, 80, 80, 255),
            SelectBlock::Barrel => Color::from_rgba(120, 90, 60, 255),
            SelectBlock::Eraser => Color::from_rgba(200, 60, 60, 255),
        }
    }
}

pub fn draw_world(world: &World, camera: &Camera, screen_w: f32, screen_h: f32) {
    let cs = camera.cell_size();
    let (min_wx, min_wy) = camera.screen_to_world(0.0, 0.0);
    let (max_wx, max_wy) = camera.screen_to_world(screen_w, screen_h);

    let draw_min_x = min_wx;
    let draw_min_y = min_wy;
    let draw_max_x = max_wx + 1;
    let draw_max_y = max_wy + 1;

    // Draw unloaded chunk areas (outside world bounds)
    let unloaded_color = Color::from_rgba(18, 18, 22, 255);

    for wy in draw_min_y..draw_max_y {
        for wx in draw_min_x..draw_max_x {
            let (sx, sy) = camera.world_to_screen(wx, wy);
            if sx + cs < -10.0 || sx > screen_w + 10.0 || sy + cs < -10.0 || sy > screen_h + 10.0 {
                continue;
            }

            if world.in_bounds(wx, wy) {
                // Inside world: draw block
                if (wx + wy) % 2 == 0 {
                    draw_rectangle(sx, sy, cs, cs, Color::from_rgba(35, 35, 35, 255));
                } else {
                    draw_rectangle(sx, sy, cs, cs, Color::from_rgba(28, 28, 28, 255));
                }
                if let Some(block) = world.get(wx, wy) {
                    draw_block(world, wx, wy, block, sx, sy, cs);
                }
            } else {
                // Outside world: draw unloaded chunk area
                draw_rectangle(sx, sy, cs, cs, unloaded_color);
            }
        }
    }

    draw_chunk_grid(camera, draw_min_x, draw_min_y, draw_max_x, draw_max_y, world);
}

fn draw_block(world: &World, wx: i32, wy: i32, block: &Block, sx: f32, sy: f32, cs: f32) {
    let inset = cs * 0.05;
    let mid = cs * 0.5;

    match block.id {
        BlockId::Air => {}

        BlockId::SolidBlock => {
            let c = Color::from_rgba(110, 110, 120, 255);
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, c);
            draw_rectangle_lines(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, 1.0, Color::from_rgba(140, 140, 150, 255));
            if block.power > 0 {
                let fs = (cs * 0.3).max(7.0);
                draw_text(format!("{}", block.power), sx + mid - fs * 0.2, sy + mid + fs * 0.3, fs, Color::from_rgba(255, 200, 100, 220));
            }
        }

        BlockId::RedstoneWire => {
            let intensity = block.power as f32 / 15.0;
            let r = (80.0 + 175.0 * intensity) as u8;
            let g = (10.0 + 30.0 * intensity) as u8;
            let b = (10.0 + 30.0 * intensity) as u8;
            let color = Color::from_rgba(r, g, b, 255);

            let (lx, ly) = (wx - world.offset_x, wy - world.offset_y);
            let conn = |d| sim::wire_connects_in_dir(world, lx, ly, d);
            let (cn, cs_dir, ce, cw) = (conn(Direction::North), conn(Direction::South), conn(Direction::East), conn(Direction::West));

            let thin = cs * 0.20;
            let _thick = cs * 0.32;

            if cn && cs_dir && !ce && !cw {
                draw_rectangle(sx + mid - thin, sy, thin * 2.0, cs, color);
            } else if ce && cw && !cn && !cs_dir {
                draw_rectangle(sx, sy + mid - thin, cs, thin * 2.0, color);
            } else if cn && ce && !cs_dir && !cw {
                draw_rectangle(sx + mid - thin, sy, thin * 2.0, mid + thin, color);
                draw_rectangle(sx + mid - thin, sy + mid - thin, mid + thin, thin * 2.0, color);
            } else if cn && cw && !cs_dir && !ce {
                draw_rectangle(sx + mid - thin, sy, thin * 2.0, mid + thin, color);
                draw_rectangle(sx, sy + mid - thin, mid + thin, thin * 2.0, color);
            } else if cs_dir && ce && !cn && !cw {
                draw_rectangle(sx + mid - thin, sy + mid - thin, thin * 2.0, mid + thin, color);
                draw_rectangle(sx + mid - thin, sy + mid - thin, mid + thin, thin * 2.0, color);
            } else if cs_dir && cw && !cn && !ce {
                draw_rectangle(sx + mid - thin, sy + mid - thin, thin * 2.0, mid + thin, color);
                draw_rectangle(sx, sy + mid - thin, mid + thin, thin * 2.0, color);
            } else if cn && cs_dir && ce && !cw {
                draw_rectangle(sx + mid - thin, sy, thin * 2.0, cs, color);
                draw_rectangle(sx + mid - thin, sy + mid - thin, mid + thin, thin * 2.0, color);
            } else if cn && cs_dir && cw && !ce {
                draw_rectangle(sx + mid - thin, sy, thin * 2.0, cs, color);
                draw_rectangle(sx, sy + mid - thin, mid + thin, thin * 2.0, color);
            } else if ce && cw && cn && !cs_dir {
                draw_rectangle(sx, sy + mid - thin, cs, thin * 2.0, color);
                draw_rectangle(sx + mid - thin, sy, thin * 2.0, mid + thin, color);
            } else if ce && cw && cs_dir && !cn {
                draw_rectangle(sx, sy + mid - thin, cs, thin * 2.0, color);
                draw_rectangle(sx + mid - thin, sy + mid - thin, thin * 2.0, mid + thin, color);
            } else if cn && cs_dir && ce && cw {
                draw_rectangle(sx + mid - thin, sy, thin * 2.0, cs, color);
                draw_rectangle(sx, sy + mid - thin, cs, thin * 2.0, color);
            } else {
                let count = [cn, cs_dir, ce, cw].iter().filter(|&&x| x).count();
                if count == 1 && (cn || cs_dir) {
                    draw_rectangle(sx + mid - thin, sy, thin * 2.0, cs, color);
                } else if count == 1 {
                    draw_rectangle(sx, sy + mid - thin, cs, thin * 2.0, color);
                } else {
                    let dot = cs * 0.12;
                    draw_rectangle(sx + mid - dot, sy + mid - dot, dot * 2.0, dot * 2.0, color);
                }
            }

            if block.power > 0 {
                let fs = (cs * 0.35).max(7.0);
                draw_text(format!("{}", block.power), sx + mid - fs * 0.2, sy + mid + fs * 0.35, fs, WHITE);
            }
        }

        BlockId::RedstoneTorch => {
            let lit = decode_torch_lit(block.data);
            let on_wall = decode_torch_on_wall(block.data);
            let dir = decode_torch_dir(block.data);
            let body = if lit { Color::from_rgba(255, 130, 20, 255) } else { Color::from_rgba(80, 40, 0, 255) };

            if on_wall {
                let attach = dir;
                let facing = attach.opposite();
                let cx = mid + facing.dx() as f32 * cs * 0.2;
                let cy = mid + facing.dy() as f32 * cs * 0.2;
                let sw = cs * 0.08;
                draw_rectangle(sx + mid - sw * 0.5, sy + cs * 0.25, sw, cs * 0.4, body);
                let hr = cs * 0.16;
                draw_circle(sx + cx, sy + cy, hr, body);
                if lit {
                    draw_circle(sx + cx, sy + cy, hr * 0.55, Color::from_rgba(255, 220, 80, 200));
                }
                draw_rectangle(sx + mid - cs * 0.02, sy + cs * 0.25, cs * 0.04, cs * 0.5, Color::from_rgba(50, 30, 20, 255));
            } else {
                draw_rectangle(sx + mid - cs * 0.05, sy + cs * 0.3, cs * 0.1, cs * 0.5, body);
                let hr = cs * 0.18;
                draw_circle(sx + mid, sy + cs * 0.25, hr, body);
                if lit {
                    draw_circle(sx + mid, sy + cs * 0.25, hr * 0.6, Color::from_rgba(255, 220, 80, 200));
                }
            }
        }

        BlockId::RedstoneBlock => {
            let c = Color::from_rgba(180, 25, 25, 255);
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, c);
            draw_rectangle_lines(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, 1.0, Color::from_rgba(210, 50, 50, 255));
        }

        BlockId::Repeater => {
            let dir = decode_repeater_dir(block.data);
            let powered = decode_repeater_powered(block.data);
            let delay = decode_repeater_delay(block.data);

            let bg = if powered { Color::from_rgba(200, 80, 80, 255) } else { Color::from_rgba(130, 120, 110, 255) };
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, bg);

            let len = cs * 0.3;
            let (bkx, bky) = (mid - dir.dx() as f32 * len * 0.5, mid - dir.dy() as f32 * len * 0.5);
            let (tkx, tky) = (mid + dir.dx() as f32 * len, mid + dir.dy() as f32 * len);
            draw_line(sx + bkx, sy + bky, sx + tkx, sy + tky, cs * 0.08, DARKGRAY);
            let perp = cs * 0.07;
            let ndx = -dir.dy() as f32;
            let ndy = dir.dx() as f32;
            draw_line(sx + tkx, sy + tky, sx + tkx - dir.dx() as f32 * perp + ndx * perp, sy + tky - dir.dy() as f32 * perp + ndy * perp, cs * 0.06, DARKGRAY);
            draw_line(sx + tkx, sy + tky, sx + tkx - dir.dx() as f32 * perp - ndx * perp, sy + tky - dir.dy() as f32 * perp - ndy * perp, cs * 0.06, DARKGRAY);

            let dt = format!("{}d", delay + 1);
            let fs = (cs * 0.28).max(7.0);
            draw_text(&dt, sx + cs * 0.35, sy + cs * 0.85, fs, WHITE);

            if powered {
                let pfs = (cs * 0.3).max(7.0);
                draw_text("15", sx + cs * 0.05, sy + cs * 0.35, pfs, Color::from_rgba(255, 200, 100, 200));
            }
        }

        BlockId::Comparator => {
            let dir = decode_comparator_dir(block.data);
            let mode = decode_comparator_mode(block.data);
            let powered = decode_comparator_powered(block.data);

            let bg = if powered { Color::from_rgba(200, 100, 60, 255) } else { Color::from_rgba(130, 130, 140, 255) };
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, bg);

            let len = cs * 0.25;
            let (bkx, bky) = (mid - dir.dx() as f32 * len, mid - dir.dy() as f32 * len);
            let (tkx, tky) = (mid + dir.dx() as f32 * len, mid + dir.dy() as f32 * len);
            draw_line(sx + bkx, sy + bky, sx + tkx, sy + tky, cs * 0.08, DARKGRAY);

            let fx = sx + mid + dir.dx() as f32 * (len + cs * 0.15);
            let fy = sy + mid + dir.dy() as f32 * (len + cs * 0.15);
            draw_circle(fx, fy, cs * 0.08, DARKGRAY);

            let mode_char = if mode == ComparatorMode::Compare { "C" } else { "S" };
            let mode_color = if mode == ComparatorMode::Compare {
                Color::from_rgba(100, 200, 255, 255)
            } else {
                Color::from_rgba(255, 200, 100, 255)
            };
            let mfs = (cs * 0.35).max(8.0);
            draw_text(mode_char, sx + cs * 0.35, sy + cs * 0.85, mfs, mode_color);

            if block.power > 0 {
                let pfs = (cs * 0.25).max(6.0);
                draw_text(format!("{}", block.power), sx + cs * 0.05, sy + cs * 0.3, pfs, Color::from_rgba(255, 220, 150, 200));
            }
        }

        BlockId::Lever => {
            let dir = decode_lever_dir(block.data);
            let powered = decode_lever_powered(block.data);
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, Color::from_rgba(70, 70, 70, 255));

            let stick = cs * 0.35;
            let d = if powered { dir.opposite() } else { dir };
            draw_line(sx + mid, sy + mid, sx + mid + d.dx() as f32 * stick, sy + mid + d.dy() as f32 * stick, cs * 0.07, Color::from_rgba(140, 100, 60, 255));
            draw_circle(sx + mid + d.dx() as f32 * stick, sy + mid + d.dy() as f32 * stick, cs * 0.12,
                if powered { Color::from_rgba(255, 50, 50, 255) } else { Color::from_rgba(100, 100, 100, 255) });
        }

        BlockId::Button => {
            let powered = ((block.data >> 2) & 1) != 0;
            let bg = if powered { Color::from_rgba(200, 200, 80, 255) } else { Color::from_rgba(100, 100, 80, 255) };
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, bg);
            let b = cs * 0.15;
            draw_rectangle(sx + mid - b, sy + mid - b, b * 2.0, b * 2.0, Color::from_rgba(60, 60, 60, 255));
        }

        BlockId::RedstoneLamp => {
            let lit = decode_lamp_lit(block.data);
            let c = if lit { Color::from_rgba(255, 255, 120, 255) } else { Color::from_rgba(80, 80, 40, 255) };
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, c);
            if lit {
                draw_rectangle_lines(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, 2.0, Color::from_rgba(255, 255, 180, 200));
            }
        }

            BlockId::Target => {
                let c = Color::from_rgba(180, 100, 100, 255);
                draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, c);
                let ring_r = cs * 0.3;
                let ring_w = cs * 0.06;
                draw_circle(sx + mid, sy + mid, ring_r, Color::from_rgba(120, 60, 60, 255));
                draw_circle(sx + mid, sy + mid, ring_r - ring_w, c);
                draw_circle(sx + mid, sy + mid, cs * 0.08, Color::from_rgba(60, 30, 30, 255));
                draw_rectangle_lines(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, 1.0, Color::from_rgba(200, 120, 120, 255));
            }

            BlockId::Barrel => {
                let c = Color::from_rgba(140, 100, 70, 255);
                let dark = Color::from_rgba(100, 70, 50, 255);
                draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, c);
                draw_rectangle(sx + cs * 0.15, sy + cs * 0.1, cs * 0.7, cs * 0.12, dark);
                draw_rectangle(sx + cs * 0.15, sy + cs * 0.78, cs * 0.7, cs * 0.12, dark);
                let strength = decode_barrel_strength(block.data);
                let fs = (cs * 0.4).max(9.0);
                draw_text(format!("{}", strength), sx + mid - fs * 0.25, sy + mid + fs * 0.35, fs, Color::from_rgba(255, 220, 150, 220));
            }
    }
}

fn draw_chunk_grid(camera: &Camera, min_wx: i32, min_wy: i32, max_wx: i32, max_wy: i32, _world: &World) {
    let chunk = World::CHUNK_SIZE as i32;
    let line_color = Color::from_rgba(50, 50, 55, 255);
    let sw = screen_width();
    let sh = screen_height();

    // Compute screen Y range for vertical lines
    let (_, top_sy) = camera.world_to_screen(min_wx, min_wy);
    let (_, bot_sy) = camera.world_to_screen(min_wx, max_wy);
    let top_sy = top_sy.max(-10.0).min(sh + 10.0);
    let bot_sy = bot_sy.max(-10.0).min(sh + 10.0);

    // Draw chunk column lines
    let first_cx = min_wx.div_euclid(chunk);
    let last_cx = (max_wx - 1).div_euclid(chunk);
    for cx in first_cx..=last_cx {
        let wx = cx * chunk;
        let (sx, _) = camera.world_to_screen(wx, min_wy);
        if sx < -10.0 || sx > sw + 10.0 { continue; }
        draw_line(sx, top_sy, sx, bot_sy, 1.0, line_color);
    }

    // Compute screen X range for horizontal lines
    let (left_sx, _) = camera.world_to_screen(min_wx, min_wy);
    let (right_sx, _) = camera.world_to_screen(max_wx, min_wy);
    let left_sx = left_sx.max(-10.0).min(sw + 10.0);
    let right_sx = right_sx.max(-10.0).min(sw + 10.0);

    // Draw chunk row lines
    let first_cy = min_wy.div_euclid(chunk);
    let last_cy = (max_wy - 1).div_euclid(chunk);
    for cy in first_cy..=last_cy {
        let wy = cy * chunk;
        let (_, sy) = camera.world_to_screen(min_wx, wy);
        if sy < -10.0 || sy > sh + 10.0 { continue; }
        draw_line(left_sx, sy, right_sx, sy, 1.0, line_color);
    }
}

pub fn draw_ui_palette(selected: SelectBlock, screen_w: f32, screen_h: f32) {
    let bar_h = 60.0;
    let bar_y = screen_h - bar_h;
    draw_rectangle(0.0, bar_y, screen_w, bar_h, Color::from_rgba(35, 35, 40, 255));
    draw_line(0.0, bar_y, screen_w, bar_y, 2.0, Color::from_rgba(70, 70, 80, 255));

    let items = SelectBlock::ALL;
    let count = items.len() as f32;
    let total_gap = 6.0 * (count + 1.0);
    let item_w = ((screen_w - total_gap) / count).max(40.0);
    let item_h = bar_h - 10.0;

    for (i, item) in items.iter().enumerate() {
        let ix = 6.0 + i as f32 * (item_w + 6.0);
        let iy = bar_y + 5.0;
        let bg = if *item == selected { Color::from_rgba(70, 80, 120, 255) } else { Color::from_rgba(45, 45, 50, 255) };
        draw_rectangle(ix, iy, item_w, item_h, bg);

        let color = item.palette_color();
        let preview = (item_w * 0.5).min(item_h * 0.45);
        draw_rectangle(ix + item_w * 0.5 - preview * 0.5, iy + 3.0, preview, preview * 0.7, color);

        let label = match item {
            SelectBlock::RedstoneWire => "Wire",
            SelectBlock::RedstoneTorch => "Torch",
            SelectBlock::RedstoneBlock => "RBlk",
            SelectBlock::Repeater => "Rep",
            SelectBlock::Comparator => "Cmp",
            SelectBlock::Lever => "Lvr",
            SelectBlock::Button => "Btn",
            SelectBlock::SolidBlock => "Blk",
            SelectBlock::RedstoneLamp => "Lmp",
            SelectBlock::Target => "Tgt",
            SelectBlock::Barrel => "Brl",
            SelectBlock::Eraser => "Erase",
        };
        draw_text(label, ix + 2.0, iy + item_h - 4.0, 10.0, Color::from_rgba(180, 180, 180, 255));

        let key_label = match i {
            0 => "1", 1 => "2", 2 => "3", 3 => "4", 4 => "5",
            5 => "6", 6 => "7", 7 => "8", 8 => "9", 9 => "0",
            _ => "",
        };
        if !key_label.is_empty() {
            draw_text(key_label, ix + item_w - 12.0, iy + 10.0, 9.0, Color::from_rgba(120, 120, 140, 200));
        }
    }
}

pub fn hit_test_palette(mx: f32, my: f32, screen_w: f32, screen_h: f32) -> Option<SelectBlock> {
    let bar_h = 60.0;
    let bar_y = screen_h - bar_h;
    if my < bar_y || my > screen_h { return None; }
    if mx < 0.0 || mx > screen_w { return None; }

    let items = SelectBlock::ALL;
    let count = items.len() as f32;
    let total_gap = 6.0 * (count + 1.0);
    let item_w = ((screen_w - total_gap) / count).max(40.0);

    let idx = ((mx - 6.0) / (item_w + 6.0)) as usize;
    if idx < items.len() { Some(items[idx]) } else { None }
}

pub fn draw_hover_tooltip(world: &World, camera: &Camera, mx: f32, my: f32, screen_w: f32, screen_h: f32) {
    let bar_h = 60.0;
    if my >= screen_h - bar_h { return; }

    let (wx, wy) = camera.screen_to_world(mx, my);
    let Some(block) = world.get(wx, wy) else { return; };

    let info = format!("({},{}) {} pwr:{}", wx, wy, block.display_name(), block.power);
    let font_size = 14.0;
    let tw = measure_text(&info, None, font_size as u16, 1.0).width;
    let tx = (mx - tw * 0.5).clamp(2.0, screen_w - tw - 4.0);
    let ty = my - 22.0;

    draw_rectangle(tx - 2.0, ty - font_size - 2.0, tw + 4.0, font_size + 4.0, Color::from_rgba(0, 0, 0, 200));
    draw_text(&info, tx, ty, font_size, WHITE);
}

pub fn draw_coordinates(camera: &Camera, mx: f32, my: f32, _screen_w: f32, _screen_h: f32) {
    let (wx, wy) = camera.screen_to_world(mx, my);
    draw_text(format!("({}, {})", wx, wy), 5.0, 15.0, 14.0, Color::from_rgba(180, 180, 180, 255));
}
