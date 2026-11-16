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
Next: 2b-2 step 4 = World (HashMap<ChunkPos, Chunk>, get/set block, build a Neighbourhood for a chunk), then step 5 = renderer draws several chunks.
