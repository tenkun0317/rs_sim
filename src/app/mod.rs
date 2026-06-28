mod input;
mod clipboard;

pub use clipboard::ClipboardData;

use crate::block::*;
use crate::constants::*;
use crate::history::History;
use crate::project::{load_project_file, save_project_file};
use crate::render::{Camera, SelectBlock};
use crate::sim;
use macroquad::prelude::*;

pub struct AppState {
    pub world: crate::world::World,
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

impl AppState {
    pub fn new() -> Self {
        let _ = std::fs::create_dir_all(DEFAULT_SAVE_DIR.trim_end_matches('\\'));
        let (world, loaded_history, loaded_camera) = if std::path::Path::new(TEMP_SAVE_PATH).exists() {
            load_project_file(TEMP_SAVE_PATH).unwrap_or_else(|| {
                let mut w = crate::world::World::new(WORLD_CHUNKS_X, WORLD_CHUNKS_Y);
                w.place_test_circuit();
                (w, History::new(), Camera::new())
            })
        } else {
            let mut w = crate::world::World::new(WORLD_CHUNKS_X, WORLD_CHUNKS_Y);
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
        let _ = save_project_file(TEMP_SAVE_PATH, &self.world, &self.history, &self.camera);
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

        self.world = crate::world::World::new(WORLD_CHUNKS_X, WORLD_CHUNKS_Y);
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
            DEFAULT_SAVE_DIR,
            &[SAVE_EXTENSION],
            SAVE_EXTENSION_LABEL,
        ) {
            let _ = self.save_world(&path);
        }
    }

    fn do_export_nbt(&mut self) {
        if let Some(path) = tinyfiledialogs::save_file_dialog_with_filter(
            "Export NBT",
            DEFAULT_SAVE_DIR,
            &[NBT_EXTENSION],
            NBT_EXTENSION_LABEL,
        ) {
            let mut file = match std::fs::File::create(&path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to create NBT file: {}", e);
                    return;
                }
            };
            if let Err(e) = crate::export::export_nbt(&mut file, &self.world) {
                eprintln!("Failed to export NBT: {}", e);
            }
        }
    }

    fn do_load(&mut self) {
        if let Some(path) =
            tinyfiledialogs::open_file_dialog("Load World", "", Some((&[SAVE_EXTENSION], SAVE_EXTENSION_LABEL)))
        {
            let _ = self.load_world(&path);
        }
    }

    pub fn update(&mut self) {
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
            let path = self.current_save_path.clone().unwrap_or_else(|| TEMP_SAVE_PATH.to_string());
            let _ = save_project_file(&path, &self.world, &self.history, &self.camera);
        }
    }

    pub fn render(&self) {
        clear_background(Color::from_rgba(BG_CLEAR_COLOR.0, BG_CLEAR_COLOR.1, BG_CLEAR_COLOR.2, BG_CLEAR_COLOR.3));

        crate::render::draw_world(
            &self.world,
            &self.camera,
            screen_width(),
            screen_height() - UI_BAR_HEIGHT,
        );

        let (mx, my) = mouse_position();
        crate::render::draw_coordinates(&self.camera, mx, my, screen_width(), screen_height());
        crate::render::draw_hover_tooltip(
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
            draw_rectangle_lines(sx, sy, cs, cs, HOVER_OUTLINE_WIDTH, Color::from_rgba(HOVER_HIGHLIGHT_COLOR.0, HOVER_HIGHLIGHT_COLOR.1, HOVER_HIGHLIGHT_COLOR.2, HOVER_HIGHLIGHT_COLOR.3));
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
            draw_rectangle(rx, ry, rw, rh, Color::from_rgba(SELECTION_FILL_COLOR.0, SELECTION_FILL_COLOR.1, SELECTION_FILL_COLOR.2, SELECTION_FILL_COLOR.3));
            draw_rectangle_lines(rx, ry, rw, rh, SELECTION_OUTLINE_WIDTH, Color::from_rgba(SELECTION_OUTLINE_COLOR.0, SELECTION_OUTLINE_COLOR.1, SELECTION_OUTLINE_COLOR.2, SELECTION_OUTLINE_COLOR.3));
            let label = format!("{}x{} area | DEL to clear", x1 - x0 + 1, y1 - y0 + 1);
            draw_text(
                &label,
                rx,
                ry - SELECTION_LABEL_Y_OFFSET,
                SELECTION_LABEL_FONT_SIZE,
                Color::from_rgba(SELECTION_LABEL_COLOR.0, SELECTION_LABEL_COLOR.1, SELECTION_LABEL_COLOR.2, SELECTION_LABEL_COLOR.3),
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
                PASTE_TEXT_X,
                PASTE_TEXT_Y,
                PASTE_TEXT_FONT_SIZE,
                Color::from_rgba(PASTE_MODE_TEXT_COLOR.0, PASTE_MODE_TEXT_COLOR.1, PASTE_MODE_TEXT_COLOR.2, PASTE_MODE_TEXT_COLOR.3),
            );
        }

        crate::render::draw_ui_palette(self.selected_block, screen_width(), screen_height());

        let mode_display = match self.sim_mode {
            SimMode::Off => "MANUAL".to_string(),
            SimMode::Timed => format!("TIMED {:.1}t/s", self.ticks_per_sec),
            SimMode::Instant => "INSTANT".to_string(),
        };
        let text = format!("[Space] {} | [Enter] Step | [+/-] Speed | [R] Clear | [C] Center | [Tab/1-0] Sel | WASD Pan | LClick+Drag: place/interact | RClick+Drag: select | Ctrl+X/C/V: cut/copy/paste | Ctrl+S: save-as | Ctrl+R: load | Ctrl+Z: undo | Ctrl+Y: redo | DEL: delete selected | ESC: cancel", mode_display);
        draw_text(
            &text,
            STATUS_TEXT_X,
            screen_height() - UI_BAR_HEIGHT - STATUS_TEXT_Y_OFFSET,
            STATUS_FONT_SIZE,
            Color::from_rgba(STATUS_TEXT_COLOR.0, STATUS_TEXT_COLOR.1, STATUS_TEXT_COLOR.2, STATUS_TEXT_COLOR.3),
        );
    }
}
