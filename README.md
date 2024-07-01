# boss

![boss_shot_final_HD](https://github.com/NQMVD/boss/assets/99403507/d7f5983f-d603-4e80-80e5-3bba4dd46cf5)

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
- modes
  - current one
  - minimal (exclude `[not found]`)
  - pipable (dont use cliclack for output but plain text or md)
- read files instead of calling commands when possible

---

## Support
### Currently
- yay
- apt
- cargo
- go (only installed)

### Planned
- pip
- pacman (if yay is not installed)
- paru (if yay is not installed)
- npm
- dnf?
- brew?

---

## Details
- uses cliclack for the pretty structured output
- calls shell commands (for now)
- works on Linux
- might work on macOS
- won't work on Windows (also not planned to do so...)
