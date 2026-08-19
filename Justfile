# Fox and Hounds - Rust & WebAssembly Task Runner

default:
    @just --list

# Build native desktop debug binary
build:
    cargo build

# Run the native app (e.g. `just run --lang ru-RU` or `just run -l es-ES`)
run *args:
    cargo run -- {{args}}

# Build release WebAssembly target and copy WASM binary to web directory
build-wasm:
    cargo build --target wasm32-unknown-unknown --release

install-wasm: build-wasm
    cp target/wasm32-unknown-unknown/release/foxandhounds.wasm web/fox-and-hounds.wasm
    @test -L web/assets || ln -s ../assets web/assets

# Build android image
build-android:
    cargo quad-apk build --release

# Build release Android App Bundle (.aab) for Google Play publishing
build-aab: build-android
    ./scripts/build-aab.sh

# Check for compilation errors
check:
    cargo check

# Run Clippy linter with strict warning checks
clippy:
    cargo clippy -- -D warnings

# Format code using rustfmt
fmt:
    cargo fmt

# Check formatting without making changes
fmt-check:
    cargo fmt --check

# Run tests
test:
    cargo test

# Serve the WASM game locally on port 8080
serve: install-wasm
    python3 -m http.server 8080 -d web

# Run complete CI test suite (formatting, clippy, tests, WASM build)
ci: fmt-check clippy test build-wasm
