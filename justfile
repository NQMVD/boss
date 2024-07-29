jobs := `echo $(($(nproc) * 2 ))`

_default:
    @just --list
    @echo {{ jobs }} jobs available

# build
build:
    cargo build --jobs {{ jobs }}

# debug a run with pkg
debug pkg:
    clear; cargo run {{ pkg }}; bat boss.log

# test by running with helix, zellij and gum
test:
    @cargo run helix
    @cargo run zellij
    @cargo run gum

# install the binary to /usr/local/bin
install:
    cargo build --release --jobs {{ jobs }}
    sudo cp -v ./target/release/boss /usr/local/bin/
