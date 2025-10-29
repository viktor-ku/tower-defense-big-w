use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

/// Convert a world-space position to chunk coordinate assuming square chunks of `size`.
pub fn world_to_chunk(pos: Vec3, size: f32) -> ChunkCoord {
    let fx = pos.x.div_euclid(size).floor();
    let fz = pos.z.div_euclid(size).floor();
    ChunkCoord {
        x: fx as i32,
        z: fz as i32,
    }
}

/// Return the square neighborhood of chunks within radius r around center (inclusive).
pub fn desired_chunks(center: ChunkCoord, r: i32) -> HashSet<ChunkCoord> {
    let mut set = HashSet::new();
    for dz in -r..=r {
        for dx in -r..=r {
            set.insert(ChunkCoord {
                x: center.x + dx,
                z: center.z + dz,
            });
        }
    }
    set
}

/// Get the 8 adjacent chunks (cardinal + diagonal) around a center chunk.
pub fn adjacent_chunks(center: ChunkCoord) -> HashSet<ChunkCoord> {
    let mut set = HashSet::new();
    set.insert(ChunkCoord {
        x: center.x,
        z: center.z + 1,
    });
    set.insert(ChunkCoord {
        x: center.x,
        z: center.z - 1,
    });
    set.insert(ChunkCoord {
        x: center.x - 1,
        z: center.z,
    });
    set.insert(ChunkCoord {
        x: center.x + 1,
        z: center.z,
    });
    set.insert(ChunkCoord {
        x: center.x - 1,
        z: center.z + 1,
    });
    set.insert(ChunkCoord {
        x: center.x + 1,
        z: center.z + 1,
    });
    set.insert(ChunkCoord {
        x: center.x - 1,
        z: center.z - 1,
    });
    set.insert(ChunkCoord {
        x: center.x + 1,
        z: center.z - 1,
    });
    set
}
