# Reco
RF Imaging Reconstruction Algorithms

![Build Status](https://github.com/Wave-View-Imaging/Reco/actions/workflows/ci.yml/badge.svg)

## Development
Since a Docker is not yet available, some setup is needed on your local machine.
Install [Rust](https://www.rust-lang.org/) and execute the following:
```bash
# Add `cargo deny`
cargo install --locked cargo-deny

# Add `clippy`
rustup component add clippy-preview
```

### Run tests
```bash
cargo test
```

### Run linter
```bash
cargo clippy
```

### Run license check
```bash
cargo deny check
```
