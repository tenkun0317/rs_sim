use crate::block::*;
use crate::world::World;
use serde::Serialize;
use std::io::Write;

#[derive(Serialize)]
struct NbtWorld<'a> {
    chunks: Vec<NbtChunk<'a>>,
}

#[derive(Serialize)]
struct NbtChunk<'a> {
    cx: i32,
    cy: i32,
    blocks: Vec<NbtBlock<'a>>,
}

#[derive(Serialize)]
struct NbtBlock<'a> {
    x: i8,
    y: i8,
    #[serde(rename = "id")]
    block_id: &'a str,
    data: i16,
    power: i8,
}

fn block_id_name(id: BlockId) -> &'static str {
    match id {
        BlockId::Air => "air",
        BlockId::SolidBlock => "solid_block",
        BlockId::RedstoneWire => "redstone_wire",
        BlockId::RedstoneTorch => "redstone_torch",
        BlockId::RedstoneBlock => "redstone_block",
        BlockId::Repeater => "repeater",
        BlockId::Comparator => "comparator",
        BlockId::Lever => "lever",
        BlockId::Button => "button",
        BlockId::RedstoneLamp => "redstone_lamp",
        BlockId::Target => "target",
        BlockId::Barrel => "barrel",
    }
}

pub fn export_nbt<W: Write>(writer: &mut W, world: &World) -> Result<(), String> {
    let mut chunks = Vec::new();

    for (&(cx, cy), chunk) in &world.chunks {
        let mut blocks = Vec::new();
        for ly in 0..16 {
            for lx in 0..16 {
                let block = &chunk.blocks[ly][lx];
                if block.id != BlockId::Air {
                    blocks.push(NbtBlock {
                        x: lx as i8,
                        y: ly as i8,
                        block_id: block_id_name(block.id),
                        data: block.data as i16,
                        power: block.power as i8,
                    });
                }
            }
        }
        if !blocks.is_empty() {
            chunks.push(NbtChunk { cx, cy, blocks });
        }
    }

    let nbt_world = NbtWorld { chunks };

    nbt::to_writer(writer, &nbt_world, Some("Circuit"))
        .map_err(|e| e.to_string())?;

    Ok(())
}
