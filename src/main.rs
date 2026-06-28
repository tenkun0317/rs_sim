#![allow(dead_code)]

mod app;
mod block;
mod constants;
mod export;
mod history;
mod project;
mod render;
mod sim;
mod world;

use app::AppState;
use macroquad::prelude::next_frame;

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
