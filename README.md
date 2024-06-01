# boss

![boss_shot-modified](https://github.com/NQMVD/boss/assets/99403507/20c5df7b-7560-4e02-b82d-91149abb116b)

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
