# beans-rs
A rewrite of the original [beans](https://github.com/int-72h/ofinstaller-beans) installer, but in rust!

Currently, everything is a 1:1 port from the python version, and things may be buggy or incomplete.

`beans-rs` is licensed under `GPLv3-only`, so please respect it!

## Developing
Requirements
- Rust Toolchain (nightly, only for building)
  - Recommended to use [rustup](https://rustup.rs/) to install.
- x86-64/AMD64 Processor ([see notes](#notes-binaries))
- fltk ([please read "FLTK Linux Dependencies"](#fltk-linux-dependencies))
  - (Optional) `fluid` for creating `.fl` files.
- **Following requirements are only required for testing**
- Steam Installed
  - Source SDK Base 2013 Multiplayer ([install](steam://instal/243750))

## Notes
### Binaries
All the bundled/embedded binaries are for x86-64/AMD64 systems. We only support that architecture because that's what Open Fortress supports.

Linux Systems not using glibc have not been tested.

### FLTK Linux Dependencies
When building `beans-rs`, some dependencies are required to build it since we need the build dependencies for fltk.

If your distribution is not listed (or the instructions are broken), please look at [`README.unix.txt` in the fltk repo.](https://github.com/fltk/fltk/blob/master/README.Unix.txt).

#### Debian-based
This includes and Linux Distribution that is based off Debian or Ubuntu. Like;
- [Ubuntu](https://ubuntu.com/),
- [Debian](https://www.debian.org/),
- [Linux Mint](https://www.linuxmint.com/),
- [Zorin OS](https://zorin.com/os/),
- [Pop!_OS](https://pop.system76.com/)

```bash
sudo apt update;
sudo apt-get install -y \
    g++ \
    gdb \
    git \
    make \
    cmake \
    autoconf \
    libx11-dev \
    libglu1-mesa-dev \
    libxft-dev \
    libxcursor-dev \
    libasound2-dev \
    freeglut3-dev \
    libcairo2-dev \
    libfontconfig1-dev \
    libglew-dev \
    libjpeg-dev \
    libpng-dev \
    libpango1.0-dev \
    libxinerama-dev;
```

#### Fedora
```bash
sudo yum groupinstall -y "Development Tools"
sudo yum groupinstall -y "X Software Development"
sudo yum groupinstall -y "C Development Tools and Libraries"
```