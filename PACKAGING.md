Packaging
=========

This file contains quick reminders and notes on how to package Constellation.

We consider here the packaging flow of Constellation version `1.0.0`, for target architecture `i686` and distribution `debian9` (the steps are alike for `x86_64`):

1. **How to bump Constellation version:**
    1. Bump version in `Cargo.toml` to `1.0.0`
    2. Execute `cargo update` to bump `Cargo.lock`

2. **How to build Constellation for Linux on Debian:**
    1. `apt-get install -y git build-essential`
    2. `curl https://sh.rustup.rs -sSf | sh` (install the `nightly` toolchain)
    3. `git clone https://github.com/valeriansaliou/constellation.git`
    4. `cd constellation/`
    5. `cargo build --release`

3. **How to package built binary:**
    1. `mkdir constellation`
    2. `mv target/release/constellation constellation/`
    3. `strip constellation/constellation`
    4. `cp -r config.cfg constellation/`
    5. `tar -czvf v1.0.0-i686-debian9.tar.gz constellation`
    6. `rm -r constellation/`

4. **How to update other repositories:**
    1. Publish package on Crates: `cargo publish`
