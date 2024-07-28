# boss

![boss_shot](https://github.com/NQMVD/boss/assets/99403507/d7f5983f-d603-4e80-80e5-3bba4dd46cf5)

_The boss of package management._

## Features
### Currently
- checks all available package managers for a given package:
  - if its **installed**,
  - if not, if its **available to download** with a manager.
- also shows descriptions (only for cargo right now).

### Planned
- show descriptions for each managers result
- check for similar package names (like `pkg-cli`, `pkg-git`, `pkg-bin`)
- preferences (sorting of order of managers)
- outputs:
  - pretty cliclack
  - plain (dont use cliclack for output but plain text or md)
- read files instead of calling commands when possible
- check mutiple packages at once
- continue with a prompt what to do (install, update, etc.)
- multithreading or async

> (also see the [mindmap](todo.hmm))

---

## Support
### Currently
#### General
- yay
- apt

#### Language specific
- cargo
- go (only installed)

### Planned
#### General
- snap
- flatpak
- brew?
- pacman (if yay is not installed)
- paru (if yay is not installed)
- dnf?
- rpm?
- zypper?
- nix?

#### Language specific
- npm
- yarn?
- pip
- pypi?
- pipx?
- conda?
- gem?

---

## Details
- uses rust because of string processing capabilities
- uses cliclack for the pretty structured output
- uses strp for parsing the command outputs
- calls shell commands (for now)
- works on Linux
- might work on macOS (will test with darling soon)
- won't work on Windows (also not planned to do so...)
