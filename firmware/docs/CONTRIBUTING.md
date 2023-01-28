# Contributors Guide
This guide is for people interested in contributing to or tinkering with the firmware.

First of all, thank you for your interest! If you need help, feel free to message me on
the [SlimeVR Discord][Discord] or file an issue on github.


## 🔨 The Development tools
We use several tools and crates (libraries) that provide a good development experience
(even better than arduino!). First, install rust, via [rustup]. Then you can follow
these links to read more if you wish. But all of the basic rust commands will still
work, like `cargo build`, `cargo doc --open`, etc.

* [Configuring](Configuring.md)
* [Building](Building.md)
* [Debugging](Debugging.md)
* [Overview of the embedded rust ecosystem](Ecosystem.md)


## 📝 Contribution guidelines
* Please run `cargo fmt` to autoformat your code. Pro tip: try setting it up to format
  on save in your IDE :)
* Don't use any dependencies that are GPL or viral or closed-source licenses (ask me if
  you are unsure)
* If you want, see [tips for making a simple PR](Pull Request Tips.md) to increase the
  speed that we review PRs.
* Read the [project goals](Goals.md) if you are interested.
* Have fun 🦀


[Discord]: https://discord.com/channels/817184208525983775/1025861916805050409
[Rustup]: https://rustup.rs

