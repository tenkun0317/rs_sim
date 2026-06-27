#![allow(dead_code)]

mod block;
mod history;
mod render;
mod sim;
mod world;

use crate::block::*;
use crate::history::{EditAction, History};
use crate::world::{Chunk, CHUNK_SIZE_I32};
use macroquad::prelude::*;
use render::{Camera, SelectBlock, CELL_SIZE};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct ClipboardData {
    rows: Vec<Vec<Block>>,
    width: usize,
    height: usize,
}

#[derive(Serialize, Deserialize)]
struct ProjectFile {
    chunks: Vec<((i32, i32), Chunk)>,
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    #[serde(default)]
    camera_offset_x: f32,
    #[serde(default)]
    camera_offset_y: f32,
    #[serde(default)]
    camera_zoom: f32,
}

fn load_project_file(path: &str) -> Option<(world::World, History, Camera)> {
    let json = std::fs::read_to_string(path).ok()?;
    let data: ProjectFile = serde_json::from_str(&json).ok()?;
    let mut history = History::new();
    history.undo_stack = data.undo_stack;
    history.redo_stack = data.redo_stack;
    let world = world::World { chunks: data.chunks.into_iter().collect() };
    let camera = Camera {
        offset_x: data.camera_offset_x,
        offset_y: data.camera_offset_y,
        zoom: data.camera_zoom,
    };
    Some((world, history, camera))
}

fn save_project_file(
    path: &str,
    world: &world::World,
    history: &History,
    camera: &Camera,
) -> Result<(), String> {
    let data = ProjectFile {
        chunks: world.chunks.iter().map(|(k, v)| (*k, v.clone())).collect(),
        undo_stack: history.undo_stack.clone(),
        redo_stack: history.redo_stack.clone(),
        camera_offset_x: camera.offset_x,
        camera_offset_y: camera.offset_y,
        camera_zoom: camera.zoom,
    };
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    std::fs::write(path, json.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

const WORLD_CHUNKS_X: usize = 4;
const WORLD_CHUNKS_Y: usize = 4;
const UI_BAR_HEIGHT: f32 = 60.0;

struct AppState {
    world: world::World,
    camera: Camera,
    selected_block: SelectBlock,
    last_mouse_world: (i32, i32),
    mouse_down_pos: Option<(f32, f32)>,
    panning: bool,
    left_held: bool,
    last_placed_pos: Option<(i32, i32)>,
    select_start: Option<(i32, i32)>,
    select_end: Option<(i32, i32)>,
    clipboard: Option<ClipboardData>,
    paste_mode: bool,
    current_save_path: Option<String>,
    history: History,
    simulation_needed: bool,
    auto_save_needed: bool,
    dirty: bool,
    sim_mode: SimMode,
    ticks_per_sec: f64,
    tick_accumulator: f64,
    lever_cooldown: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SimMode {
    Off,
    Timed,
    Instant,
}

const MIN_TPS: f64 = 0.15625;
const MAX_TPS: f64 = 81920.0;
const DEFAULT_TPS: f64 = 10.0;
const MAX_TICKS_PER_FRAME: u32 = 60;

impl AppState {
    fn new() -> Self {
        let _ = std::fs::create_dir_all("saves");
        let temp_path = "saves/temp.json";
        let (world, loaded_history, loaded_camera) = if std::path::Path::new(temp_path).exists() {
            load_project_file(temp_path).unwrap_or_else(|| {
                let mut w = world::World::new(WORLD_CHUNKS_X, WORLD_CHUNKS_Y);
                w.place_test_circuit();
                (w, History::new(), Camera::new())
            })
        } else {
            let mut w = world::World::new(WORLD_CHUNKS_X, WORLD_CHUNKS_Y);
            w.place_test_circuit();
            (w, History::new(), Camera::new())
        };
        let mut app = AppState {
            world,
            camera: loaded_camera,
            selected_block: SelectBlock::RedstoneWire,
            last_mouse_world: (0, 0),
            mouse_down_pos: None,
            panning: false,
            left_held: false,
            last_placed_pos: None,
            select_start: None,
            select_end: None,
            clipboard: None,
            paste_mode: false,
            current_save_path: None,
            history: loaded_history,
            simulation_needed: true,
            auto_save_needed: false,
            dirty: false,
            sim_mode: SimMode::Timed,
            ticks_per_sec: DEFAULT_TPS,
            tick_accumulator: 0.0,
            lever_cooldown: false,
        };

        app.center_camera();
        app
    }

    fn center_camera(&mut self) {
        let screen_w = screen_width();
        let screen_h = screen_height() - UI_BAR_HEIGHT;
        let cs = CELL_SIZE * self.camera.zoom;
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for (&(cx, cy), _) in &self.world.chunks {
            min_x = min_x.min(cx * CHUNK_SIZE_I32);
            min_y = min_y.min(cy * CHUNK_SIZE_I32);
            max_x = max_x.max((cx + 1) * CHUNK_SIZE_I32);
            max_y = max_y.max((cy + 1) * CHUNK_SIZE_I32);
        }
        if min_x == i32::MAX { return; }
        let cx = (min_x as f32 + max_x as f32) / 2.0;
        let cy = (min_y as f32 + max_y as f32) / 2.0;
        self.camera.offset_x = screen_w / 2.0 - cx * cs;
        self.camera.offset_y = screen_h / 2.0 - cy * cs;
    }

    fn handle_input(&mut self) {
        let (mx, my) = mouse_position();

        let is_in_ui = my >= screen_height() - UI_BAR_HEIGHT;

        if let Some(block) = render::hit_test_palette(mx, my, screen_width(), screen_height()) {
            if is_mouse_button_pressed(MouseButton::Left) {
                self.selected_block = block;
            }
        }

        if !is_in_ui {
            let (wx, wy) = self.camera.screen_to_world(mx, my);
            self.last_mouse_world = (wx, wy);

            if is_mouse_button_pressed(MouseButton::Middle)
                || (is_mouse_button_pressed(MouseButton::Right)
                    && is_key_down(KeyCode::LeftControl))
            {
                self.mouse_down_pos = Some((mx, my));
                self.panning = true;
            }

            if self.panning {
                if is_mouse_button_down(MouseButton::Middle)
                    || (is_mouse_button_down(MouseButton::Right)
                        && is_key_down(KeyCode::LeftControl))
                {
                    if let Some((last_mx, last_my)) = self.mouse_down_pos {
                        self.camera.offset_x += mx - last_mx;
                        self.camera.offset_y += my - last_my;
                        self.mouse_down_pos = Some((mx, my));
                    }
                } else {
                    self.panning = false;
                    self.mouse_down_pos = None;
                }
            }

            if !self.panning {
                let left_down = is_mouse_button_down(MouseButton::Left);
                let left_pressed = is_mouse_button_pressed(MouseButton::Left);

                let mut cleared_selection = false;
                if left_pressed {
                    if self.select_start.is_some() {
                        self.select_start = None;
                        self.select_end = None;
                        cleared_selection = true;
                    }
                }

                if self.paste_mode && left_pressed {
                    if self.world.in_bounds(wx, wy) {
                        self.paste_clipboard(wx, wy);
                    }
                } else if !cleared_selection
                    && self.select_start.is_none()
                    && left_down
                {
                    let moved = self.last_placed_pos != Some((wx, wy));
                    let should_place = left_pressed || (self.left_held && moved);

                    if should_place {
                        let prev_pos = self.last_placed_pos;
                        self.last_placed_pos = Some((wx, wy));
                        self.left_held = true;

                        let existing = self.world.get(wx, wy).copied();

                        if let Some(block) = existing {
                            if left_pressed {
                                match block.id {
                                    BlockId::Barrel => {
                                        let strength =
                                            (decode_barrel_strength(block.data) + 1) % 16;
                                        self.set_block(wx, wy, Block::barrel(strength));
                                        return;
                                    }
                                    BlockId::Lever => {
                                        let powered = !decode_lever_powered(block.data);
                                        self.set_block(
                                            wx,
                                            wy,
                                            Block::lever(decode_lever_dir(block.data), powered),
                                        );
                                        return;
                                    }
                                    BlockId::Button => {
                                        self.set_block(
                                            wx,
                                            wy,
                                            Block::button(decode_lever_dir(block.data), true),
                                        );
                                        return;
                                    }
                                    BlockId::Repeater => {
                                        let dir = decode_repeater_dir(block.data);
                                        let delay = (decode_repeater_delay(block.data) + 1) % 4;
                                        let locked = decode_repeater_locked(block.data);
                                        let powered = decode_repeater_powered(block.data);
                                        self.set_block(
                                            wx,
                                            wy,
                                            Block::repeater(dir, delay, locked, powered),
                                        );
                                        return;
                                    }
                                    BlockId::Comparator => {
                                        let dir = decode_comparator_dir(block.data);
                                        let mode = match decode_comparator_mode(block.data) {
                                            ComparatorMode::Compare => ComparatorMode::Subtract,
                                            ComparatorMode::Subtract => ComparatorMode::Compare,
                                        };
                                        let powered = decode_comparator_powered(block.data);
                                        self.set_block(
                                            wx,
                                            wy,
                                            Block::comparator(dir, mode, powered),
                                        );
                                        return;
                                    }
                                    _ => {}
                                }
                            }
                        }

                        if self.selected_block == SelectBlock::Eraser {
                            self.set_block(wx, wy, Block::air());
                            return;
                        }

                        let is_drag = !left_pressed;
                        let dir = if is_drag {
                            let (ddx, ddy) = prev_pos.map_or((0, 0), |(px, py)| (wx - px, wy - py));
                            if ddx.abs() > ddy.abs() {
                                Some(if ddx > 0 {
                                    Direction::East
                                } else {
                                    Direction::West
                                })
                            } else if ddy != 0 {
                                Some(if ddy > 0 {
                                    Direction::South
                                } else {
                                    Direction::North
                                })
                            } else {
                                self.calculate_placement_dir(wx, wy)
                            }
                        } else {
                            self.calculate_placement_dir(wx, wy)
                        };
                        let is_center = if is_drag {
                            false
                        } else {
                            let (mx, my) = mouse_position();
                            let cx = wx as f32 * CELL_SIZE * self.camera.zoom
                                + self.camera.offset_x
                                + CELL_SIZE * self.camera.zoom / 2.0;
                            let cy = wy as f32 * CELL_SIZE * self.camera.zoom
                                + self.camera.offset_y
                                + CELL_SIZE * self.camera.zoom / 2.0;
                            let threshold = CELL_SIZE * self.camera.zoom * 0.25;
                            (mx - cx).abs() < threshold && (my - cy).abs() < threshold
                        };

                        let needs_dir = matches!(
                            self.selected_block,
                            SelectBlock::Repeater
                                | SelectBlock::Comparator
                                | SelectBlock::Lever
                                | SelectBlock::Button
                                | SelectBlock::RedstoneTorch
                        );
                        let new_block = if needs_dir {
                            if self.selected_block == SelectBlock::RedstoneTorch {
                                if is_center {
                                    Block::torch(true, false, Direction::North)
                                } else {
                                    let d = dir.unwrap_or(Direction::South);
                                    let (bx, by) = (wx + d.dx(), wy + d.dy());
                                    if self.world.in_bounds(bx, by)
                                        && self.world.get(bx, by).is_some_and(|b| {
                                            matches!(
                                                b.id,
                                                BlockId::SolidBlock
                                                    | BlockId::Target
                                                    | BlockId::RedstoneLamp
                                                    | BlockId::Barrel
                                            )
                                        })
                                    {
                                        Block::torch(true, true, d)
                                    } else {
                                        Block::torch(true, false, Direction::North)
                                    }
                                }
                            } else {
                                self.selected_block
                                    .to_block(dir.unwrap_or(Direction::South))
                            }
                        } else {
                            self.selected_block.to_block(Direction::North)
                        };

                        if existing.is_none_or(|b| b.id != new_block.id) {
                            self.set_block(wx, wy, new_block);
                        }
                    }
                } else {
                    self.left_held = false;
                    self.last_placed_pos = None;
                }

                if is_mouse_button_pressed(MouseButton::Right)
                    && !is_key_down(KeyCode::LeftControl)
                    && self.world.in_bounds(wx, wy)
                {
                    if let (Some(s), Some(e)) = (self.select_start, self.select_end) {
                        if s == e && s == (wx, wy) {
                            self.set_block(wx, wy, Block::air());
                            self.select_start = None;
                            self.select_end = None;
                        } else {
                            self.select_start = Some((wx, wy));
                            self.select_end = Some((wx, wy));
                        }
                    } else {
                        self.select_start = Some((wx, wy));
                        self.select_end = Some((wx, wy));
                    }
                }

                let right_down =
                    is_mouse_button_down(MouseButton::Right) && !is_key_down(KeyCode::LeftControl);
                if right_down && self.world.in_bounds(wx, wy) && self.select_start.is_some() {
                    self.select_end = Some((wx, wy));
                }

                if is_key_pressed(KeyCode::Delete) {
                    if let (Some(s), Some(e)) = (self.select_start, self.select_end) {
                        let x0 = s.0.min(e.0);
                        let x1 = s.0.max(e.0);
                        let y0 = s.1.min(e.1);
                        let y1 = s.1.max(e.1);
                        self.edit_begin();
                        for y in y0..=y1 {
                            for x in x0..=x1 {
                                if self.world.in_bounds(x, y) {
                                    self.set_block(x, y, Block::air());
                                }
                            }
                        }
                        self.edit_end();
                        self.select_start = None;
                        self.select_end = None;
                    }
                }

                if is_key_pressed(KeyCode::Escape) {
                    self.select_start = None;
                    self.select_end = None;
                    self.paste_mode = false;
                    self.left_held = false;
                    self.last_placed_pos = None;
                }
            }
        }

        let wheel_delta = mouse_wheel().1;
        if wheel_delta != 0.0 {
            let steps = if wheel_delta.abs() > 1.0 {
                wheel_delta.signum()
            } else {
                wheel_delta
            };
            let old_zoom = self.camera.zoom;
            let new_zoom = (self.camera.zoom * (1.0 + steps * 0.06)).clamp(0.0625, 16.0);
            if (new_zoom - old_zoom).abs() > 0.001 {
                let (mx, my) = mouse_position();
                let wx_exact = (mx - self.camera.offset_x) / (CELL_SIZE * old_zoom);
                let wy_exact = (my - self.camera.offset_y) / (CELL_SIZE * old_zoom);
                self.camera.zoom = new_zoom;
                self.camera.offset_x = mx - wx_exact * CELL_SIZE * new_zoom;
                self.camera.offset_y = my - wy_exact * CELL_SIZE * new_zoom;
            } else {
                self.camera.zoom = new_zoom;
            }
        }

        if is_key_pressed(KeyCode::Space) {
            self.sim_mode = match self.sim_mode {
                SimMode::Off => SimMode::Timed,
                SimMode::Timed => SimMode::Instant,
                SimMode::Instant => SimMode::Off,
            };
            if self.sim_mode == SimMode::Instant {
                self.simulation_needed = true;
            }
        }
        if is_key_pressed(KeyCode::Enter) && self.sim_mode == SimMode::Off {
            sim::step_simulation(&mut self.world);
        }
        let ctrl = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

        if ctrl && !shift && is_key_pressed(KeyCode::Z) {
            self.undo();
        }
        if ctrl && !shift && is_key_pressed(KeyCode::Y) {
            self.redo();
        }
        if ctrl && !shift && is_key_pressed(KeyCode::X) {
            self.cut_selection();
        }
        if ctrl && !shift && is_key_pressed(KeyCode::C) {
            self.copy_selection();
        }
        if ctrl && !shift && is_key_pressed(KeyCode::V) {
            if self.clipboard.is_some() {
                self.paste_mode = !self.paste_mode;
                if self.paste_mode {
                    self.select_start = None;
                    self.select_end = None;
                }
            }
        }
        if ctrl && is_key_pressed(KeyCode::S) {
            self.do_save_as();
        }
        if ctrl && is_key_pressed(KeyCode::R) {
            self.do_load();
        }
        if ctrl && is_key_pressed(KeyCode::N) {
            self.do_new();
        }
        if (is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::LeftBracket))
            && self.sim_mode == SimMode::Timed
        {
            self.ticks_per_sec = (self.ticks_per_sec * 0.5).max(MIN_TPS);
        }
        if (is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::RightBracket))
            && self.sim_mode == SimMode::Timed
        {
            self.ticks_per_sec = (self.ticks_per_sec * 2.0).min(MAX_TPS);
        }
        if is_key_pressed(KeyCode::R) && !ctrl {
            let positions: Vec<(i32, i32)> = {
                let w = &self.world;
                let mut pos = Vec::new();
                for (&(cx, cy), _) in &w.chunks {
                    let base_x = cx * CHUNK_SIZE_I32;
                    let base_y = cy * CHUNK_SIZE_I32;
                    for ly in 0..16 {
                        for lx in 0..16 {
                            let wx = base_x + lx;
                            let wy = base_y + ly;
                            if let Some(b) = w.get(wx, wy) {
                                if b.id != BlockId::Air {
                                    pos.push((wx, wy));
                                }
                            }
                        }
                    }
                }
                pos
            };
            self.edit_begin();
            for (wx, wy) in positions {
                self.set_block(wx, wy, Block::air());
            }
            self.edit_end();
        }
        if is_key_pressed(KeyCode::C) {
            self.center_camera();
        }
        if is_key_pressed(KeyCode::Tab) {
            let idx = SelectBlock::ALL
                .iter()
                .position(|s| *s == self.selected_block);
            if let Some(idx) = idx {
                self.selected_block = SelectBlock::ALL[(idx + 1) % SelectBlock::ALL.len()];
            }
        }

        for i in 0..SelectBlock::ALL.len() {
            let key = match i {
                0 => KeyCode::Key1,
                1 => KeyCode::Key2,
                2 => KeyCode::Key3,
                3 => KeyCode::Key4,
                4 => KeyCode::Key5,
                5 => KeyCode::Key6,
                6 => KeyCode::Key7,
                7 => KeyCode::Key8,
                8 => KeyCode::Key9,
                9 => KeyCode::Key0,
                _ => continue,
            };
            if is_key_pressed(key) {
                self.selected_block = SelectBlock::ALL[i];
            }
        }

        let ps = 10.0 / self.camera.zoom;
        for &(key, dx, dy) in &[
            (KeyCode::Left, 1, 0),
            (KeyCode::A, 1, 0),
            (KeyCode::Right, -1, 0),
            (KeyCode::D, -1, 0),
            (KeyCode::Up, 0, 1),
            (KeyCode::W, 0, 1),
            (KeyCode::Down, 0, -1),
            (KeyCode::S, 0, -1),
        ] {
            if !ctrl && is_key_down(key) {
                self.camera.offset_x += dx as f32 * ps;
                self.camera.offset_y += dy as f32 * ps;
            }
        }
    }

    fn calculate_placement_dir(&self, wx: i32, wy: i32) -> Option<Direction> {
        let (mx, my) = mouse_position();
        let cx = wx as f32 * CELL_SIZE * self.camera.zoom
            + self.camera.offset_x
            + CELL_SIZE * self.camera.zoom / 2.0;
        let cy = wy as f32 * CELL_SIZE * self.camera.zoom
            + self.camera.offset_y
            + CELL_SIZE * self.camera.zoom / 2.0;
        let dx = mx - cx;
        let dy = my - cy;
        if dx.abs() > dy.abs() {
            Some(if dx > 0.0 {
                Direction::East
            } else {
                Direction::West
            })
        } else {
            Some(if dy > 0.0 {
                Direction::South
            } else {
                Direction::North
            })
        }
    }

    fn set_block(&mut self, x: i32, y: i32, block: Block) {
        let (cx, cy) = self.world.chunk_at(x, y);
        self.world.expand_to_chunk(cx, cy);
        let old = self.world.get(x, y).copied().unwrap_or(Block::air());
        if old != block {
            self.history.record(x, y, old, block);
            self.world.set(x, y, block);
            self.simulation_needed = true;
            self.auto_save_needed = true;
            self.dirty = true;
        }
    }

    fn edit_begin(&mut self) {
        self.history.begin_action();
    }

    fn edit_end(&mut self) {
        self.history.end_action();
        self.auto_save_needed = true;
        self.dirty = true;
    }

    fn undo(&mut self) {
        if let Some(action) = self.history.undo() {
            for ch in &action.changes {
                self.world.set(ch.x, ch.y, ch.old_block);
            }
            self.simulation_needed = true;
            self.auto_save_needed = true;
            self.dirty = true;
        }
    }

    fn redo(&mut self) {
        if let Some(action) = self.history.redo() {
            for ch in &action.changes {
                self.world.set(ch.x, ch.y, ch.new_block);
            }
            self.simulation_needed = true;
            self.auto_save_needed = true;
            self.dirty = true;
        }
    }

    fn copy_selection(&mut self) {
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

    fn cut_selection(&mut self) {
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

    fn paste_clipboard(&mut self, wx: i32, wy: i32) {
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

    fn get_selection_size(&self) -> Option<(i32, i32)> {
        let (s, e) = (self.select_start?, self.select_end?);
        let x0 = s.0.min(e.0);
        let x1 = s.0.max(e.0);
        let y0 = s.1.min(e.1);
        let y1 = s.1.max(e.1);
        Some((x1 - x0 + 1, y1 - y0 + 1))
    }

    fn save_world(&mut self, path: &str) -> Result<(), String> {
        save_project_file(path, &self.world, &self.history, &self.camera)?;
        self.current_save_path = Some(path.to_string());
        self.dirty = false;
        self.auto_save_needed = false;
        Ok(())
    }

    fn load_world(&mut self, path: &str) -> Result<(), String> {
        let (world, history, camera) = load_project_file(path).ok_or("Failed to load file")?;
        self.world = world;
        self.history = history;
        self.camera = camera;
        self.current_save_path = Some(path.to_string());
        self.simulation_needed = true;
        self.dirty = false;
        let _ = save_project_file("saves/temp.json", &self.world, &self.history, &self.camera);
        Ok(())
    }

    fn do_new(&mut self) {
        if self.dirty {
            let result = tinyfiledialogs::message_box_ok_cancel(
                "Unsaved Changes",
                "Save changes before creating new file?",
                tinyfiledialogs::MessageBoxIcon::Question,
                tinyfiledialogs::OkCancel::Cancel,
            );
            match result {
                tinyfiledialogs::OkCancel::Ok => {
                    self.do_save();
                    if self.dirty {
                        return;
                    }
                }
                tinyfiledialogs::OkCancel::Cancel => return,
            }
        }

        self.world = world::World::new(WORLD_CHUNKS_X, WORLD_CHUNKS_Y);
        self.history = History::new();
        self.camera = Camera::new();
        self.current_save_path = None;
        self.simulation_needed = true;
        self.auto_save_needed = false;
        self.dirty = false;
        self.select_start = None;
        self.select_end = None;
        self.clipboard = None;
        self.paste_mode = false;
        self.center_camera();
    }

    fn do_save(&mut self) {
        if let Some(ref path) = self.current_save_path.clone() {
            let _ = self.save_world(path);
        } else {
            self.do_save_as();
        }
    }

    fn do_save_as(&mut self) {
        if let Some(path) = tinyfiledialogs::save_file_dialog_with_filter(
            "Save World As",
            "saves\\",
            &["*.json"],
            "JSON files",
        ) {
            let _ = self.save_world(&path);
        }
    }

    fn do_load(&mut self) {
        if let Some(path) =
            tinyfiledialogs::open_file_dialog("Load World", "", Some((&["*.json"], "JSON files")))
        {
            let _ = self.load_world(&path);
        }
    }

    fn update(&mut self) {
        match self.sim_mode {
            SimMode::Instant => {
                if self.simulation_needed {
                    self.simulation_needed = false;
                    sim::update_simulation(&mut self.world);
                }
            }
            SimMode::Timed => {
                self.simulation_needed = false;
                let dt = get_frame_time() as f64;
                self.tick_accumulator += dt;
                let interval = 1.0 / self.ticks_per_sec;
                let mut ticks = 0u32;
                while self.tick_accumulator >= interval && ticks < MAX_TICKS_PER_FRAME {
                    self.tick_accumulator -= interval;
                    sim::step_simulation(&mut self.world);
                    ticks += 1;
                }
                if ticks >= MAX_TICKS_PER_FRAME {
                    self.tick_accumulator = 0.0;
                }
            }
            SimMode::Off => {}
        }
        if self.auto_save_needed {
            self.auto_save_needed = false;
            let path = self.current_save_path.clone().unwrap_or_else(|| "saves/temp.json".to_string());
            let _ = save_project_file(&path, &self.world, &self.history, &self.camera);
        }
    }

    fn render(&self) {
        clear_background(Color::from_rgba(20, 20, 20, 255));

        render::draw_world(
            &self.world,
            &self.camera,
            screen_width(),
            screen_height() - UI_BAR_HEIGHT,
        );

        let (mx, my) = mouse_position();
        render::draw_coordinates(&self.camera, mx, my, screen_width(), screen_height());
        render::draw_hover_tooltip(
            &self.world,
            &self.camera,
            mx,
            my,
            screen_width(),
            screen_height(),
        );

        let is_in_ui = my >= screen_height() - UI_BAR_HEIGHT;
        if !is_in_ui
            && self
                .world
                .in_bounds(self.last_mouse_world.0, self.last_mouse_world.1)
        {
            let (wx, wy) = self.last_mouse_world;
            let (sx, sy) = self.camera.world_to_screen(wx, wy);
            let cs = self.camera.cell_size();
            draw_rectangle_lines(sx, sy, cs, cs, 2.0, Color::from_rgba(255, 255, 255, 100));
        }

        if let (Some(s), Some(e)) = (self.select_start, self.select_end) {
            let x0 = s.0.min(e.0);
            let x1 = s.0.max(e.0);
            let y0 = s.1.min(e.1);
            let y1 = s.1.max(e.1);
            let (sx, sy) = self.camera.world_to_screen(x0, y0);
            let (ex, ey) = self.camera.world_to_screen(x1, y1);
            let cs = self.camera.cell_size();
            let rx = sx;
            let ry = sy;
            let rw = ex - sx + cs;
            let rh = ey - sy + cs;
            draw_rectangle(rx, ry, rw, rh, Color::from_rgba(255, 255, 255, 20));
            draw_rectangle_lines(rx, ry, rw, rh, 2.0, Color::from_rgba(255, 255, 255, 160));
            let label = format!("{}x{} area | DEL to clear", x1 - x0 + 1, y1 - y0 + 1);
            draw_text(
                &label,
                rx,
                ry - 6.0,
                14.0,
                Color::from_rgba(255, 255, 255, 200),
            );
        }

        if self.paste_mode {
            if let Some(ref clip) = self.clipboard {
                if !is_in_ui {
                    let (wx, wy) = self.last_mouse_world;
                    let cs = self.camera.cell_size();
                    for y in 0..clip.height {
                        for x in 0..clip.width {
                            let bx = wx + x as i32;
                            let by = wy + y as i32;
                            if self.world.in_bounds(bx, by) && clip.rows[y][x].id != BlockId::Air {
                                let (sx, sy) = self.camera.world_to_screen(bx, by);
                                draw_rectangle(sx, sy, cs, cs, Color::from_rgba(255, 255, 255, 40));
                                draw_rectangle_lines(
                                    sx,
                                    sy,
                                    cs,
                                    cs,
                                    1.0,
                                    Color::from_rgba(255, 255, 255, 120),
                                );
                            }
                        }
                    }
                }
            }
        }

        if self.paste_mode {
            draw_text(
                "PASTE MODE (click to paste, ESC to cancel)",
                10.0,
                20.0,
                18.0,
                Color::from_rgba(255, 255, 100, 255),
            );
        }

        render::draw_ui_palette(self.selected_block, screen_width(), screen_height());

        let mode_display = match self.sim_mode {
            SimMode::Off => "MANUAL".to_string(),
            SimMode::Timed => format!("TIMED {:.1}t/s", self.ticks_per_sec),
            SimMode::Instant => "INSTANT".to_string(),
        };
        let text = format!("[Space] {} | [Enter] Step | [+/-] Speed | [R] Clear | [C] Center | [Tab/1-0] Sel | WASD Pan | LClick+Drag: place/interact | RClick+Drag: select | Ctrl+X/C/V: cut/copy/paste | Ctrl+S: save-as | Ctrl+R: load | Ctrl+Z: undo | Ctrl+Y: redo | DEL: delete selected | ESC: cancel", mode_display);
        draw_text(
            &text,
            5.0,
            screen_height() - UI_BAR_HEIGHT - 5.0,
            12.0,
            Color::from_rgba(150, 150, 150, 255),
        );
    }
}

#[macroquad::main("RS Sim - Minecraft Redstone Simulator")]
async fn main() {
    let mut app = AppState::new();
    sim::update_simulation(&mut app.world);

    loop {
        app.handle_input();
        app.update();
        app.render();
        next_frame().await;
    }
}
