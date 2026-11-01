M2a: complete (commits 1-6).

Verified (pasted output, 2026-11-01):
- cargo test --workspace: 16 tests pass (rc-core 2, rc-render 12, rc-client 2); clippy clean.
- Intel UHD Graphics (ICL GT1) selected, software=false.
- Test chunk (1036 faces) renders; occlusion correct from one close-up view.
- Fifo, 30 s: 1791 frames, avg 16.76 ms, p50 16.70, p99 20.99, max 138.32 ms.
- Immediate, 30 s: 92433 frames, avg 0.32 ms (3081 fps), p99 3.30 ms. Trivial scene at 1280x720; not a 1080p result.
- Slow frames under Fifo are almost entirely `acquire` (submit <=1.3 ms, present <=0.21 ms).

Open:
- Cause of Fifo acquire stalls (compositor/presentation suspected, unproven).
- Each control key and two-step Esc not individually confirmed.
- No RAM, GPU-execution-time, or multi-chunk/1080p measurements yet.
- Colors: palette and clear color treated as linear on an sRGB surface (washed out).

Next: 2b (scope to be decided).