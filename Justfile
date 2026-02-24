# Justfile https://just.systems/man/en/

export RUST_LOG := env_var_or_default('RUST_LOG', 'debug')

_default:
    @just --list

# run unit tests with various feature combinations
test *args:
    cargo test -- {{ args }}
    cargo test --no-default-features --features chrono -- {{ args }}
    cargo test --no-default-features --features time -- {{ args }}
    cargo test --all-features -- {{ args }}
    @# uncomment once there are some date-independent tests
    @# cargo test --no-default-features -- {{ args }}

# run code lint tools with various feature combinations
lint *args:
    cargo clippy --lib --tests --examples {{ args }}
    cargo clippy --lib --tests --examples --no-default-features --features chrono {{ args }}
    cargo clippy --lib --tests --examples --no-default-features --features time {{ args }}
    cargo clippy --lib --tests --examples --all-features {{ args }}
    @# uncomment once there are some date-independent tests
    @# cargo clippy --lib --tests --examples --no-default-features {{ args }}

# generate coverage report
coverage *args:
    cargo install cargo-llvm-cov
    cargo llvm-cov --all-features --lib --examples {{ args }}

# generate documentation
docs *args:
    rm -rf target/doc
    cargo doc --lib --all-features --no-deps {{ args }}

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
    @# uncomment once all features can be enabled on _previous_ release
    @# cargo semver-checks check-release --all-features {{ args }}
    @# uncomment once all default features can be disabled on _previous_ release
    @# cargo semver-checks check-release --only-explicit-features {{ args }}

# clean built binaries and dependencies
clean:
    cargo clean

# refresh cached bank holidays from GOV.UK
refresh-cache:
    cargo run --example download -- src/data_source/bank-holidays.json

# run demo cli
demo *args:
    cargo run --example bank-holidays -- {{ args }}
