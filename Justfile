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
    @test -e web/assets || ln -s ../assets web/assets

# Package the whole web/ directory (wasm binary + assets) into fox-and-hounds.zip
zip-wasm: install-wasm
    zip -r fox-and-hounds.zip web

# Build android image
build-android:
    cargo quad-apk build --release

# Build release Android App Bundle (.aab) for Google Play publishing
build-aab: build-android
    ./scripts/build-aab.sh

# Build debug APK with separate application ID (installs alongside Play Store version)
build-android-debug:
    ./scripts/build-debug-apk.sh

# Check for compilation errors
check:
    cargo check

# Run Clippy linter with strict warning checks
clippy:
    cargo clippy --all-targets -- -D warnings

# Format code using rustfmt
fmt:
    cargo fmt

# Check formatting without making changes
fmt-check:
    cargo fmt --check

# Run tests
test *args:
    cargo test {{args}}

# Serve the WASM game locally on port 8080
serve: install-wasm
    python3 -m http.server 8080 -d web

# Run complete CI test suite (formatting, clippy, tests, wasm target check)
ci: fmt-check clippy test
    cargo check --target wasm32-unknown-unknown

# Re-subset font from system Arial Unicode MS for all current locale strings
subset-font:
    @if [ -f scratch/venv/bin/python ]; then \
        scratch/venv/bin/python scripts/subset-font.py; \
    elif [ -f .venv/bin/python ]; then \
        .venv/bin/python scripts/subset-font.py; \
    else \
        python3 scripts/subset-font.py; \
    fi

