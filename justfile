# zellij start
dev:
    zellij -l rustdev

build:
    cargo build --jobs 12

debug pkg:
    clear; cargo run {{ pkg }}; bat boss.log

# test by running with zellij and gum
test:
    cargo run zellij && cargo run gum && echo "SUCCESS!"

# install the binary to /usr/local/bin
install:
    cargo build --release --jobs 12 && sudo cp -v ./target/release/boss /usr/local/bin/
