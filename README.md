# KU Drive

## Project Structure

```
.
├── client          # cross-platform client library
│   ├── Cargo.toml
│   └── src
│       ├── file_server.rs
│       ├── lib.rs
│       ├── main.rs
│       └── net
├── common          # shared library
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
└── server          # server library/binary
    ├── Cargo.toml
    └── src
        └── main.rs
```

## Build & Run

### Build

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repo
git clone https://github.com/{}/20242R0136COSE48001 kudrive
cd kudrive

# Install dependency for all
cargo build

# For kudrive-server
cargo build -p kudrive-server

# For kudrive-client
cargo build -p kudrive-client

# For tauri
cargo install tauri-cli
cd app
cargo tauri build
```

### Run

```bash
# For kudrive-server
cargo run --bin kudrive-server

# For kudrive-client
cargo build --bin kudrive-client

# For tauri
cd app
cargo tauri dev
```

### Dev

```bash
cargo install cargo-watch

# For kudrive-server
cargo watch -x 'run --bin kudrive-server'

# For kudrive-client
cargo watch -x 'run --bin kudrive-client'

# Format
cargo fmt
cargo watch -x fmt -x 'run --bin kudrive-client'

# Test
# RUST_LOG=debug, RUST_BACKTRACE=1
RUST_LOG=info cargo test

# tauri
cargo tauri info
```

## Cross-Compile Notes

### Supported Architecture

- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-gnu`

### Build

```bash
# supported arch : https://doc.rust-lang.org/nightly/rustc/platform-support.html
export TARGET_ARCH="x86_64-unknown-linux-gnu"
rustup target add $TARGET_ARCH
cargo build --target=$TARGET_ARCH --bin kudrive-client
```
