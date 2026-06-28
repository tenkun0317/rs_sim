use crate::block::*;
use crate::history::{EditAction, History};
use crate::render::Camera;
use crate::world;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProjectFile {
    chunks: Vec<((i32, i32), Vec<(u8, u8, Block)>)>,
    undo_stack: Vec<EditAction>,
    redo_stack: Vec<EditAction>,
    #[serde(default)]
    camera_offset_x: f32,
    #[serde(default)]
    camera_offset_y: f32,
    #[serde(default)]
    camera_zoom: f32,
}

pub fn load_project_file(path: &str) -> Option<(world::World, History, Camera)> {
    let json = std::fs::read_to_string(path).ok()?;
    let data: ProjectFile = serde_json::from_str(&json).ok()?;
    let mut history = History::new();
    history.undo_stack = data.undo_stack;
    history.redo_stack = data.redo_stack;
    let mut world = world::World { chunks: std::collections::HashMap::new() };
    for ((cx, cy), blocks) in data.chunks {
        let mut chunk = world::Chunk::new();
        for (lx, ly, block) in blocks {
            chunk.blocks[ly as usize][lx as usize] = block;
        }
        world.chunks.insert((cx, cy), chunk);
    }
    let camera = Camera {
        offset_x: data.camera_offset_x,
        offset_y: data.camera_offset_y,
        zoom: data.camera_zoom,
    };
    Some((world, history, camera))
}

pub fn save_project_file(
    path: &str,
    world: &world::World,
    history: &History,
    camera: &Camera,
) -> Result<(), String> {
    let chunks: Vec<((i32, i32), Vec<(u8, u8, Block)>)> = world
        .chunks
        .iter()
        .filter_map(|(&key, chunk)| {
            let blocks: Vec<(u8, u8, Block)> = chunk
                .blocks
                .iter()
                .enumerate()
                .flat_map(|(ly, row)| {
                    row.iter().enumerate().filter_map(move |(lx, b)| {
                        if b.id != BlockId::Air {
                            Some((lx as u8, ly as u8, *b))
                        } else {
                            None
                        }
                    })
                })
                .collect();
            if blocks.is_empty() { None } else { Some((key, blocks)) }
        })
        .collect();
    let data = ProjectFile {
        chunks,
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
