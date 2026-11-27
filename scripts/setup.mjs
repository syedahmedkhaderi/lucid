#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import process from "node:process";

const platform = process.platform;
const dryRun = process.argv.includes("--dry-run");

function versionTuple(value) {
  return value.split(".").map((part) => Number.parseInt(part, 10));
}

function compareVersion(left, right) {
  for (let index = 0; index < Math.max(left.length, right.length); index += 1) {
    const a = left[index] ?? 0;
    const b = right[index] ?? 0;
    if (a !== b) return a - b;
  }
  return 0;
}

function fail(message) {
  console.error(`Setup cannot continue: ${message}`);
  process.exit(1);
}

function checkCommand(command, args = ["--version"]) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    stdio: "ignore",
    shell: platform === "win32",
  });
  if (result.error || result.status !== 0) fail(`install ${command} and make it available on PATH`);
}

function run(command, args) {
  console.log(`\n> ${command} ${args.join(" ")}`);
  if (dryRun) return;
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: platform === "win32",
  });
  if (result.error) fail(result.error.message);
  if (result.status !== 0) process.exit(result.status ?? 1);
}

const nodeVersion = versionTuple(process.versions.node);
const buildSupported = nodeVersion[0] === 20
  ? compareVersion(nodeVersion, [20, 19, 0]) >= 0
  : nodeVersion[0] === 22
    ? compareVersion(nodeVersion, [22, 12, 0]) >= 0
    : nodeVersion[0] >= 24;
if (!buildSupported) {
  fail(`Node ${process.versions.node} is too old for the Vite build; install Node 20.19+, 22.12+, or 24+`);
}
const jsdomSupported = nodeVersion[0] === 22
  ? compareVersion(nodeVersion, [22, 22, 2]) >= 0
  : nodeVersion[0] === 24
    ? compareVersion(nodeVersion, [24, 15, 0]) >= 0
    : nodeVersion[0] >= 25;
if (!jsdomSupported) {
  console.warn(`Warning: Node ${process.versions.node} is below the pinned jsdom test engine minimum; the app build can proceed, but use Node 22.22.2+ or 24.15.0+ to run the full frontend test suite without an engine warning.`);
}

if (platform !== "darwin" && platform !== "win32") {
  fail("this setup command supports macOS and Windows; build Linux from DEVELOPMENT.md");
}

checkCommand("npm");
checkCommand("cargo", ["--version"]);
checkCommand("rustc", ["--version"]);
checkCommand("cmake", ["--version"]);

if (platform === "darwin") {
  const xcode = spawnSync("xcode-select", ["-p"], { stdio: "ignore" });
  if (xcode.error || xcode.status !== 0) {
    fail("install Xcode Command Line Tools (`xcode-select --install`) before building");
  }
} else {
  const rustInfo = spawnSync("rustc", ["-vV"], { encoding: "utf8" });
  if (rustInfo.error || !rustInfo.stdout.includes("host: x86_64-pc-windows-msvc")) {
    fail("install the x86_64 MSVC Rust toolchain and Visual Studio C++ Build Tools");
  }
}

console.log(`Preparing Lucid on ${platform === "darwin" ? "macOS" : "Windows"}${dryRun ? " (dry run)" : ""}.`);
run("npm", ["ci"]);
run("node", ["scripts/fetch-onnxruntime.mjs"]);

if (platform === "darwin") {
  run("/bin/bash", ["./build-mac.sh", "--bundles", "app"]);
  console.log("\nLucid is installed in /Applications/Lucid.app.");
} else {
  run("npm", ["run", "tauri", "build", "--", "--bundles", "nsis"]);
  console.log("\nLucid's Windows installer is in src-tauri/target/release/bundle/nsis/.");
}
