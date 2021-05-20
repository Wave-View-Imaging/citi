# Citi
Read and write .cti files.

![Build Status](https://github.com/Wave-View-Imaging/citi/actions/workflows/ci.yml/badge.svg)

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

