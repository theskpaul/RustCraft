use crate::chunk::CHUNK_SIZE;

/// Address of a chunk in chunk units (not blocks). Chunk (1, 0, 0) starts at block x = 16.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// +X, -X, +Y, -Y, +Z, -Z. Must match FACE_NORMALS in rc-render (a test checks this).
const FACE_OFFSETS: [[i32; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

impl ChunkPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// World coordinates of this chunk's block (0, 0, 0).
    pub fn origin(self) -> [i32; 3] {
        let n = CHUNK_SIZE as i32;
        [self.x * n, self.y * n, self.z * n]
    }

    /// The chunk next to this one across `face` (index into the face order above).
    pub fn neighbour(self, face: usize) -> ChunkPos {
        let [dx, dy, dz] = FACE_OFFSETS[face];
        ChunkPos::new(self.x + dx, self.y + dy, self.z + dz)
    }

    /// World block coordinate -> (chunk, position inside that chunk, each 0..16).
    pub fn from_block(x: i32, y: i32, z: i32) -> (ChunkPos, [usize; 3]) {
        let n = CHUNK_SIZE as i32;
        let chunk = ChunkPos::new(x.div_euclid(n), y.div_euclid(n), z.div_euclid(n));
        let local = [x, y, z].map(|v| v.rem_euclid(n) as usize);
        (chunk, local)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_coordinates() {
        assert_eq!(
            ChunkPos::from_block(0, 0, 0),
            (ChunkPos::new(0, 0, 0), [0, 0, 0])
        );
        assert_eq!(
            ChunkPos::from_block(15, 0, 0),
            (ChunkPos::new(0, 0, 0), [15, 0, 0])
        );
        assert_eq!(
            ChunkPos::from_block(16, 0, 0),
            (ChunkPos::new(1, 0, 0), [0, 0, 0])
        );
    }

    #[test]
    fn negative_coordinates() {
        assert_eq!(
            ChunkPos::from_block(-1, 0, 0),
            (ChunkPos::new(-1, 0, 0), [15, 0, 0])
        );
        assert_eq!(
            ChunkPos::from_block(-16, 0, 0),
            (ChunkPos::new(-1, 0, 0), [0, 0, 0])
        );
        assert_eq!(
            ChunkPos::from_block(-17, 0, 0),
            (ChunkPos::new(-2, 0, 0), [15, 0, 0])
        );
    }

    #[test]
    fn origin_plus_local_roundtrips() {
        for i in -40..40 {
            let (x, y, z) = (i, i * 3, -i * 5);
            let (c, l) = ChunkPos::from_block(x, y, z);
            let o = c.origin();
            assert_eq!(
                [o[0] + l[0] as i32, o[1] + l[1] as i32, o[2] + l[2] as i32],
                [x, y, z]
            );
        }
    }

    #[test]
    fn neighbour_steps_one_chunk() {
        let p = ChunkPos::new(2, -1, 5);
        assert_eq!(p.neighbour(0), ChunkPos::new(3, -1, 5)); // +X
        assert_eq!(p.neighbour(3), ChunkPos::new(2, -2, 5)); // -Y
        assert_eq!(p.neighbour(4), ChunkPos::new(2, -1, 6)); // +Z
    }
}
