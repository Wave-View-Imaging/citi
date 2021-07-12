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

## Python

### Dev Install
```bash
# Create conda environment
conda create -n wvi --file dev-requirements.txt python=3.8

# Local install
pip install -e .
```

### Run tests
```bash
nosetests ffi/python/tests
```

### Run lint
Note that everything is linted including source and out of source tests.
```bash
flake8
```

## Creating a release

### Create Release
- Determine new release version
- Update `version` and commit
  - `package.version` in `Cargo.toml`
  - `version` in `setup.py`
- Create the tag and push
```bash
git tag -a v1.4 -m "my version 1.4"
git push origin v1.4
```

### Publish to crates.io
- `cargo login`
- `cargo publish --dry-run`
- `cargo publish`
