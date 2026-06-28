use macroquad::prelude::*;
use crate::block::*;
use crate::constants::{
    BAR_BG_COLOR, BAR_LINE_COLOR, COORD_FONT_SIZE, COORD_TEXT_COLOR, COORD_X, COORD_Y,
    PALETTE_FONT_SIZE, PALETTE_GAP, PALETTE_ITEM_HEIGHT_OFFSET,
    PALETTE_ITEM_INSET_Y, PALETTE_KEY_COLOR, PALETTE_KEY_FONT_SIZE, PALETTE_MIN_ITEM_WIDTH,
    PALETTE_SELECTED_COLOR, PALETTE_SWATCH_FRAC, PALETTE_SWATCH_H_FRAC,
    PALETTE_SWATCH_HEIGHT_FRAC, PALETTE_SWATCH_Y_OFFSET, PALETTE_TEXT_COLOR, PALETTE_UNSELECTED_COLOR,
    TOOLTIP_BG_COLOR, TOOLTIP_FONT_SIZE, TOOLTIP_Y_OFFSET, UI_BAR_HEIGHT,
};
use crate::world::World;
use super::Camera;

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
