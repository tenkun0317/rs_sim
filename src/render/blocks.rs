use macroquad::prelude::*;
use crate::block::*;
use crate::constants::{
    BARREL_FILL_COLOR, BARREL_STRIPE_COLOR, BARREL_TEXT_COLOR, BARREL_TEXT_FRAC, BARREL_TEXT_MIN,
    BARREL_STRIPE_X_FRAC, BARREL_STRIPE_Y_FRAC, BARREL_STRIPE_W_FRAC, BARREL_STRIPE_H_FRAC,
    BARREL_STRIPE2_Y_FRAC, BLOCK_INSET_FRAC, BLOCK_OUTLINE_WIDTH,
    BUTTON_INNER_COLOR, BUTTON_OFF_COLOR, BUTTON_ON_COLOR, BUTTON_POWERED_SHIFT, BUTTON_INNER_FRAC,
    COMPARATOR_ARROW_LEN_FRAC, COMPARATOR_CIRCLE_OFF_FRAC, COMPARATOR_CIRCLE_R_FRAC,
    COMPARATOR_COMPARE_COLOR, COMPARATOR_LINE_W_FRAC, COMPARATOR_OFF_COLOR, COMPARATOR_ON_COLOR,
    COMPARATOR_POWER_COLOR, COMPARATOR_POWER_TEXT_FRAC, COMPARATOR_POWER_X_FRAC,
    COMPARATOR_POWER_Y_FRAC, COMPARATOR_POWER_MIN, COMPARATOR_SUBTRACT_COLOR,
    COMPARATOR_TEXT_MIN, COMPARATOR_TEXT_X_FRAC, COMPARATOR_TEXT_Y_FRAC,
    LAMP_OFF_COLOR, LAMP_ON_COLOR, LAMP_ON_OUTLINE_COLOR,
    LEVER_BG_COLOR, LEVER_KNOB_OFF_COLOR, LEVER_KNOB_ON_COLOR, LEVER_STICK_COLOR,
    LEVER_STICK_LEN_FRAC, LEVER_LINE_W_FRAC, LEVER_KNOB_R_FRAC,
    MIN_FONT_SIZE, MAX_POWER,
    REDSTONE_BLOCK_FILL_COLOR, REDSTONE_BLOCK_OUTLINE_COLOR,
    REPEATER_ARROW_LEN_FRAC, REPEATER_ARROW_W_FRAC, REPEATER_LINE_W_FRAC, REPEATER_OFF_COLOR,
    REPEATER_ON_COLOR, REPEATER_PERP_FRAC, REPEATER_POWER_COLOR, REPEATER_POWER_TEXT_FRAC,
    REPEATER_POWER_X_FRAC, REPEATER_POWER_Y_FRAC, REPEATER_TEXT_MIN, REPEATER_TEXT_X_FRAC,
    REPEATER_TEXT_Y_FRAC,
    SOLID_BLOCK_FILL_COLOR, SOLID_BLOCK_OUTLINE_COLOR, SOLID_POWER_COLOR, SOLID_POWER_TEXT_FRAC,
    SOLID_POWER_TEXT_MIN,
    TARGET_DOT_COLOR, TARGET_DOT_R_FRAC, TARGET_FILL_COLOR, TARGET_OUTLINE_COLOR,
    TARGET_RING_COLOR, TARGET_RING_R_FRAC, TARGET_RING_W_FRAC,
    TORCH_ATTACH_OFFSET_FRAC, TORCH_BODY_LIT_COLOR, TORCH_BODY_UNLIT_COLOR,
    TORCH_FLOOR_GLOW_R_FRAC, TORCH_FLOOR_HEAD_R_FRAC, TORCH_FLOOR_STALK_H_FRAC,
    TORCH_FLOOR_STALK_W_FRAC, TORCH_GLOW_COLOR, TORCH_HEAD_INNER_R_FRAC, TORCH_HEAD_R_FRAC,
    TORCH_POST_COLOR, TORCH_POST_H_FRAC, TORCH_POST_W_FRAC, TORCH_STALK_H_FRAC,
    TORCH_STALK_H_START_FRAC, TORCH_STALK_W_FRAC,
    WIRE_COLOR_B_BASE, WIRE_COLOR_B_RANGE, WIRE_COLOR_G_BASE, WIRE_COLOR_G_RANGE,
    WIRE_COLOR_R_BASE, WIRE_COLOR_R_RANGE, WIRE_DOT_FRAC, WIRE_TEXT_FRAC, WIRE_TEXT_MIN,
    WIRE_THICK_FRAC, WIRE_THIN_FRAC,
};
use crate::sim::power;
use crate::world::World;

fn rgba(c: (u8, u8, u8, u8)) -> Color {
    Color::from_rgba(c.0, c.1, c.2, c.3)
}

pub fn draw_block(world: &World, wx: i32, wy: i32, block: &Block, sx: f32, sy: f32, cs: f32) {
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

            let conn = |d| power::wire_connects_in_dir(world, wx, wy, d);
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
