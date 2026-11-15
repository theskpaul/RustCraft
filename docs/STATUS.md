M2a: complete (commits 1-6).
2b-1: complete (oracle, greedy mesher, client switched to it).

Verified 2b-1 (pasted output, 2026-11-15):
- cargo nextest run --workspace: 38 tests pass (rc-core 2, rc-render 28, rc-client 8); clippy clean.
- Greedy mesher covers the same surface as the reference on 6 oracle cases.
- Test chunk: naive 1036 quads / 4144 vertices; greedy 99 quads / 396 vertices (-90.4%).
- On screen: same scene as with the naive mesher. Shift and mouse look exercised.
- Fifo, debug build, 1280x720, 3379 frames: avg 16.67 ms, p99 20.43, max 39.15. Vsync-bound trivial scene; not a mesher speed result.
