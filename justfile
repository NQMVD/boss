dev:
    zellij -l rustdev

build:
    cargo build --release

test:
    cargo run zellij && cargo run gum && echo "SUCCESS!"

install:
    cargo build --release && sudo cp -v ./target/release/boss /usr/local/bin/
