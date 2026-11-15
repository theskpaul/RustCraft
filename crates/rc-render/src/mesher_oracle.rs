//! Test oracle for meshers. Expands a mesh into unit squares ("what surface
//! does it cover?") and compares with a reference computed straight from the
//! chunk's block data. Independent of how any mesher builds quads.

use crate::mesher::{ChunkMesh, FACE_NORMALS, Vertex};
use rc_core::{CHUNK_SIZE, Chunk};

/// One unit square of surface: the solid block cell it belongs to, which of
/// its 6 faces, and the block id. Derives `Ord` so Vecs of these can be sorted
/// and then compared as multisets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnitFace {
    pub cell: [u8; 3],
    pub face: u8,
    pub block: u16,
}

/// Ground truth: every solid cell, every face whose neighbour is air.
pub fn reference_faces(chunk: &Chunk) -> Vec<UnitFace> {
    let n = CHUNK_SIZE as i32;
    let mut out = Vec::new();
    for y in 0..n {
        for z in 0..n {
            for x in 0..n {
                let b = chunk.get_signed(x, y, z);
                if b.is_air() {
                    continue;
                }
                for (face, nrm) in FACE_NORMALS.iter().enumerate() {
                    if chunk
                        .get_signed(x + nrm[0], y + nrm[1], z + nrm[2])
                        .is_air()
                    {
                        out.push(UnitFace {
                            cell: [x as u8, y as u8, z as u8],
                            face: face as u8,
                            block: b.0,
                        });
                    }
                }
            }
        }
    }
    out.sort();
    out
}

/// Expand every quad of `mesh` into the unit squares it covers.
/// Panics if a quad is not a flat, axis-aligned rectangle with one face and
/// one block id; a malformed mesher output should fail loudly here.
pub fn covered_faces(mesh: &ChunkMesh) -> Vec<UnitFace> {
    let mut out = Vec::new();
    for quad in mesh.indices.as_chunks::<6>().0 {
        // Two triangles share two corners: 6 indices, 4 distinct.
        let mut ids = quad.to_vec();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 4, "quad must have 4 distinct vertices");
        let vs: Vec<&Vertex> = ids.iter().map(|&i| &mesh.vertices[i as usize]).collect();

        let (face, block) = (vs[0].face, vs[0].block);
        assert!(vs.iter().all(|v| v.face == face && v.block == block));

        // Face order is +X,-X,+Y,-Y,+Z,-Z, so axis = face/2, positive = even.
        let axis = (face / 2) as usize;
        let positive = face % 2 == 0;
        let plane = vs[0].pos[axis];
        assert!(vs.iter().all(|v| v.pos[axis] == plane), "quad not flat");
        // A +face sits on the far side of its cell, a -face on the near side.
        let cell_axis = if positive { plane - 1 } else { plane };

        let (ua, va) = ((axis + 1) % 3, (axis + 2) % 3);
        let min_max = |a: usize| {
            (
                vs.iter().map(|v| v.pos[a]).min().unwrap(),
                vs.iter().map(|v| v.pos[a]).max().unwrap(),
            )
        };
        let ((u0, u1), (v0, v1)) = (min_max(ua), min_max(va));

        // The 4 corners must be exactly the corners of the bounding rectangle.
        let mut corners: Vec<(u8, u8)> = vs.iter().map(|v| (v.pos[ua], v.pos[va])).collect();
        corners.sort_unstable();
        assert_eq!(
            corners,
            [(u0, v0), (u0, v1), (u1, v0), (u1, v1)],
            "not a rectangle"
        );

        for u in u0..u1 {
            for v in v0..v1 {
                let mut cell = [0u8; 3];
                cell[axis] = cell_axis;
                cell[ua] = u;
                cell[va] = v;
                out.push(UnitFace { cell, face, block });
            }
        }
    }
    out.sort();
    out
}

/// Readable failure: sizes plus the first position where the lists differ.
pub fn assert_same(name: &str, got: &[UnitFace], want: &[UnitFace]) {
    if got == want {
        return;
    }
    let first = got.iter().zip(want).position(|(a, b)| a != b);
    panic!(
        "{name}: got {} unit faces, want {}; first difference at index {first:?}:\n  got  {:?}\n  want {:?}",
        got.len(),
        want.len(),
        first.map(|i| got[i]),
        first.map(|i| want[i]),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::greedy::mesh_chunk_greedy;
    use crate::mesher::mesh_chunk;
    use rc_core::BlockId;

    fn chunk_from(f: impl Fn(usize, usize, usize) -> BlockId) -> Chunk {
        let mut c = Chunk::new_empty();
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    c.set(x, y, z, f(x, y, z));
                }
            }
        }
        c
    }

    fn cases() -> Vec<(&'static str, Chunk)> {
        let stone = |p: bool| if p { BlockId::STONE } else { BlockId::AIR };
        vec![
            ("empty", Chunk::new_empty()),
            ("full", chunk_from(|_, _, _| BlockId::STONE)),
            ("corner block", chunk_from(|x, y, z| stone(x + y + z == 0))),
            (
                "checkerboard",
                chunk_from(|x, y, z| stone((x + y + z) % 2 == 0)),
            ),
            (
                "hash pattern",
                chunk_from(|x, y, z| stone((x * 7 + y * 13 + z * 5) % 3 == 0)),
            ),
            (
                "layers, half chunk",
                chunk_from(|x, y, _| match (x < 8, y) {
                    (false, _) => BlockId::AIR,
                    (true, 0..=3) => BlockId::STONE,
                    (true, 4) => BlockId::DIRT,
                    (true, 5) => BlockId::GRASS,
                    _ => BlockId::AIR,
                }),
            ),
        ]
    }

    #[test]
    fn naive_mesher_matches_reference() {
        for (name, chunk) in &cases() {
            assert_same(
                name,
                &covered_faces(&mesh_chunk(chunk)),
                &reference_faces(chunk),
            );
        }
    }

    #[test]
    fn greedy_mesher_matches_reference() {
        for (name, chunk) in &cases() {
            assert_same(
                name,
                &covered_faces(&mesh_chunk_greedy(chunk)),
                &reference_faces(chunk),
            );
        }
    }

    #[test]
    fn hand_built_wide_quad_expands_to_two_unit_faces() {
        // One +Y quad at y=1 spanning x 0..2, z 0..1: the top of cells (0,0,0), (1,0,0).
        let v = |x, z, corner| Vertex {
            pos: [x, 1, z],
            face: 2,
            block: 1,
            corner,
        };
        let mesh = ChunkMesh {
            vertices: vec![v(0, 1, 0), v(2, 1, 1), v(2, 0, 2), v(0, 0, 3)],
            indices: vec![0, 1, 2, 0, 2, 3],
        };
        let want = vec![
            UnitFace {
                cell: [0, 0, 0],
                face: 2,
                block: 1,
            },
            UnitFace {
                cell: [1, 0, 0],
                face: 2,
                block: 1,
            },
        ];
        assert_same("wide quad", &covered_faces(&mesh), &want);
    }
}
