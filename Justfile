# Justfile https://just.systems/man/en/

export RUST_LOG := env_var_or_default('RUST_LOG', 'debug')

_default:
    @just --list

# run unit tests with various feature combinations
test *args:
    cargo test -- {{ args }}
    cargo test --no-default-features --features chrono -- {{ args }}
    cargo test --no-default-features --features time -- {{ args }}

# run code lint tools with various feature combinations
lint:
    cargo clippy --tests --examples
    cargo clippy --tests --examples --no-default-features --features chrono
    cargo clippy --tests --examples --no-default-features --features time

# generate documentation
docs *args:
    rm -rf target/doc
    cargo doc --lib --no-deps {{ args }}

# audit dependencies
audit *args:
    cargo install cargo-audit
    cargo audit {{ args }}

# check semver changes
semver *args:
    cargo install cargo-semver-checks
    cargo semver-checks check-release --default-features {{ args }}
    cargo semver-checks check-release --only-explicit-features --features chrono {{ args }}
    cargo semver-checks check-release --only-explicit-features --features time {{ args }}

# clean built binaries and dependencies
clean:
    cargo clean

# refresh cached bank holidays from GOV.UK
refresh-cache:
    cargo run --example download -- src/data_source/bank-holidays.json

# run demo cli
demo *args:
    cargo run --example bank-holidays -- {{ args }}
