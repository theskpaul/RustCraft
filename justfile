run:
    cargo run -p rc-client

check:
    cargo clippy --workspace --all-targets

test:
    cargo nextest run --workspace

gpu:
    WGPU_BACKEND=vulkan mangohud cargo run -p rc-client --release

# Frame-time capture with the known stall sources off (title updates, terminal logging).
# Trace goes to /tmp/rc-trace.csv, log to /tmp/rc-bench.log. Keep hands off the input.
bench secs="30":
    RC_NO_TITLE=1 RC_TRACE=/tmp/rc-trace.csv RC_EXIT_AFTER={{secs}} cargo run --release -p rc-client 2>/tmp/rc-bench.log

# Count frames whose own acquire exceeded 19 ms (~2.5 ms over a normal vsync wait).
stalls file="/tmp/rc-trace.csv":
    awk -F, 'NR>1 && $4>19 {n++} END{printf "%d stalls (acquire>19ms) / %d frames\n", n, NR-1}' {{file}}
