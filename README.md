# Citi
Read and write .cti files.

![Build Status](https://github.com/Wave-View-Imaging/citi/actions/workflows/ci.yml/badge.svg)
[![Rust Documentation](https://docs.rs/citi/badge.svg)](docs.rs/citi)
[![Cargo](https://img.shields.io/crates/d/citi)](https://crates.io/crates/citi/)

## Development
Since a Docker is not yet available, some setup is needed on your local machine.
Install [Rust](https://www.rust-lang.org/) and execute the following:
```bash
# Add `cargo deny`
cargo install --locked cargo-deny

# Add `clippy`
rustup component add clippy-preview
```

| Check         | Command            |
|:--------------|:-------------------|
| Run tests     | `cargo test`       |
| Lint          | `cargo clippy`     |
| License Check | `cargo deny check` |

## Creating a release

### Create Release
- Determine new release version
- Update `version` in `Cargo.toml`, commit
- Create the tag and push
```bash
git tag -a v1.4 -m "my version 1.4"
git push origin v1.4
```

### Publish to crates.io
- `cargo login`
- `cargo publish --dry-run`
- `cargo publish`
