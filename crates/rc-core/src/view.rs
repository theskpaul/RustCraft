use crate::block::BlockId;
use crate::chunk::{CHUNK_SIZE, Chunk};

/// Anything that can answer "which block is at this chunk-local position?".
/// Coordinates may be outside 0..16; the implementor decides what lives there.
pub trait BlockView {
    fn get_signed(&self, x: i32, y: i32, z: i32) -> BlockId;
}

impl BlockView for Chunk {
    fn get_signed(&self, x: i32, y: i32, z: i32) -> BlockId {
        Chunk::get_signed(self, x, y, z) // the existing method: outside = air
    }
}

/// A chunk plus its 6 face neighbours (borrowed, not copied).
pub struct Neighbourhood<'a> {
    center: &'a Chunk,
    /// Order: +X, -X, +Y, -Y, +Z, -Z (same as FACE_NORMALS in rc-render).
    faces: [Option<&'a Chunk>; 6],
}

impl<'a> Neighbourhood<'a> {
    pub fn new(center: &'a Chunk, faces: [Option<&'a Chunk>; 6]) -> Self {
        Self { center, faces }
    }
}

impl BlockView for Neighbourhood<'_> {
    fn get_signed(&self, x: i32, y: i32, z: i32) -> BlockId {
        let n = CHUNK_SIZE as i32;
        // For each axis: -1 = before the chunk, 0 = inside, 1 = past the end.
        let side = [x, y, z].map(|v| {
            if v < 0 {
                -1
            } else if v >= n {
                1
            } else {
                0
            }
        });
        // The same position wrapped into 0..16 (16 -> 0, -1 -> 15).
        let [wx, wy, wz] = [x, y, z].map(|v| v.rem_euclid(n) as usize);

        let which = match side {
            [0, 0, 0] => return self.center.get(wx, wy, wz),
            [1, 0, 0] => 0,
            [-1, 0, 0] => 1,
            [0, 1, 0] => 2,
            [0, -1, 0] => 3,
            [0, 0, 1] => 4,
            [0, 0, -1] => 5,
            _ => return BlockId::AIR, // edge/corner: not needed for face culling
        };
        match self.faces[which] {
            Some(c) => c.get(wx, wy, wz),
            None => BlockId::AIR, // neighbour not loaded
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_stone(x: usize, y: usize, z: usize) -> Chunk {
        let mut c = Chunk::new_empty();
        c.set(x, y, z, BlockId::STONE);
        c
    }

    #[test]
    fn inside_reads_center() {
        let c = with_stone(1, 2, 3);
        let nb = Neighbourhood::new(&c, [None; 6]);
        assert_eq!(nb.get_signed(1, 2, 3), BlockId::STONE);
        assert_eq!(nb.get_signed(3, 2, 1), BlockId::AIR);
    }

    #[test]
    fn plus_x_neighbour_is_read_at_x_16() {
        let center = Chunk::new_empty();
        let east = with_stone(0, 3, 4);
        let nb = Neighbourhood::new(&center, [Some(&east), None, None, None, None, None]);
        assert_eq!(nb.get_signed(16, 3, 4), BlockId::STONE);
        assert_eq!(nb.get_signed(16, 3, 5), BlockId::AIR);
    }

    #[test]
    fn minus_x_neighbour_is_read_at_x_minus_1() {
        let center = Chunk::new_empty();
        let west = with_stone(15, 3, 4);
        let nb = Neighbourhood::new(&center, [None, Some(&west), None, None, None, None]);
        assert_eq!(nb.get_signed(-1, 3, 4), BlockId::STONE);
    }

    #[test]
    fn missing_neighbour_is_air() {
        let center = Chunk::new_empty();
        let nb = Neighbourhood::new(&center, [None; 6]);
        assert!(nb.get_signed(16, 0, 0).is_air());
        assert!(nb.get_signed(0, -1, 0).is_air());
    }

    #[test]
    fn edge_and_corner_positions_are_air() {
        let east = with_stone(0, 0, 0);
        let center = Chunk::new_empty();
        let nb = Neighbourhood::new(&center, [Some(&east), None, None, None, None, None]);
        assert!(nb.get_signed(16, 16, 0).is_air()); // two axes outside
    }
}
