# just manual: https://github.com/casey/just/#readme

_default:
    @just --list

# Runs clippy on the sources
check:
    cargo clippy --locked --tests  -- -D warnings

# removes all build artifacts
clean:
    rm -r target

# builds esc
build:
    ESC_CLIENT_VERSION={{`cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "esc") | .version'` }} cargo build --locked

# builds in release mode
build-release:
    ESC_CLIENT_VERSION={{`cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "esc") | .version'` }} cargo build --locked --release

# Formats the code base
fmt:
    cargo fmt

# runs tests
test:
    cargo test --locked

# runs the same checks performed by the CI job
ci:
    just fmt
    git diff --exit-code
    just check
    just build
    just test
