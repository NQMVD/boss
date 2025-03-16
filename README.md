<div align="center">
  <h1>
    <a href="https://github.com/NQMVD/boss">boss<a/>
  </h1>
  <h3></h3>
  <i>The boss of package management.</i>
  <h3></h3>
</div>

<div align="center">

![](https://img.shields.io/github/last-commit/NQMVD/boss?&style=for-the-badge&color=b1ffb4&logoColor=D9E0EE&labelColor=292324)
![](https://img.shields.io/badge/Rust-fe7a15?style=for-the-badge&logo=rust&logoColor=white&logoSize=auto&labelColor=292324)
[![](https://img.shields.io/crates/v/boss-cli.svg?style=for-the-badge&logo=rust&logoColor=white&logoSize=auto&labelColor=292324)](https://crates.io/crates/boss-cli)
![](https://img.shields.io/badge/Linux-E95420?style=for-the-badge&logo=linux&logoColor=white&logoSize=auto&labelColor=292324)
[![](https://img.shields.io/badge/just-white?style=for-the-badge&logo=just&color=black)](https://just.systems)
</a>

</div>

<div align="center">
  <img alt="boss shot" src="https://github.com/NQMVD/boss/blob/main/.assets/boss_shot.png?raw=true" />
</div>

<div align="center">
  <img alt="table preview" src="https://github.com/NQMVD/boss/blob/main/.assets/table_preview.png?raw=true" />
</div>

## Usage

Just give it a package name and it will collect info about it from your
available package managers. It will show you an entry for each manager: a state
and an optional version. There is three states:

1. Installed
2. Not installed, but available
3. Not available (not found)

Oh and it sorts the results based on the state. And when the new table format is
finally implemented it will be as compact and clean as possible.

> [!WARNING] `boss` is still in development and far from being fully featured.

> [!TIP]
> ... but you can already use it anyways! Let me know how you like it.

## Showcase

<details>
  <summary>open sesame</summary>

## helix query

![default.tape](https://github.com/NQMVD/boss/blob/main/.assets/tapes/default.gif?raw=true)

## helix query --interactive

![interactive.tape](https://github.com/NQMVD/boss/blob/main/.assets/tapes/interactive.gif?raw=true)

## shows latest version and installed version

![newversion.tape](https://github.com/NQMVD/boss/blob/main/.assets/tapes/newversion.gif?raw=true)

## stays quiet for scripts

![quiet.tape](https://github.com/NQMVD/boss/blob/main/.assets/tapes/quiet.gif?raw=true)

</details>

## Roadmap

> theres also a [mind map](https://github.com/NQMVD/boss/blob/main/.assets/boss_map.jpg?raw=true) that's a little more
> structured

- [x] check all available package managers for a given package:
  - [x] if it's **installed**,
  - [x] if not, if it's **available to download** with a manager.
- [x] show descriptions for available packages
- [x] show the **latest version** of the package
- [x] show the **installed version** of the package
- [x] quiet flag to only return with 0 or 1
- [x] interative flag to prompt the user
- [ ] continue with a prompt what to do (install, update, etc.)
- [ ] check for similar package names (like `pkg-cli`, `pkg-git`, `pkg-bin`,
      `pkg-2`)
- [ ] preferences (sorting of order of managers)
- [ ] outputs:
  - [x] pretty cliclack
  - [ ] plain (dont use cliclack for output but plain text or markdown)
  - [ ] table (use nu)
- [ ] read files instead of calling commands when possible
- [ ] check mutiple packages
- [ ] config file
- [ ] cache results for a day
- [ ] more checks (validate location, sourced in path, etc.)
- [ ] multithreading or async (main bottleneck right now are the individual
      managers)
- [ ] taking inspiration from topgrade on how to work with different managers.

## Support

#### General

- [x] apt
- [x] snap
- [ ] yay
- [ ] flatpak
- [ ] brew?
- [ ] pacman (if yay is not installed)
- [ ] paru (if yay is not installed)
- [ ] dnf?
- [ ] rpm?
- [ ] zypper?
- [ ] nix?

#### Language specific

- [x] cargo
- [ ] go (disabled for now)
- [ ] npm
- [ ] yarn?
- [ ] pip
- [ ] pypi?
- [ ] pipx?
- [ ] gem?

## Installation

### From source

> with cargo (recommended)

```bash
cargo install boss-cli
```

> [!NOTE]
> the crate for boss is called [boss-cli](https://crates.io/crates/boss-cli) as
> there's already a baseball progam called
> [boss](https://crates.io/crates/boss)...

> with cargo via git

```bash
cargo install --git https://github.com/NQMVD/boss.git
```

> clone the repo (with gh)

```bash
gh repo clone NQMVD/boss
cd boss
cargo install --path .
```

> clone the repo

```bash
git clone https://github.com/NQMVD/boss
cd boss
cargo install --path .
```

> [!NOTE]
> binaries will be included at some point...

## Update

### From source

> with cargo

```bash
cargo install boss-cli
```

> with [cargo-update](https://crates.io/crates/cargo-update)

```bash
cargo install-update boss-cli
```

> update the repo

```bash
git pull
cargo install --path .
```

## Details

- uses rust because of string processing capabilities and safety
- uses cliclack for the pretty structured output
- uses strp for parsing the command outputs
- calls shell commands
- works on Linux
- might work on macOS (will test with darling soon)
- won't work on Windows (also not planned to do so...)
