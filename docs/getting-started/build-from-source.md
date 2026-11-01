---
title: "Prebuilt binaries and building from source"
description: "Release downloads, building with the pinned toolchain and the optional Cargo features."
order: 3
---

## Prebuilt binaries

Download from [Releases](https://github.com/casoon/auditmysite/releases).

- macOS/Linux: `.tar.gz`
- Windows: `.zip`

## Build from source

```bash
git clone https://github.com/casoon/auditmysite.git
cd auditmysite
cargo build --release
./target/release/auditmysite --version
```

### Toolchain

`rust-toolchain.toml` pins the exact Rust version used for local development, CI, and release builds; rustup picks it up automatically inside the repository. There is no MSRV promise: the pin is raised deliberately, and older compilers are not supported.

**Optional Cargo features:**

| Feature | What it adds | Build command |
|---------|-------------|---------------|
| `pdf` | PDF report generation via the `renderreport`/Typst engine | `cargo build --release --features pdf` |
| `pdf_test` | PDF rendering integration tests | `cargo test --features pdf_test` |
| `ai-transparency` | C2PA image-provenance check (EU AI Act Art. 50, opt-in `--ai-transparency` flag, single-URL mode only) | `cargo build --release --features ai-transparency` |
