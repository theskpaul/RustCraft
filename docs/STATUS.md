M2a: complete (commits 1-6).

Verified (pasted output, 2026-11-01):
- cargo test --workspace: 16 tests pass (rc-core 2, rc-render 12, rc-client 2); clippy clean.
- Intel UHD Graphics (ICL GT1) selected, software=false.
- Test chunk (1036 faces) renders; occlusion correct from one close-up view.
- Fifo, 30 s: 1791 frames, avg 16.76 ms, p50 16.70, p99 20.99, max 138.32 ms.
- Immediate, 30 s: 92433 frames, avg 0.32 ms (3081 fps), p99 3.30 ms. Trivial scene at 1280x720; not a 1080p result.
- Slow frames under Fifo are almost entirely `acquire` (submit <=1.3 ms, present <=0.21 ms).
- Palette and clear color converted sRGB to linear, visually confirmed (2026-11-02).
- Diagnostics: input counters in logs; RC_TRACE per-frame CSV; just bench / just stalls.

Open:
- Not input-caused. Once-per-second set_title roughly triples them (22/23 vs 7/7 per 40 s, conditions alternated). A residual 0.16 to 0.4 stalls/s remains, possibly higher when idle (unverified), with a roughly 2 s cadence in the idle run; cause unknown. Parked until GPU timestamps (2b-3).
- W/A/S/D/Space/Ctrl confirmed via trace; Shift, mouse look, two-step Esc pending.
- No RAM, GPU timestamp, multi-chunk, or 1080p numbers yet.

Next: 2b-1 greedy mesher.
