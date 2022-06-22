# Ferrous SlimeVR
A Rust implementation of [SlimeVR](https://github.com/SlimeVR)

## Current Status
The [skeletal model](skeletal_model/) is in the early stages of development. If you're
really passionate about Rust and want to make this a reality, feel free to reach out to
me on the official SlimeVR discord for questions.

## Motivation
* Fulfill the Rewrite It In Rust meme
* Anger Java developers

![Rust Evangelism Strike Force meme](https://external-preview.redd.it/Ikj0dtD2q1f70pJtxZEJahFAJH0LkkcdtNuxMWT8Dl0.jpg?auto=webp&s=1c6212f4d10bc678f00d19b36b99d0eba6a8ca79)

Jokes aside, the official Java and C++ implementations of SlimeVR are here to
stay, and thats a good thing. Many people know Java and C++, but not so many
people know Rust. We should seek to port our work to Java and then upstream it
to benefit the wider SlimeVR community, and give back to the mainline project.

There are however a few reasons for an alternative, experimental Rust implementation:
* Some people are more proficient in Rust than they are in Java or C++
* Having another implementation lets us validate the interoperability between the
  firmware implementations and the server implementations
* Opportunity to experiment with different architectural design in the code
* Potential for performance improvements
* Experiment with frameworks more bare-metal than arduino/platformio. I've had a *lot* of issues with the esp32-c3 on platformio.


## License
All code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option. This means you can select the license you prefer! This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are very good reasons to include both.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
