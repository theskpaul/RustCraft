run:
    cargo run -p rc-client

check:
    cargo clippy --workspace --all-targets

test:
    cargo nextest run --workspace

gpu:
    WGPU_BACKEND=vulkan mangohud cargo run -p rc-client --release
