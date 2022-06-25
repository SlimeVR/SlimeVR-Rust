# SlimeVR Overlay
A SteamVR overlay for rendering the SlimeVR skeletal model from a first person perspective while inside other VR apps.

## Installation

The latest builds are available in the published releases for [windows] and [linux]. Simply give them a download and run them!

[windows]: https://github.com/SlimeVR/SlimeVR-Overlay/releases/download/latest/slimevr_overlay.exe
[linux]: https://github.com/SlimeVR/SlimeVR-Overlay/releases/download/latest/slimevr_overlay

## Contributing

### Installing the dependencies

First, install the `Rust` programming language by following the instructions at
[rustup.rs](https://rustup.rs).

Next, install `libclang`, which is used by the
[`ovr_overlay`](https://crates.io/crates/ovr_overlay) bindings for OpenVR. For windows,
use [this] link. For Linux, simply `sudo apt-get install -y libclang-dev`.

[this]: https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.5/LLVM-14.0.5-win64.exe

You will also need SteamVR installed.

## License
All code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](docs/LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](docs/LICENSE-APACHE))

at your option. This means you can select the license you prefer!

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
