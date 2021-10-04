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
conda create -n citi --file dev-requirements.txt python=3.8

# Source
conda activate citi

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

## C++

### CMake
Use the following command to run `cmake` and generate the build files for a debug build.
Note that this command assumes at the command is invoked from the root directory and that
the directory `./ffi/cpp/build/debug/` exists. Also, the command sets the `BUILD_TESTING`
to `ON`.
```bash
cmake -S ./ -B ./ffi/cpp/build/debug -DCMAKE_BUILD_TYPE=Debug -DBUILD_TESTING=ON
```

Similarly, use the following command to generate a release build without building tests.
```bash
cmake -S ./ -B ./ffi/cpp/build/release -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=ON
```

### Building
Invoke the following command with the path to the build directory to build the project.
```bash
cmake --build <path to build directory>
```

### Testing
If tests were enabled, then the executable for running the tests will be found under the
`tests` directory within the respective `build` directory. For example, the following command
shows how the executable `test_exec` can be invoked on a unix system.
```bash
./ffi/cpp/build/debug/ffi/cpp/tests/test_exec
```

## Creating a release

### Create Release
- Determine new release version
- Update `version` and commit
  - `package.version` in `Cargo.toml`
  - `version` in `setup.py`
  - `VERSION` in `CMakeLists.txt`
- Create the tag and push
```bash
git tag -a v1.4 -m "my version 1.4"
git push origin v1.4
```

### Publish to crates.io
- `cargo login`
- `cargo publish --dry-run`
- `cargo publish`
