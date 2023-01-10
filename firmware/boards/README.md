# Board definitions
This folder contains several `.toml` files that each describe information about a particular
board, such as its pins.

The build script in `build.rs` will read the `BOARD` environment variable to get the
path to the `board.toml` file to use for the firmware.

Note that if an absolute path is not given, it will check this directory (and not the 
current working directory!) for the board.
