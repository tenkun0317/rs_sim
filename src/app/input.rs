use crate::block::*;
use crate::constants::*;
use crate::render::{self, SelectBlock};
use crate::sim;
use macroquad::prelude::*;

impl super::AppState {
    pub fn handle_input(&mut self) {
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
                        if !self.world.in_bounds(wx, wy) {
                            self.world.expand_to_chunk(
                                self.world.chunk_at(wx, wy).0,
                                self.world.chunk_at(wx, wy).1,
                            );
                            self.last_placed_pos = Some((wx, wy));
                            self.left_held = true;
                            return;
                        }

                        let prev_pos = self.last_placed_pos;
                        self.last_placed_pos = Some((wx, wy));
                        self.left_held = true;

                        let existing = self.world.get(wx, wy).copied();

                        if let Some(block) = existing {
                            if left_pressed {
                                match block.id {
                                    BlockId::Barrel => {
                                        let strength =
                                            (decode_barrel_strength(block.data) + 1) % BARREL_STRENGTH_COUNT;
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
                                        let delay = (decode_repeater_delay(block.data) + 1) % REPEATER_DELAY_COUNT;
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
                            let threshold = CELL_SIZE * self.camera.zoom * CENTER_CLICK_THRESHOLD;
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
            let new_zoom = (self.camera.zoom * (1.0 + steps * ZOOM_STEP)).clamp(ZOOM_MIN, ZOOM_MAX);
            if (new_zoom - old_zoom).abs() > ZOOM_EPSILON {
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
                super::SimMode::Off => super::SimMode::Timed,
                super::SimMode::Timed => super::SimMode::Instant,
                super::SimMode::Instant => super::SimMode::Off,
            };
            if self.sim_mode == super::SimMode::Instant {
                self.simulation_needed = true;
            }
        }
        if is_key_pressed(KeyCode::Enter) && self.sim_mode == super::SimMode::Off {
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
            && self.sim_mode == super::SimMode::Timed
        {
            self.ticks_per_sec = (self.ticks_per_sec * 0.5).max(MIN_TPS);
        }
        if (is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::RightBracket))
            && self.sim_mode == super::SimMode::Timed
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

        let ps = PAN_SPEED_BASE / self.camera.zoom;
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
}
