# Third-party notices

## NexQ

This project contains and adapts NexQ, Copyright (c) 2026 Vahid Alizadeh, under the MIT License. See [`LICENSE`](LICENSE). The NexQ upstream repository is <https://github.com/naxhq/NexQ>.

## Dependencies

The project retains NexQ's JavaScript and Rust dependencies. Generate the current metadata inventory with `npm run license:inventory` after installing dependencies. The output is checked in under [`licenses/`](licenses/):

- [`lucid-web-production.spdx.json`](licenses/lucid-web-production.spdx.json) — SPDX inventory of production npm dependencies.
- [`lucid-rust-dependencies.json`](licenses/lucid-rust-dependencies.json) — declared license and source metadata for packages in `Cargo.lock`.
- [`lucid-npm-unasserted.json`](licenses/lucid-npm-unasserted.json) — npm packages without a declared license expression, with package license-file paths found in `node_modules` at generation time.
- [`inventory-summary.json`](licenses/inventory-summary.json) — scope and package counts.

The current inventory covers 165 production npm packages and 711 locked Rust packages. All 98 npm packages without a declared SPDX expression had a matching license, licence, copying, or notice file in the installed package tree when scanned. The script records paths; it does not copy or validate the legal content of those files. This metadata inventory is not a legal license review, and required third-party texts/notices have not been assembled and release-checked. Review the upstream license texts, bundled transitive dependencies, and attribution requirements before distribution.
