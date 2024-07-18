# zellij start
dev:
    zellij -l rustdev

build:
    cargo build

# test by running with zellij and gum
test:
    cargo run zellij && cargo run gum && echo "SUCCESS!"

# install the binary to /usr/local/bin
install:
    cargo build --release && sudo cp -v ./target/release/boss /usr/local/bin/
