use bytemuck::{Pod, Zeroable};
use rc_core::{CHUNK_SIZE, Chunk};

/// 8 bytes. `corner` is unused by the shader in 2a (reserved for UV/AO in 2b).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [u8; 3], // chunk-local, 0..=16
    pub face: u8,     // index into FACE_NORMALS
    pub block: u16,
    pub corner: u16,
}
const _: () = assert!(std::mem::size_of::<Vertex>() == 8);

/// +X, -X, +Y, -Y, +Z, -Z
pub const FACE_NORMALS: [[i32; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

/// Corners per face, counter-clockwise when viewed from outside (right-handed, Y up).
const FACE_CORNERS: [[[u8; 3]; 4]; 6] = [
    [[1, 0, 1], [1, 0, 0], [1, 1, 0], [1, 1, 1]], // +X
    [[0, 0, 0], [0, 0, 1], [0, 1, 1], [0, 1, 0]], // -X
    [[0, 1, 1], [1, 1, 1], [1, 1, 0], [0, 1, 0]], // +Y
    [[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]], // -Y
    [[0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1]], // +Z
    [[1, 0, 0], [0, 0, 0], [0, 1, 0], [1, 1, 0]], // -Z
];

#[derive(Default)]
pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl ChunkMesh {
    pub fn face_count(&self) -> usize {
        self.indices.len() / 6
    }
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

/// Naive mesher: one quad per exposed face. Out-of-chunk neighbours are air.
pub fn mesh_chunk(chunk: &Chunk) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    let n = CHUNK_SIZE as i32;
    for y in 0..n {
        for z in 0..n {
            for x in 0..n {
                let b = chunk.get_signed(x, y, z);
                if b.is_air() {
                    continue;
                }
                for (face, nrm) in FACE_NORMALS.iter().enumerate() {
                    if !chunk
                        .get_signed(x + nrm[0], y + nrm[1], z + nrm[2])
                        .is_air()
                    {
                        continue;
                    }
                    let base = mesh.vertices.len() as u32;
                    for (corner, c) in FACE_CORNERS[face].iter().enumerate() {
                        mesh.vertices.push(Vertex {
                            pos: [x as u8 + c[0], y as u8 + c[1], z as u8 + c[2]],
                            face: face as u8,
                            block: b.0,
                            corner: corner as u16,
                        });
                    }
                    mesh.indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
                }
            }
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;
    use rc_core::BlockId;

    fn filled(pred: impl Fn(usize, usize, usize) -> bool) -> Chunk {
        let mut c = Chunk::new_empty();
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    if pred(x, y, z) {
                        c.set(x, y, z, BlockId::STONE);
                    }
                }
            }
        }
        c
    }

    #[test]
    fn empty_chunk_has_no_faces() {
        assert!(mesh_chunk(&Chunk::new_empty()).is_empty());
    }

    #[test]
    fn single_block_has_six_faces() {
        let mut c = Chunk::new_empty();
        c.set(5, 5, 5, BlockId::STONE);
        let m = mesh_chunk(&c);
        assert_eq!(m.face_count(), 6);
        assert_eq!(m.vertices.len(), 24);
        assert_eq!(m.indices.len(), 36);
    }

    #[test]
    fn adjacent_blocks_hide_shared_face() {
        let mut c = Chunk::new_empty();
        c.set(5, 5, 5, BlockId::STONE);
        c.set(6, 5, 5, BlockId::STONE);
        assert_eq!(mesh_chunk(&c).face_count(), 10);
    }

    #[test]
    fn full_chunk_only_boundary_faces() {
        // out-of-chunk = air => 6 sides * 16*16
        assert_eq!(mesh_chunk(&filled(|_, _, _| true)).face_count(), 6 * 256);
    }

    #[test]
    fn checkerboard_exposes_everything() {
        let m = mesh_chunk(&filled(|x, y, z| (x + y + z) % 2 == 0));
        assert_eq!(m.face_count(), 2048 * 6);
    }

    #[test]
    fn indices_in_range() {
        let m = mesh_chunk(&filled(|x, y, z| (x * 7 + y * 13 + z * 5) % 3 == 0));
        assert!(m.indices.iter().all(|&i| (i as usize) < m.vertices.len()));
    }

    #[test]
    fn winding_matches_face_normals() {
        let m = mesh_chunk(&filled(|x, y, z| (x * 7 + y * 13 + z * 5) % 3 == 0));
        assert!(!m.is_empty());
        let p = |v: &Vertex| Vec3::new(v.pos[0] as f32, v.pos[1] as f32, v.pos[2] as f32);
        for tri in m.indices.chunks(3) {
            let v = [
                &m.vertices[tri[0] as usize],
                &m.vertices[tri[1] as usize],
                &m.vertices[tri[2] as usize],
            ];
            assert!(v[0].face == v[1].face && v[1].face == v[2].face);
            let geo = (p(v[1]) - p(v[0])).cross(p(v[2]) - p(v[0])).normalize();
            let e = FACE_NORMALS[v[0].face as usize];
            let exp = Vec3::new(e[0] as f32, e[1] as f32, e[2] as f32);
            assert!(
                (geo - exp).length() < 1e-5,
                "face {} geo {:?}",
                v[0].face,
                geo
            );
        }
    }
}
