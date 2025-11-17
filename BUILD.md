# Building for macOS and Windows

## Prerequisites

### For macOS builds:
- Rust toolchain installed (`rustup`)
- macOS development tools (Xcode Command Line Tools)

### For Windows builds:
You have two options:

#### Option 1: Build on Windows (Recommended)
- Install Rust on Windows
- Install Visual Studio Build Tools or Visual Studio with C++ workload

#### Option 2: Cross-compile from macOS (More complex)
- Install Windows target: `rustup target add x86_64-pc-windows-gnu`
- Install MinGW-w64 or use `cargo-xwin` for MSVC toolchain

## Building

### macOS Build

```bash
cd cloud-save-uploader
cargo build --release
```

The binary will be at: `target/release/cloud-save-uploader`

### Windows Build (on Windows)

```bash
cd cloud-save-uploader
cargo build --release
```

The binary will be at: `target/release/cloud-save-uploader.exe`

### Cross-compile from macOS to Windows

#### Using GNU toolchain (MinGW):

1. Install the Windows target:
```bash
rustup target add x86_64-pc-windows-gnu
```

2. Install MinGW-w64:
```bash
brew install mingw-w64
```

3. Create `.cargo/config.toml` in the project root:
```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

4. Build:
```bash
cargo build --release --target x86_64-pc-windows-gnu
```

The binary will be at: `target/x86_64-pc-windows-gnu/release/cloud-save-uploader.exe`

#### Using MSVC toolchain (via cargo-xwin):

1. Install cargo-xwin:
```bash
cargo install cargo-xwin
```

2. Install Windows target:
```bash
rustup target add x86_64-pc-windows-msvc
```

3. Build:
```bash
cargo xwin build --release --target x86_64-pc-windows-msvc
```

## Using Build Scripts

See `build.sh` (macOS) and `build.bat` (Windows) for automated builds.

## GitHub Actions (Recommended)

For automated builds on both platforms, see `.github/workflows/build.yml`.

