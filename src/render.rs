use macroquad::prelude::*;
use crate::block::*;
use crate::constants::{
    BAR_BG_COLOR, BAR_LINE_COLOR, BARREL_FILL_COLOR, BARREL_STRIPE_COLOR, BARREL_TEXT_COLOR, BLOCK_INSET_FRAC, BLOCK_OUTLINE_WIDTH,
    BUTTON_INNER_COLOR, BUTTON_OFF_COLOR, BUTTON_ON_COLOR, BUTTON_POWERED_SHIFT, CELL_SIZE,
    CHECKER_EVEN_COLOR, CHECKER_ODD_COLOR, CHUNK_GRID_COLOR, CHUNK_SIZE, COMPARATOR_COMPARE_COLOR,
    COMPARATOR_OFF_COLOR, COMPARATOR_ON_COLOR, COMPARATOR_POWER_COLOR, COMPARATOR_SUBTRACT_COLOR,
    COORD_FONT_SIZE, COORD_TEXT_COLOR, COORD_X, COORD_Y, CULLING_MARGIN, LAMP_OFF_COLOR,
    LAMP_ON_COLOR, LAMP_ON_OUTLINE_COLOR, LEVER_BG_COLOR, LEVER_KNOB_OFF_COLOR, LEVER_KNOB_ON_COLOR,
    LEVER_STICK_COLOR, MIN_FONT_SIZE,     PALETTE_FONT_SIZE, PALETTE_GAP, PALETTE_ITEM_HEIGHT_OFFSET,
    PALETTE_ITEM_INSET_Y, PALETTE_KEY_COLOR, PALETTE_KEY_FONT_SIZE, PALETTE_MIN_ITEM_WIDTH,
    PALETTE_SELECTED_COLOR, PALETTE_SWATCH_FRAC, PALETTE_SWATCH_H_FRAC,
    PALETTE_SWATCH_HEIGHT_FRAC, PALETTE_SWATCH_Y_OFFSET, PALETTE_TEXT_COLOR, PALETTE_UNSELECTED_COLOR,
    REDSTONE_BLOCK_FILL_COLOR, REDSTONE_BLOCK_OUTLINE_COLOR, REPEATER_OFF_COLOR, REPEATER_ON_COLOR,
    REPEATER_POWER_COLOR, SOLID_BLOCK_FILL_COLOR, SOLID_BLOCK_OUTLINE_COLOR, SOLID_POWER_COLOR,
    SOLID_POWER_TEXT_FRAC, SOLID_POWER_TEXT_MIN, TARGET_DOT_COLOR, TARGET_FILL_COLOR,
    TARGET_OUTLINE_COLOR, TARGET_RING_COLOR, TARGET_RING_W_FRAC, TARGET_RING_R_FRAC, TARGET_DOT_R_FRAC,
    TOOLTIP_BG_COLOR, TOOLTIP_FONT_SIZE, TOOLTIP_Y_OFFSET, BARREL_TEXT_FRAC, BARREL_TEXT_MIN,
    BARREL_STRIPE_X_FRAC, BARREL_STRIPE_Y_FRAC, BARREL_STRIPE_W_FRAC, BARREL_STRIPE_H_FRAC,
    BARREL_STRIPE2_Y_FRAC, BUTTON_INNER_FRAC, COMPARATOR_ARROW_LEN_FRAC, COMPARATOR_CIRCLE_OFF_FRAC,
    COMPARATOR_CIRCLE_R_FRAC, COMPARATOR_LINE_W_FRAC, COMPARATOR_TEXT_MIN, COMPARATOR_TEXT_X_FRAC,
    COMPARATOR_TEXT_Y_FRAC, COMPARATOR_POWER_TEXT_FRAC, COMPARATOR_POWER_X_FRAC, COMPARATOR_POWER_Y_FRAC,
    COMPARATOR_POWER_MIN, LEVER_STICK_LEN_FRAC, LEVER_LINE_W_FRAC, LEVER_KNOB_R_FRAC,
    REPEATER_ARROW_LEN_FRAC, REPEATER_LINE_W_FRAC, REPEATER_PERP_FRAC, REPEATER_ARROW_W_FRAC,
    REPEATER_TEXT_X_FRAC, REPEATER_TEXT_Y_FRAC, REPEATER_TEXT_MIN, REPEATER_POWER_X_FRAC,
    REPEATER_POWER_Y_FRAC, REPEATER_POWER_TEXT_FRAC, TORCH_ATTACH_OFFSET_FRAC, TORCH_BODY_LIT_COLOR,
    TORCH_BODY_UNLIT_COLOR, TORCH_FLOOR_GLOW_R_FRAC, TORCH_FLOOR_HEAD_R_FRAC, TORCH_FLOOR_STALK_H_FRAC,
    TORCH_FLOOR_STALK_W_FRAC, TORCH_GLOW_COLOR, TORCH_HEAD_INNER_R_FRAC, TORCH_HEAD_R_FRAC,
    TORCH_POST_COLOR, TORCH_POST_H_FRAC, TORCH_POST_W_FRAC, TORCH_STALK_H_FRAC, TORCH_STALK_H_START_FRAC,
    TORCH_STALK_W_FRAC, UI_BAR_HEIGHT, UNLOADED_CHUNK_COLOR, WIRE_COLOR_B_BASE, WIRE_COLOR_B_RANGE, WIRE_COLOR_G_BASE,
    WIRE_COLOR_G_RANGE, WIRE_COLOR_R_BASE, WIRE_COLOR_R_RANGE, WIRE_DOT_FRAC, WIRE_TEXT_FRAC,
    WIRE_TEXT_MIN, WIRE_THICK_FRAC, WIRE_THIN_FRAC, MAX_POWER,
};
use crate::world::World;
use crate::sim;

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

fn draw_block(world: &World, wx: i32, wy: i32, block: &Block, sx: f32, sy: f32, cs: f32) {
    let inset = cs * BLOCK_INSET_FRAC;
    let mid = cs * 0.5;

    match block.id {
        BlockId::Air => {}

        BlockId::SolidBlock => {
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, rgba(SOLID_BLOCK_FILL_COLOR));
            draw_rectangle_lines(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, BLOCK_OUTLINE_WIDTH, rgba(SOLID_BLOCK_OUTLINE_COLOR));
            if block.power > 0 {
                let fs = (cs * SOLID_POWER_TEXT_FRAC).max(SOLID_POWER_TEXT_MIN);
                draw_text(format!("{}", block.power), sx + mid - fs * 0.2, sy + mid + fs * 0.3, fs, rgba(SOLID_POWER_COLOR));
            }
        }

        BlockId::RedstoneWire => {
            let intensity = block.power as f32 / MAX_POWER as f32;
            let r = (WIRE_COLOR_R_BASE + WIRE_COLOR_R_RANGE * intensity) as u8;
            let g = (WIRE_COLOR_G_BASE + WIRE_COLOR_G_RANGE * intensity) as u8;
            let b = (WIRE_COLOR_B_BASE + WIRE_COLOR_B_RANGE * intensity) as u8;
            let color = Color::from_rgba(r, g, b, 255);

            let conn = |d| sim::wire_connects_in_dir(world, wx, wy, d);
            let (cn, cs_dir, ce, cw) = (conn(Direction::North), conn(Direction::South), conn(Direction::East), conn(Direction::West));

            let thin = cs * WIRE_THIN_FRAC;
            let _thick = cs * WIRE_THICK_FRAC;

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
                    let dot = cs * WIRE_DOT_FRAC;
                    draw_rectangle(sx + mid - dot, sy + mid - dot, dot * 2.0, dot * 2.0, color);
                }
            }

            if block.power > 0 {
                let fs = (cs * WIRE_TEXT_FRAC).max(WIRE_TEXT_MIN);
                draw_text(format!("{}", block.power), sx + mid - fs * 0.2, sy + mid + fs * 0.35, fs, WHITE);
            }
        }

        BlockId::RedstoneTorch => {
            let lit = decode_torch_lit(block.data);
            let on_wall = decode_torch_on_wall(block.data);
            let dir = decode_torch_dir(block.data);
            let body = if lit { rgba(TORCH_BODY_LIT_COLOR) } else { rgba(TORCH_BODY_UNLIT_COLOR) };

            if on_wall {
                let attach = dir;
                let facing = attach.opposite();
                let cx = mid + facing.dx() as f32 * cs * TORCH_ATTACH_OFFSET_FRAC;
                let cy = mid + facing.dy() as f32 * cs * TORCH_ATTACH_OFFSET_FRAC;
                let sw = cs * TORCH_STALK_W_FRAC;
                draw_rectangle(sx + mid - sw * 0.5, sy + cs * TORCH_STALK_H_START_FRAC, sw, cs * TORCH_STALK_H_FRAC, body);
                let hr = cs * TORCH_HEAD_R_FRAC;
                draw_circle(sx + cx, sy + cy, hr, body);
                if lit {
                    draw_circle(sx + cx, sy + cy, hr * TORCH_HEAD_INNER_R_FRAC, rgba(TORCH_GLOW_COLOR));
                }
                draw_rectangle(sx + mid - cs * TORCH_POST_W_FRAC, sy + cs * TORCH_STALK_H_START_FRAC, cs * TORCH_POST_W_FRAC * 2.0, cs * TORCH_POST_H_FRAC, rgba(TORCH_POST_COLOR));
            } else {
                draw_rectangle(sx + mid - cs * TORCH_FLOOR_STALK_W_FRAC * 0.5, sy + cs * 0.3, cs * TORCH_FLOOR_STALK_W_FRAC, cs * TORCH_FLOOR_STALK_H_FRAC, body);
                let hr = cs * TORCH_FLOOR_HEAD_R_FRAC;
                draw_circle(sx + mid, sy + cs * 0.25, hr, body);
                if lit {
                    draw_circle(sx + mid, sy + cs * 0.25, hr * TORCH_FLOOR_GLOW_R_FRAC, rgba(TORCH_GLOW_COLOR));
                }
            }
        }

        BlockId::RedstoneBlock => {
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, rgba(REDSTONE_BLOCK_FILL_COLOR));
            draw_rectangle_lines(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, BLOCK_OUTLINE_WIDTH, rgba(REDSTONE_BLOCK_OUTLINE_COLOR));
        }

        BlockId::Repeater => {
            let dir = decode_repeater_dir(block.data);
            let powered = decode_repeater_powered(block.data);
            let delay = decode_repeater_delay(block.data);

            let bg = if powered { rgba(REPEATER_ON_COLOR) } else { rgba(REPEATER_OFF_COLOR) };
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, bg);

            let len = cs * REPEATER_ARROW_LEN_FRAC;
            let (bkx, bky) = (mid - dir.dx() as f32 * len * 0.5, mid - dir.dy() as f32 * len * 0.5);
            let (tkx, tky) = (mid + dir.dx() as f32 * len, mid + dir.dy() as f32 * len);
            draw_line(sx + bkx, sy + bky, sx + tkx, sy + tky, cs * REPEATER_LINE_W_FRAC, DARKGRAY);
            let perp = cs * REPEATER_PERP_FRAC;
            let ndx = -dir.dy() as f32;
            let ndy = dir.dx() as f32;
            draw_line(sx + tkx, sy + tky, sx + tkx - dir.dx() as f32 * perp + ndx * perp, sy + tky - dir.dy() as f32 * perp + ndy * perp, cs * REPEATER_ARROW_W_FRAC, DARKGRAY);
            draw_line(sx + tkx, sy + tky, sx + tkx - dir.dx() as f32 * perp - ndx * perp, sy + tky - dir.dy() as f32 * perp - ndy * perp, cs * REPEATER_ARROW_W_FRAC, DARKGRAY);

            let dt = format!("{}d", delay + 1);
            let fs = (cs * 0.28).max(REPEATER_TEXT_MIN);
            draw_text(&dt, sx + cs * REPEATER_TEXT_X_FRAC, sy + cs * REPEATER_TEXT_Y_FRAC, fs, WHITE);

            if powered {
                let pfs = (cs * REPEATER_POWER_TEXT_FRAC).max(MIN_FONT_SIZE);
                draw_text("15", sx + cs * REPEATER_POWER_X_FRAC, sy + cs * REPEATER_POWER_Y_FRAC, pfs, rgba(REPEATER_POWER_COLOR));
            }
        }

        BlockId::Comparator => {
            let dir = decode_comparator_dir(block.data);
            let mode = decode_comparator_mode(block.data);
            let powered = decode_comparator_powered(block.data);

            let bg = if powered { rgba(COMPARATOR_ON_COLOR) } else { rgba(COMPARATOR_OFF_COLOR) };
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, bg);

            let len = cs * COMPARATOR_ARROW_LEN_FRAC;
            let (bkx, bky) = (mid - dir.dx() as f32 * len, mid - dir.dy() as f32 * len);
            let (tkx, tky) = (mid + dir.dx() as f32 * len, mid + dir.dy() as f32 * len);
            draw_line(sx + bkx, sy + bky, sx + tkx, sy + tky, cs * COMPARATOR_LINE_W_FRAC, DARKGRAY);

            let fx = sx + mid + dir.dx() as f32 * (len + cs * COMPARATOR_CIRCLE_OFF_FRAC);
            let fy = sy + mid + dir.dy() as f32 * (len + cs * COMPARATOR_CIRCLE_OFF_FRAC);
            draw_circle(fx, fy, cs * COMPARATOR_CIRCLE_R_FRAC, DARKGRAY);

            let mode_char = if mode == ComparatorMode::Compare { "C" } else { "S" };
            let mode_color = if mode == ComparatorMode::Compare {
                rgba(COMPARATOR_COMPARE_COLOR)
            } else {
                rgba(COMPARATOR_SUBTRACT_COLOR)
            };
            let mfs = (cs * 0.35).max(COMPARATOR_TEXT_MIN);
            draw_text(mode_char, sx + cs * COMPARATOR_TEXT_X_FRAC, sy + cs * COMPARATOR_TEXT_Y_FRAC, mfs, mode_color);

            if block.power > 0 {
                let pfs = (cs * COMPARATOR_POWER_TEXT_FRAC).max(COMPARATOR_POWER_MIN);
                draw_text(format!("{}", block.power), sx + cs * COMPARATOR_POWER_X_FRAC, sy + cs * COMPARATOR_POWER_Y_FRAC, pfs, rgba(COMPARATOR_POWER_COLOR));
            }
        }

        BlockId::Lever => {
            let dir = decode_lever_dir(block.data);
            let powered = decode_lever_powered(block.data);
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, rgba(LEVER_BG_COLOR));

            let stick = cs * LEVER_STICK_LEN_FRAC;
            let d = if powered { dir.opposite() } else { dir };
            draw_line(sx + mid, sy + mid, sx + mid + d.dx() as f32 * stick, sy + mid + d.dy() as f32 * stick, cs * LEVER_LINE_W_FRAC, rgba(LEVER_STICK_COLOR));
            draw_circle(sx + mid + d.dx() as f32 * stick, sy + mid + d.dy() as f32 * stick, cs * LEVER_KNOB_R_FRAC,
                if powered { rgba(LEVER_KNOB_ON_COLOR) } else { rgba(LEVER_KNOB_OFF_COLOR) });
        }

        BlockId::Button => {
            let powered = ((block.data >> BUTTON_POWERED_SHIFT) & 1) != 0;
            let bg = if powered { rgba(BUTTON_ON_COLOR) } else { rgba(BUTTON_OFF_COLOR) };
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, bg);
            let b = cs * BUTTON_INNER_FRAC;
            draw_rectangle(sx + mid - b, sy + mid - b, b * 2.0, b * 2.0, rgba(BUTTON_INNER_COLOR));
        }

        BlockId::RedstoneLamp => {
            let lit = decode_lamp_lit(block.data);
            let c = if lit { rgba(LAMP_ON_COLOR) } else { rgba(LAMP_OFF_COLOR) };
            draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, c);
            if lit {
                draw_rectangle_lines(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, BLOCK_OUTLINE_WIDTH * 2.0, rgba(LAMP_ON_OUTLINE_COLOR));
            }
        }

            BlockId::Target => {
                draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, rgba(TARGET_FILL_COLOR));
                let ring_r = cs * TARGET_RING_R_FRAC;
                let ring_w = cs * TARGET_RING_W_FRAC;
                draw_circle(sx + mid, sy + mid, ring_r, rgba(TARGET_RING_COLOR));
                draw_circle(sx + mid, sy + mid, ring_r - ring_w, rgba(TARGET_FILL_COLOR));
                draw_circle(sx + mid, sy + mid, cs * TARGET_DOT_R_FRAC, rgba(TARGET_DOT_COLOR));
                draw_rectangle_lines(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, BLOCK_OUTLINE_WIDTH, rgba(TARGET_OUTLINE_COLOR));
            }

            BlockId::Barrel => {
                draw_rectangle(sx + inset, sy + inset, cs - inset * 2.0, cs - inset * 2.0, rgba(BARREL_FILL_COLOR));
                draw_rectangle(sx + cs * BARREL_STRIPE_X_FRAC, sy + cs * BARREL_STRIPE_Y_FRAC, cs * BARREL_STRIPE_W_FRAC, cs * BARREL_STRIPE_H_FRAC, rgba(BARREL_STRIPE_COLOR));
                draw_rectangle(sx + cs * BARREL_STRIPE_X_FRAC, sy + cs * BARREL_STRIPE2_Y_FRAC, cs * BARREL_STRIPE_W_FRAC, cs * BARREL_STRIPE_H_FRAC, rgba(BARREL_STRIPE_COLOR));
                let strength = decode_barrel_strength(block.data);
                let fs = (cs * BARREL_TEXT_FRAC).max(BARREL_TEXT_MIN);
                draw_text(format!("{}", strength), sx + mid - fs * 0.25, sy + mid + fs * 0.35, fs, rgba(BARREL_TEXT_COLOR));
            }
    }
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

pub fn draw_ui_palette(selected: SelectBlock, screen_w: f32, screen_h: f32) {
    let bar_y = screen_h - UI_BAR_HEIGHT;
    draw_rectangle(0.0, bar_y, screen_w, UI_BAR_HEIGHT, rgba(BAR_BG_COLOR));
    draw_line(0.0, bar_y, screen_w, bar_y, 2.0, rgba(BAR_LINE_COLOR));

    let items = SelectBlock::ALL;
    let count = items.len() as f32;
    let total_gap = PALETTE_GAP * (count + 1.0);
    let item_w = ((screen_w - total_gap) / count).max(PALETTE_MIN_ITEM_WIDTH);
    let item_h = UI_BAR_HEIGHT - PALETTE_ITEM_HEIGHT_OFFSET;

    for (i, item) in items.iter().enumerate() {
        let ix = PALETTE_GAP + i as f32 * (item_w + PALETTE_GAP);
        let iy = bar_y + PALETTE_ITEM_INSET_Y;
        let bg = if *item == selected { rgba(PALETTE_SELECTED_COLOR) } else { rgba(PALETTE_UNSELECTED_COLOR) };
        draw_rectangle(ix, iy, item_w, item_h, bg);

        let color = item.palette_color();
        let preview = (item_w * PALETTE_SWATCH_FRAC).min(item_h * PALETTE_SWATCH_HEIGHT_FRAC);
        draw_rectangle(ix + item_w * 0.5 - preview * 0.5, iy + PALETTE_SWATCH_Y_OFFSET, preview, preview * PALETTE_SWATCH_H_FRAC, color);

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
        draw_text(label, ix + 2.0, iy + item_h - 4.0, PALETTE_FONT_SIZE, rgba(PALETTE_TEXT_COLOR));

        let key_label = match i {
            0 => "1", 1 => "2", 2 => "3", 3 => "4", 4 => "5",
            5 => "6", 6 => "7", 7 => "8", 8 => "9", 9 => "0",
            _ => "",
        };
        if !key_label.is_empty() {
            draw_text(key_label, ix + item_w - 12.0, iy + 10.0, PALETTE_KEY_FONT_SIZE, rgba(PALETTE_KEY_COLOR));
        }
    }
}

pub fn hit_test_palette(mx: f32, my: f32, screen_w: f32, screen_h: f32) -> Option<SelectBlock> {
    let bar_y = screen_h - UI_BAR_HEIGHT;
    if my < bar_y || my > screen_h { return None; }
    if mx < 0.0 || mx > screen_w { return None; }

    let items = SelectBlock::ALL;
    let count = items.len() as f32;
    let total_gap = PALETTE_GAP * (count + 1.0);
    let item_w = ((screen_w - total_gap) / count).max(PALETTE_MIN_ITEM_WIDTH);

    let idx = ((mx - PALETTE_GAP) / (item_w + PALETTE_GAP)) as usize;
    if idx < items.len() { Some(items[idx]) } else { None }
}

pub fn draw_hover_tooltip(world: &World, camera: &Camera, mx: f32, my: f32, screen_w: f32, screen_h: f32) {
    let bar_y = screen_h - UI_BAR_HEIGHT;
    if my >= bar_y { return; }

    let (wx, wy) = camera.screen_to_world(mx, my);
    let Some(block) = world.get(wx, wy) else { return; };

    let info = format!("({},{}) {} pwr:{}", wx, wy, block.display_name(), block.power);
    let tw = measure_text(&info, None, TOOLTIP_FONT_SIZE as u16, 1.0).width;
    let tx = (mx - tw * 0.5).clamp(2.0, screen_w - tw - 4.0);
    let ty = my - TOOLTIP_Y_OFFSET;

    draw_rectangle(tx - 2.0, ty - TOOLTIP_FONT_SIZE - 2.0, tw + 4.0, TOOLTIP_FONT_SIZE + 4.0, rgba(TOOLTIP_BG_COLOR));
    draw_text(&info, tx, ty, TOOLTIP_FONT_SIZE, WHITE);
}

pub fn draw_coordinates(camera: &Camera, mx: f32, my: f32, _screen_w: f32, _screen_h: f32) {
    let (wx, wy) = camera.screen_to_world(mx, my);
    draw_text(format!("({}, {})", wx, wy), COORD_X, COORD_Y, COORD_FONT_SIZE, rgba(COORD_TEXT_COLOR));
}
