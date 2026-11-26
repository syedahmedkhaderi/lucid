# Develop Lucid

The repository currently builds on Apple Silicon macOS. A Windows host is still required to verify Windows audio capture and the installer.

## macOS prerequisites

- Node.js 22.22.2+ or 24.15.0+ (the pinned jsdom dependency rejects older Node 24 releases)
- Rust stable, Xcode Command Line Tools, and CMake
- Optional: Ninja

## Commands

```sh
npm ci
npm test
npm run build
source scripts/macos-env.sh
(cd src-tauri && cargo test --lib)
npx tauri dev
./build-mac.sh --no-install --bundles app
```

`npm run build` creates the frontend bundle in `dist/`. A Tauri build additionally stages the pinned ONNX Runtime and compiles native dependencies. The command above creates `src-tauri/target/release/bundle/macos/Lucid.app` without installing it into `/Applications`. A standalone macOS DMG is also available in the project root. The release script is maintainer-oriented; do not cut or publish releases from this working copy.

## Windows

Use Node.js 22.22.2+ or 24.15.0+, stable Rust with Visual Studio C++ build tools, WebView2, CMake, Ninja, and LLVM. Then run `npm ci`, `npm test`, `npm run build`, `cargo test --lib` from `src-tauri`, and `npx tauri build`. No Windows build has yet been run from this macOS host.

## Secrets and sample data

Use synthetic fixtures only. Enter a provider key through the app's credential settings; never place it in `.env`, source, test fixtures, logs, or Git. The keys pasted in the originating request are intentionally not part of this checkout.
