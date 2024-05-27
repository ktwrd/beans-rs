# beans-rs
A rewrite of the original [beans](https://github.com/int-72h/ofinstaller-beans) installer, but in rust!

Currently, everything is a 1:1 port from the python version, and things may be buggy or incomplete.

`beans-rs` is licensed under `GPLv3-only`, so please respect it!

## Developing
Requirements
- Rust Toolchain (nightly, only for building)
  - Recommended to use [rustup](https://rustup.rs/) to install.
- x86-64/AMD64 Processor ([see notes](#notes-binaries))
- **Following requirements are only required for testing**
- Steam Installed
  - Source SDK Base 2013 Multiplayer ([install](steam://instal/243750))

## Notes
### Binaries
All the bundled/embedded binaries are for x86-64/AMD64 systems. We only support that architecture because that's what Open Fortress supports.

Linux Systems not using glibc have not been tested.