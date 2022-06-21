# Ferrous SimeVR Skeletal Model

The skeletal model for SlimeVR. Its worth reading the documentation [here][docs],
lots of time has been spent to make it easy to understand.

## Goals
* Should be independent of networking/server
* Easily embeddable in other applications
* Support bindings to Java, and typescript via [deno_bindgen].
* Extensive documentation, easily understood by rust beginners.

## Implementation Status
The basic graph structure is completed, but needs to be documented. The model solver is
not yet written.

[docs]: https://thebutlah.github.io/ferrous_slimevr/skeletal_model/
[deno_bindgen]: https://github.com/denoland/deno_bindgen
