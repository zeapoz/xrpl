# Ziggurat x Ripple

The Ziggurat implementation for Ripple nodes.

## Getting started

1. Clone this repository.
2. Build [rippled](https://github.com/ripple/rippled) from source.
3. Create a the `~/.ziggurat` directory and copy the [validators configuration](https://github.com/ripple/rippled/blob/develop/cfg/validators-example.txt) there and name it `validators.txt`. Also create a `config.toml` with the following contents:
```
# path = "<path to the directory where you built rippled>"
# start_command = "./rippled"
```
4. Run the start-stop test with `cargo +stable t`.

