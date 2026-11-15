//! Greedy merging of one 2D slice of faces into rectangles.
//! Pure logic: no GPU, no chunk. The chunk-to-mask part comes in the next step.
use crate::mesher::{ChunkMesh, FACE_NORMALS, Vertex};
use rc_core::{CHUNK_SIZE, Chunk};

const N: usize = CHUNK_SIZE;

/// One slice of faces, indexed `mask[v][u]` (row v, column u).
/// `Some(block)` = a face to draw for that block id; `None` = no face.
pub type Mask = [[Option<u16>; N]; N];

/// A rectangle of identical faces. `(u, v)` is its top-left cell,
/// `w` cells wide (along u), `h` cells tall (along v).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub u: u8,
    pub v: u8,
    pub w: u8,
    pub h: u8,
    pub block: u16,
}

/// Cover every `Some` cell of `mask` with rectangles, each cell exactly once.
pub fn merge_mask(mask: &Mask) -> Vec<Rect> {
    let mut done = [[false; N]; N]; // cells already covered by a rectangle
    let mut rects = Vec::new();

    for v in 0..N {
        for u in 0..N {
            if done[v][u] {
                continue;
            }
            let Some(block) = mask[v][u] else {
                continue;
            };

            // "Can the cell at (uu, vv) join the rectangle I'm growing?"
            let matches = |uu: usize, vv: usize| !done[vv][uu] && mask[vv][uu] == Some(block);

            // Grow right as far as the faces match.
            let mut w = 1;
            while u + w < N && matches(u + w, v) {
                w += 1;
            }
            // Grow down while the whole next row (w cells) matches.
            let mut h = 1;
            while v + h < N && (0..w).all(|du| matches(u + du, v + h)) {
                h += 1;
            }

            // Mark the rectangle's cells as covered.
            for dv in 0..h {
                for du in 0..w {
                    done[v + dv][u + du] = true;
                }
            }
            rects.push(Rect {
                u: u as u8,
                v: v as u8,
                w: w as u8,
                h: h as u8,
                block,
            });
        }
    }
    rects
}

/// The mask for one face direction at one layer.
/// `face` is an index into FACE_NORMALS (+X,-X,+Y,-Y,+Z,-Z).
/// `layer` is the coordinate along the face's axis (0..16).
/// A cell is `Some(block)` if that block is solid and its neighbour in the
/// face direction is air (the same rule the naive mesher uses).
pub fn build_mask(chunk: &Chunk, face: usize, layer: usize) -> Mask {
    let axis = face / 2; // 0 = x, 1 = y, 2 = z
    let (ua, va) = ((axis + 1) % 3, (axis + 2) % 3); // the two other axes
    let nrm = FACE_NORMALS[face];

    std::array::from_fn(|v| {
        std::array::from_fn(|u| {
            let mut p = [0i32; 3];
            p[axis] = layer as i32;
            p[ua] = u as i32;
            p[va] = v as i32;

            let block = chunk.get_signed(p[0], p[1], p[2]);
            if block.is_air() {
                return None;
            }
            let next = chunk.get_signed(p[0] + nrm[0], p[1] + nrm[1], p[2] + nrm[2]);
            if next.is_air() { Some(block.0) } else { None }
        })
    })
}

/// Greedy mesher: same output contract as `mesher::mesh_chunk`,
/// but adjacent identical faces are merged into one quad.
pub fn mesh_chunk_greedy(chunk: &Chunk) -> ChunkMesh {
    let mut mesh = ChunkMesh::default();
    for face in 0..6 {
        for layer in 0..N {
            let mask = build_mask(chunk, face, layer);
            for rect in merge_mask(&mask) {
                push_quad(&mut mesh, face, layer, &rect);
            }
        }
    }
    mesh
}

/// Append one rectangle as 4 vertices + 6 indices.
fn push_quad(mesh: &mut ChunkMesh, face: usize, layer: usize, r: &Rect) {
    let axis = face / 2;
    let positive = face.is_multiple_of(2);
    let (ua, va) = ((axis + 1) % 3, (axis + 2) % 3);
    // A + face sits on the far side of its cell, a - face on the near side.
    let plane = (layer + positive as usize) as u8;

    let (u0, v0) = (r.u, r.v);
    let (u1, v1) = (r.u + r.w, r.v + r.h);
    // Counter-clockwise as seen from outside. The - faces look the other
    // way, so their corner order is reversed.
    let corners: [(u8, u8); 4] = if positive {
        [(u0, v0), (u1, v0), (u1, v1), (u0, v1)]
    } else {
        [(u0, v0), (u0, v1), (u1, v1), (u1, v0)]
    };

    let base = mesh.vertices.len() as u32;
    for (i, &(u, v)) in corners.iter().enumerate() {
        let mut pos = [0u8; 3];
        pos[axis] = plane;
        pos[ua] = u;
        pos[va] = v;
        mesh.vertices.push(Vertex {
            pos,
            face: face as u8,
            block: r.block,
            corner: i as u16,
        });
    }
    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rc_core::BlockId;

    fn count(m: &Mask) -> usize {
        m.iter().flatten().filter(|c| c.is_some()).count()
    }

    #[test]
    fn full_chunk_is_six_quads() {
        let mut c = Chunk::new_empty();
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    c.set(x, y, z, BlockId::STONE);
                }
            }
        }
        let m = mesh_chunk_greedy(&c);
        assert_eq!(m.face_count(), 6); // naive gives 1536
        assert_eq!(m.vertices.len(), 24);
    }

    #[test]
    fn greedy_winding_matches_face_normals() {
        let mut c = Chunk::new_empty();
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    if (x * 7 + y * 13 + z * 5) % 3 == 0 {
                        c.set(x, y, z, BlockId::STONE);
                    }
                }
            }
        }
        let m = mesh_chunk_greedy(&c);
        assert!(!m.is_empty());
        let p = |v: &Vertex| glam::Vec3::new(v.pos[0] as f32, v.pos[1] as f32, v.pos[2] as f32);
        for tri in m.indices.chunks(3) {
            let v = [
                &m.vertices[tri[0] as usize],
                &m.vertices[tri[1] as usize],
                &m.vertices[tri[2] as usize],
            ];
            let geo = (p(v[1]) - p(v[0])).cross(p(v[2]) - p(v[0])).normalize();
            let e = FACE_NORMALS[v[0].face as usize];
            let exp = glam::Vec3::new(e[0] as f32, e[1] as f32, e[2] as f32);
            assert!(
                (geo - exp).length() < 1e-5,
                "face {} geo {:?}",
                v[0].face,
                geo
            );
        }
    }

    #[test]
    fn single_block_top_face_lands_in_expected_cell() {
        let mut c = Chunk::new_empty();
        c.set(5, 6, 7, BlockId::STONE);
        // face 2 = +Y, so axis = 1, u runs along z, v runs along x
        let m = build_mask(&c, 2, 6);
        assert_eq!(m[5][7], Some(1)); // v = x = 5, u = z = 7
        assert_eq!(count(&m), 1);
        assert_eq!(count(&build_mask(&c, 2, 5)), 0); // other layer: nothing
    }

    #[test]
    fn covered_face_is_not_in_mask() {
        let mut c = Chunk::new_empty();
        c.set(5, 6, 7, BlockId::STONE);
        c.set(5, 7, 7, BlockId::STONE); // sits on top of the first
        assert_eq!(count(&build_mask(&c, 2, 6)), 0); // top of the lower block is hidden
        assert_eq!(build_mask(&c, 2, 7)[5][7], Some(1)); // top of the upper block shows
    }

    fn mask_from(f: impl Fn(usize, usize) -> Option<u16>) -> Mask {
        std::array::from_fn(|v| std::array::from_fn(|u| f(u, v)))
    }

    #[test]
    fn empty_mask_gives_no_rects() {
        assert!(merge_mask(&mask_from(|_, _| None)).is_empty());
    }

    #[test]
    fn full_mask_gives_one_rect() {
        let rects = merge_mask(&mask_from(|_, _| Some(1)));
        assert_eq!(
            rects,
            vec![Rect {
                u: 0,
                v: 0,
                w: 16,
                h: 16,
                block: 1
            }]
        );
    }

    #[test]
    fn different_blocks_are_not_merged() {
        // left half block 1, right half block 2
        let rects = merge_mask(&mask_from(|u, _| Some(if u < 8 { 1 } else { 2 })));
        assert_eq!(rects.len(), 2);
    }

    #[test]
    fn staircase_shape() {
        // row 0: columns 0..4 filled; row 1: columns 0..2 filled
        let rects = merge_mask(&mask_from(|u, v| match (u, v) {
            (0..=3, 0) | (0..=1, 1) => Some(1),
            _ => None,
        }));
        assert_eq!(
            rects,
            vec![
                Rect {
                    u: 0,
                    v: 0,
                    w: 4,
                    h: 1,
                    block: 1
                },
                Rect {
                    u: 0,
                    v: 1,
                    w: 2,
                    h: 1,
                    block: 1
                },
            ]
        );
    }

    #[test]
    fn checkerboard_cannot_merge() {
        let rects = merge_mask(&mask_from(
            |u, v| if (u + v) % 2 == 0 { Some(1) } else { None },
        ));
        assert_eq!(rects.len(), 128);
    }
}
