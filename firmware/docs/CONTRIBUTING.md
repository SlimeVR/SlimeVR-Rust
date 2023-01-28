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
* See [Pull Request Tips.md] to increase the speed that we review PRs.
* Read [Goals.md] if you are interested about the project goals.
* Respect other contributors and the community. Respect their time, also.
* When there is a technical disagreement, don't argue ad infinitum. The maintainers will
  at some point make the final decision. Likewise, the BDFL[^1] has the final say even
  over the maintainers. It is ok to bring the matter up again once new data is available
  or the circumstances have changed. And decisions by the maintainers may change at any
  time, whether there is new info or not.
* Have fun 🦀


[^1]: Benevolent Dictator for Life. @TheButlah has this title.

[Discord]: https://discord.com/channels/817184208525983775/1025861916805050409
[Rustup]: https://rustup.rs

