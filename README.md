# boss

![boss_shot_final_HD](https://github.com/NQMVD/boss/assets/99403507/d7f5983f-d603-4e80-80e5-3bba4dd46cf5)

_The boss of package management._

## Features
- checks all available package managers for a given package:
  - if its **installed**,
  - if not, if its **available to download** with a manager.
- also shows descriptions when needed (flag for that is planned).

## Support
### Currently
- yay
- cargo
- go (only installed)

### Planned
- apt (apt-get)
- pip
- dnf
- npm

## Details
- uses cliclack for the pretty structured output.
- only works on Linux
- might work on macOS
- won't work on Windows (also not planned to do so...)
