# SlimeVR-Rust
A monorepo for [SlimeVR's](https://github.com/SlimeVR) Rust code.


## Project Structure
A monorepo stores mutliple librares and apps all in one git repo. This one contains:


* [SlimeVR Overlay](overlay/): An OpenVR Overlay that displays the skeletal model of
  SlimeVR for easier debugging and tuning of body proportions
* [Skeletal Model](skeletal_model/): A WIP implementation of a new fullbody FK/IK solver
  with the goal of being callable from several languages, including Typescript(node.js)
  and Java.
* [Autoupdater](autoupdater/): A small utility for updating SlimeVR applications in unison.

## Motivation
* Fulfill the Rewrite It In Rust meme
* Anger Java developers

Jokes aside, the official Java and C++ implementations of SlimeVR are here to
stay. Many people know Java and C++, but there is a growing interest in Rust too. By
providing this repository, we hope to encourage people interested in rust to contribute
to SlimeVR. This may lead to new applications and features that can either work
alongside, or give back to, the mainline Java codebase.


## License
All code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option. This means you can select the license you prefer!

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual
licensed as above, without any additional terms or conditions.
