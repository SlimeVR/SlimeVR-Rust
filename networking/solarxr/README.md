# 🦀 SolarXR Client
A rust client for the [SolarXR protocol](https://github.com/SlimeVR/SolarXR-Protocol).

## 🔭 Goals
* Make it easier to write rust applications that act as clients for SolarXR.
* Provide a reference implementation of a SolarXR client.
* Improve the code quality of the [SlimeVR overlay](/overlay) by keeping networking
  separate.

## 🚧Implementation Status
This client is being used in the beta of the SlimeVR overlay, but its API is not yet
stable. We will make no stability guarantees just yet.

We just recently pulled this into its own crate, so the API is being reworked to stop
hard-coding in overlay-specific networking logic. That is why some of the protocols
aren't checkmarked as completed, even though they are used in the crate already. Once I
finish expressing a clean and general purpose API, I will mark them as done.

* [X] Initial functionality necessary to support SlimeVR overlay.
* [X] Validate that the initial implementation works in the overlay.
* [ ] Support for controlling the DataFeed protocol.
* [ ] Support for controlling the PubSub protocol.
* [ ] Support for the functionality in the RPC protocol.
