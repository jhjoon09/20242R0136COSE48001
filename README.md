# KU Drive

## Project Documentation

### [Project Report](./docs/report.pdf)

## Project Structure

```
.
├── app
│   ├── package.json
│   ├── public
│   ├── src         # tauri frontend
│   └── src-tauri   # tauri backend
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

# For kudrive-client-core
cargo build -p kudrive-client

# For App
cargo install tauri-cli
cd app
npm install
cargo tauri build --target $TARGET_ARCH
```

### Run

```bash
# For kudrive-server
cargo run --bin kudrive-server

# For kudrive-client
cargo build --bin kudrive-client

# For app
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

### Tested Architecture

- Linux : `x86_64-unknown-linux-gnu`
- Windows : `x86_64-pc-windows-msvc`
- macOS : `aarch64-apple-darwin` / `x86_64-apple-darwin`
- iOS : `aarch64-apple-ios` / `x86_64-apple-ios`
- Android : `aarch64-linux-android`

### App Backend Build

```bash
# supported arch : https://doc.rust-lang.org/nightly/rustc/platform-support.html
export TARGET_ARCH="x86_64-unknown-linux-gnu"
rustup target add $TARGET_ARCH
cargo build --target=$TARGET_ARCH --bin kudrive-client
```

### iOS

```bash
# Requisites
rustup target add aarch64-apple-ios x86_64-apple-ios

# Init
cargo tauri ios init
cargo tauri icon /path/to/app-icon.png
# Xcode : Signing - Provisioning Profile

# ipa Build
cargo tauri ios build # ipa

# Dev
cargo tauri ios dev
```

### Android

```bash
# Requisites
# Android Studio install
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android
export ANDROID_HOME=/path/to/android/sdk
export NDK_HOME=/path/to/android/sdk/ndk/ver

# Init
cargo tauri android init

# Run Emulator
emulator -list-avds
emulator -avd $EMULATOR_NAME

# Build
cargo tauri android build

# Dev
cargo tauri android dev
```

### Windows

```bash
# Requisites
# Install MSVC
rustup target add x86_64-pc-windows-msvc

# Build
cargo tauri build --target x86_64-pc-windows-msvc
```

### macOS

```bash
# Requisites
rustup target add aarch64-apple-darwin

# Build
cargo tauri build --target aarch64-apple-darwin
```
