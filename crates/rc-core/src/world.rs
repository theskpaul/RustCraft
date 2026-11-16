use crate::block::BlockId;
use crate::chunk::Chunk;
use crate::pos::ChunkPos;
use crate::view::Neighbourhood;
use std::collections::HashMap;

/// All currently loaded chunks. Simulation-side: no GPU types here.
#[derive(Default)]
pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    /// Store a chunk at `pos`. Returns the chunk it replaced, if any.
    pub fn insert_chunk(&mut self, pos: ChunkPos, chunk: Chunk) -> Option<Chunk> {
        self.chunks.insert(pos, chunk)
    }

    pub fn chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Block at world coordinates. Unloaded chunks read as air.
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> BlockId {
        let (cp, [lx, ly, lz]) = ChunkPos::from_block(x, y, z);
        match self.chunks.get(&cp) {
            Some(c) => c.get(lx, ly, lz),
            None => BlockId::AIR,
        }
    }

    /// Set a block at world coordinates. Returns false (and changes nothing)
    /// if the chunk is not loaded.
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, b: BlockId) -> bool {
        let (cp, [lx, ly, lz]) = ChunkPos::from_block(x, y, z);
        match self.chunks.get_mut(&cp) {
            Some(c) => {
                c.set(lx, ly, lz, b);
                true
            }
            None => false,
        }
    }

    /// The chunk at `pos` plus its six face neighbours, or None if `pos` itself is not loaded.
    pub fn neighbourhood(&self, pos: ChunkPos) -> Option<Neighbourhood<'_>> {
        let center = self.chunks.get(&pos)?;
        let faces = std::array::from_fn(|i| self.chunks.get(&pos.neighbour(i)));
        Some(Neighbourhood::new(center, faces))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_with(positions: &[ChunkPos]) -> World {
        let mut w = World::new();
        for &p in positions {
            w.insert_chunk(p, Chunk::new_empty());
        }
        w
    }

    #[test]
    fn unloaded_reads_as_air() {
        let w = World::new();
        assert!(w.get_block(100, -5, 7).is_air());
    }

    #[test]
    fn set_on_unloaded_chunk_fails_and_changes_nothing() {
        let mut w = World::new();
        assert!(!w.set_block(0, 0, 0, BlockId::STONE));
        assert_eq!(w.chunk_count(), 0);
    }

    #[test]
    fn negative_coordinates_land_in_the_right_chunk() {
        let mut w = world_with(&[ChunkPos::new(-1, 0, 0)]);
        assert!(w.set_block(-1, 3, 4, BlockId::STONE));
        assert_eq!(w.get_block(-1, 3, 4), BlockId::STONE);
        let c = w.chunk(ChunkPos::new(-1, 0, 0)).unwrap();
        assert_eq!(c.get(15, 3, 4), BlockId::STONE);
    }

    #[test]
    fn insert_returns_previous_chunk() {
        let mut w = World::new();
        assert!(
            w.insert_chunk(ChunkPos::new(0, 0, 0), Chunk::new_empty())
                .is_none()
        );
        assert!(
            w.insert_chunk(ChunkPos::new(0, 0, 0), Chunk::new_empty())
                .is_some()
        );
        assert_eq!(w.chunk_count(), 1);
    }

    #[test]
    fn neighbourhood_sees_loaded_neighbours_only() {
        let mut w = world_with(&[ChunkPos::new(0, 0, 0), ChunkPos::new(1, 0, 0)]);
        w.set_block(16, 3, 4, BlockId::STONE); // first block of the +X chunk

        assert!(w.neighbourhood(ChunkPos::new(5, 5, 5)).is_none());

        let nb = w.neighbourhood(ChunkPos::new(0, 0, 0)).unwrap();
        use crate::view::BlockView;
        assert_eq!(nb.get_signed(16, 3, 4), BlockId::STONE); // loaded +X neighbour
        assert!(nb.get_signed(-1, 3, 4).is_air()); // -X not loaded
    }
}
