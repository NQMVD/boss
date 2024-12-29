jobs := `nproc --all 2>/dev/null || sysctl -n hw.logicalcpu`

_default:
    @just --list
    @echo {{ jobs }} jobs available

# install boss via cargo
install:
    cargo build --release --jobs {{ jobs }}
    cargo install --path . || (gum confirm 'Force install?' && cargo install --path . --force)

# fetch git and update dependencies
@update: && install
    git fetch
    cargo update

# debug a run with pkg
debug pkg:
    clear; cargo run {{ pkg }}; bat boss.log

# simple tests
@test:
    cargo run -- helix && echo "✓ helix passed"; hr
    cargo run -- zellij && echo "✓ zellij passed"; hr
    cargo run -- gum && echo "✓ gum passed"; hr
    cargo run -- gpaste && echo "✓ gpaste passed"; hr
    cargo run -- -q -i || echo "✓ -qi passed"; hr
    cargo run -- helix -i || echo "✓ helix -i passed"; hr
    cargo run -- " "|| echo "✓ no args passed"; hr
    echo 'All Tests passed!'

# increase the version and update the changelog
update-version:
    #!/usr/bin/env bash

    current_version=$(grep -E '^version = "' Cargo.toml | cut -d '"' -f2)
    IFS='.' read -r -a version_parts <<< "$current_version"

    major=${version_parts[0]}
    minor=${version_parts[1]}
    patch=${version_parts[2]}

    choice=$(gum choose --header "  [$current_version] - increase:" '1. MAJOR' '2. MINOR' '3. PATCH')
    if [ $? -eq 130 ]; then
        gum log -l error 'User aborted...'
        exit 1
    fi

    message=$(gum write --header "Changes:" --char-limit 0)
    if [ $? -eq 130 ]; then
        gum log -l error 'User aborted...'
        exit 1
    fi

    case $choice in
        '1. MAJOR')
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        '2. MINOR')
            minor=$((minor + 1))
            patch=0
            ;;
        '3. PATCH')
            patch=$((patch + 1))
            ;;
    esac

    new_version="$major.$minor.$patch"
    sed -i "s/^version = .*$/version = \"$new_version\"/" Cargo.toml

    temp_file=$(mktemp)
    echo -e "## [$new_version] - $(date +%Y-%m-%d)\n\n$message\n" > "$temp_file"
    cat CHANGELOG.md >> "$temp_file"
    mv "$temp_file" CHANGELOG.md

    echo "Version updated to $new_version"
