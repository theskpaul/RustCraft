M2a: complete (commits 1-6).
2b-1: complete (oracle, greedy mesher, client switched to it).

Verified 2b-1 (pasted output, 2026-11-15):
- cargo nextest run --workspace: 38 tests pass (rc-core 2, rc-render 28, rc-client 8); clippy clean.
- Greedy mesher covers the same surface as the reference on 6 oracle cases.
- Test chunk: naive 1036 quads / 4144 vertices; greedy 99 quads / 396 vertices (-90.4%).
- On screen: same scene as with the naive mesher. Shift and mouse look exercised.
- Fifo, debug build, 1280x720, 3379 frames: avg 16.67 ms, p99 20.43, max 39.15. Vsync-bound trivial scene; not a mesher speed result.

2b-2 step 1: complete (BlockView, Neighbourhood in rc-core).
2b-2 step 2: complete (both meshers generic over BlockView; border-face tests).
2b-2 step 3: complete (ChunkPos, from_block with div_euclid/rem_euclid, face-order test).
Verified 2026-11-15 (pasted output): cargo nextest run --workspace: 50 tests pass (rc-core 11, rc-render 31, rc-client 8); clippy clean.
2b-2 step 4: complete (World).
2b-2 step 5a: complete (per-chunk origin bind group).
2b-2 step 5b: complete (3x3 patch via World + neighbourhood meshing).
Verified 2026-11-16 (pasted output + screenshot): nextest 55 pass; world: 9 chunks, greedy quads 651 with neighbours vs 867 isolated; no visible seams; Fifo, debug, avg 16.69 ms, p99 22.24, max 43.49.
2b-2: complete. Benchmark (release, Fifo, 1280x720, RC_NO_TITLE=1, idle, 2026-11-16, pasted): 1801 frames, avg 16.67 ms, p50 16.66, p99 18.35, max 33.12; 6 frames with acquire > 19 ms (0.3%). submit ~0.5 ms. Vsync-bound; stall cause between debug/title/logging/input not isolated.
