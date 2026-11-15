//! Greedy merging of one 2D slice of faces into rectangles.
//! Pure logic: no GPU, no chunk. The chunk-to-mask part comes in the next step.

use rc_core::CHUNK_SIZE;

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

#[cfg(test)]
mod tests {
    use super::*;

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
