# Network crawler

## Running
The network crawler uses optional features and dependencies, which **must** be enabled in order for the binary to 
compile. These can be enabled by supplying `--features crawler` when running the command.

To see all arguments, run:
```bash
cargo r --bin crawler --features="crawler" -- --help
```

Argument `--seed-addrs` is the only required argument. It takes a list initial peers to start crawling from. For example:
```bash
cargo r --bin crawler --features="crawler" -- --seed-addrs 127.0.0.1:8081 127.0.0.1:8082
```