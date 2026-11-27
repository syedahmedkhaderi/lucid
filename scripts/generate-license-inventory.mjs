import { execFileSync } from "node:child_process";
import { mkdirSync, readdirSync, writeFileSync } from "node:fs";

const runJson = (command, args) => JSON.parse(execFileSync(command, args, {
  encoding: "utf8",
  maxBuffer: 32 * 1024 * 1024,
}));
const writeJson = (path, value) => writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);

const webSbom = runJson("npm", [
  "sbom",
  "--sbom-format=spdx",
  "--sbom-type=application",
  "--omit=dev",
  "--package-lock-only",
]);
const cargoMetadata = runJson("cargo", [
  "metadata",
  "--manifest-path",
  "src-tauri/Cargo.toml",
  "--locked",
  "--format-version",
  "1",
]);

const webPackages = webSbom.packages.filter((pkg) => pkg.name !== "lucid");
const npmUnassertedPackages = webPackages
  .filter((pkg) => !pkg.licenseDeclared || pkg.licenseDeclared === "NOASSERTION")
  .map((pkg) => {
    const packageDir = `node_modules/${pkg.name}`;
    let files = [];
    try {
      files = readdirSync(packageDir);
    } catch (error) {
      if (error.code !== "ENOENT") throw error;
    }
    const licenseFiles = files
      .filter((name) => /^(license|licence|copying|notice)/i.test(name))
      .sort()
      .map((name) => `${packageDir}/${name}`);
    return { name: pkg.name, version: pkg.versionInfo, licenseFiles };
  });
const rustPackages = cargoMetadata.packages
  .filter((pkg) => pkg.source)
  .map((pkg) => ({
    name: pkg.name,
    version: pkg.version,
    license: pkg.license || "NOASSERTION",
    source: pkg.source,
    repository: pkg.repository || null,
  }))
  .sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version));

const summary = {
  generatedFrom: ["package-lock.json", "src-tauri/Cargo.lock"],
  scope: {
    npm: "Production dependency graph from package-lock.json; development dependencies omitted.",
    rust: "All source packages in Cargo.lock, including platform-specific, build, and test dependencies.",
  },
  npmProductionPackages: webPackages.length,
  npmPackagesWithoutDeclaredLicense: npmUnassertedPackages.length,
  npmUnassertedPackagesWithLicenseFiles: npmUnassertedPackages.filter((pkg) => pkg.licenseFiles.length > 0).length,
  rustLockedPackages: rustPackages.length,
  rustPackagesWithoutDeclaredLicense: rustPackages.filter((pkg) => pkg.license === "NOASSERTION").length,
  note: "Declared metadata only; this output is not a legal license review and does not replace required license texts or notices.",
};

mkdirSync("licenses", { recursive: true });
writeJson("licenses/lucid-web-production.spdx.json", webSbom);
writeJson("licenses/lucid-rust-dependencies.json", rustPackages);
writeJson("licenses/lucid-npm-unasserted.json", npmUnassertedPackages);
writeJson("licenses/inventory-summary.json", summary);
process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);
