# Ziggurat x XRPL

The Ziggurat implementation for XRPLF's `rippled` nodes.

## Getting started

1. Clone this repository.
2. Build [rippled](https://github.com/XRPLF/rippled) from source.
3. Create the `~/.ziggurat` directory, copy the [validators configuration](https://github.com/XRPLF/rippled/blob/develop/cfg/validators-example.txt) there, and name it `validators.txt`. Also create a `config.toml` with the following contents:
```
# path = "<path to the directory where you built rippled>"
# start_command = "./rippled"
```
4. Run the start-stop test with `cargo +stable t`.

