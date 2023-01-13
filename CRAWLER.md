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

Argument `--rpc-addr` takes socket address for the web server. Example:
```bash
cargo r --bin crawler --features="crawler" -- --seed-addrs 35.162.59.23:51235 --rpc-addr 127.0.0.1:8080
```
Crawler's metrics can be accessed by:
```bash
curl --data-binary '{"jsonrpc": "2.0", "id":0, "method": "getmetrics", "params": {} }' -H 'content-type: application/json'  http://127.0.0.1:8080/
```

Field "params" accepts a json. Param's field "file" can be added to dump data to a file:
```bash
curl --data-binary '{"jsonrpc": "2.0", "id":0, "method": "getmetrics", "params": {"file":"dump.json"}}' -H 'content-type: application/json'  http://127.0.0.1:8080/
```
