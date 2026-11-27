# Develop Lucid

The one-command bootstrap is `npm run setup`. On macOS it installs the app in `/Applications`; on Windows it builds the NSIS installer. Windows runtime and audio still need testing on a Windows machine.

## macOS prerequisites

- Node.js 22.22.2+ or 24.15.0+ (the pinned jsdom dependency rejects older Node 24 releases)
- Rust stable, Xcode Command Line Tools, and CMake
- Optional: Ninja

## Commands

```sh
npm run setup
```

The setup command checks the host toolchain, installs the locked Node dependencies, stages ONNX Runtime, and builds the platform app. `npm run setup -- --dry-run` checks prerequisites and prints the commands without building.

For frontend/backend development after setup:

```sh
npm ci
npm test
npm run build
source scripts/macos-env.sh
(cd src-tauri && cargo test --lib)
npx tauri dev
./build-mac.sh --no-install --bundles app
```

`npm run build` creates the frontend bundle in `dist/`. A Tauri build additionally stages the pinned ONNX Runtime and compiles native dependencies. The final command creates `src-tauri/target/release/bundle/macos/Lucid.app` without installing it into `/Applications`. The verified macOS DMG is under `src-tauri/target/release/bundle/dmg/`. The release script is maintainer-oriented; do not cut or publish releases from this working copy.

## Windows

Use Node.js 22.22.2+ or 24.15.0+, stable x86_64 MSVC Rust, Visual Studio C++ build tools, WebView2, CMake, Ninja, and LLVM. Run `npm run setup` to produce the NSIS installer. No Windows build has yet been run from this macOS host.

## Secrets and sample data

Use synthetic fixtures only. Enter a provider key through the app's credential settings; never place it in `.env`, source, test fixtures, logs, or Git. The keys pasted in the originating request are intentionally not part of this checkout.
