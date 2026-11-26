#!/usr/bin/env node
import { readFileSync, writeFileSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");

// Read version from package.json (single source of truth)
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const version = pkg.version;
const dateParts = Object.fromEntries(new Intl.DateTimeFormat("en", {
  timeZone: "Asia/Qatar", year: "numeric", month: "2-digit", day: "2-digit",
}).formatToParts(new Date()).map(({ type, value }) => [type, value]));
const today = `${dateParts.year}-${dateParts.month}-${dateParts.day}`;

console.log(`Syncing version ${version} (build date: ${today}) to all files...`);

// 1. Update src/lib/version.ts
const versionTsPath = join(root, "src", "lib", "version.ts");
let versionTs = readFileSync(versionTsPath, "utf8");
versionTs = versionTs.replace(
  /export const LUCID_VERSION = ".*?";/,
  `export const LUCID_VERSION = "${version}";`
);
versionTs = versionTs.replace(
  /export const LUCID_BUILD_DATE = ".*?";/,
  `export const LUCID_BUILD_DATE = "${today}";`
);
writeFileSync(versionTsPath, versionTs);
console.log(`  Updated src/lib/version.ts`);

// 2. Update src-tauri/tauri.conf.json
const tauriConfPath = join(root, "src-tauri", "tauri.conf.json");
let tauriConf = readFileSync(tauriConfPath, "utf8");
tauriConf = tauriConf.replace(
  /"version": ".*?"/,
  `"version": "${version}"`
);
writeFileSync(tauriConfPath, tauriConf);
console.log(`  Updated src-tauri/tauri.conf.json`);

// 3. Update src-tauri/Cargo.toml
const cargoTomlPath = join(root, "src-tauri", "Cargo.toml");
let cargoToml = readFileSync(cargoTomlPath, "utf8");
cargoToml = cargoToml.replace(
  /^(version = )".*?"/m,
  `$1"${version}"`
);
writeFileSync(cargoTomlPath, cargoToml);
console.log(`  Updated src-tauri/Cargo.toml`);

// 4. Update the lucid package in src-tauri/Cargo.lock (else the next cargo
// build rewrites it and the release commit leaves it stale)
const cargoLockPath = join(root, "src-tauri", "Cargo.lock");
const cargoLock = readFileSync(cargoLockPath, "utf8");
const lucidPackage = /(\[\[package\]\]\r?\nname = "lucid"\r?\nversion = )".*?"/;
if (!lucidPackage.test(cargoLock)) {
  throw new Error("src-tauri/Cargo.lock: lucid package entry not found");
}
writeFileSync(cargoLockPath, cargoLock.replace(lucidPackage, `$1"${version}"`));
console.log(`  Updated src-tauri/Cargo.lock`);

console.log(`\nVersion sync complete.`);
