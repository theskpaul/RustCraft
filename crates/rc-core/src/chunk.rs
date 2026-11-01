use crate::block::BlockId;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// 16^3 blocks, flat array, index = x + z*16 + y*256.
pub struct Chunk {
    blocks: Box<[BlockId; CHUNK_VOLUME]>,
}

impl Chunk {
    pub fn new_empty() -> Self {
        Self {
            blocks: Box::new([BlockId::AIR; CHUNK_VOLUME]),
        }
    }

    #[inline]
    fn index(x: usize, y: usize, z: usize) -> usize {
        assert!(
            x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE,
            "chunk coord out of range"
        );
        x + z * CHUNK_SIZE + y * CHUNK_SIZE * CHUNK_SIZE
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        self.blocks[Self::index(x, y, z)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, b: BlockId) {
        self.blocks[Self::index(x, y, z)] = b;
    }

    /// Out-of-chunk coordinates return AIR (2a rule; neighbour chunks come in 3).
    #[inline]
    pub fn get_signed(&self, x: i32, y: i32, z: i32) -> BlockId {
        let n = CHUNK_SIZE as i32;
        if x < 0 || y < 0 || z < 0 || x >= n || y >= n || z >= n {
            BlockId::AIR
        } else {
            self.get(x as usize, y as usize, z as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_roundtrip() {
        let mut c = Chunk::new_empty();
        c.set(1, 2, 3, BlockId::STONE);
        assert_eq!(c.get(1, 2, 3), BlockId::STONE);
        assert_eq!(c.get(3, 2, 1), BlockId::AIR);
    }

    #[test]
    fn out_of_range_is_air() {
        let mut c = Chunk::new_empty();
        c.set(0, 0, 0, BlockId::STONE);
        assert!(c.get_signed(-1, 0, 0).is_air());
        assert!(c.get_signed(0, 16, 0).is_air());
        assert!(!c.get_signed(0, 0, 0).is_air());
    }
}
