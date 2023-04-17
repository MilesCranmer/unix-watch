# unix-watch

[![Crates.io](https://img.shields.io/crates/v/unix-watch.svg)](https://crates.io/crates/unix-watch)
[![CI](https://github.com/MilesCranmer/unix-watch/workflows/CI/badge.svg)](https://github.com/MilesCranmer/unix-watch/actions)

## Installation

Install with:

```bash
cargo install unix-watch
```

You can then execute the binary with:

```bash
watch [-n <seconds>] -- <command>
```

Note that unlike the `watch` command on Linux, you must separate
CLI flags from the command with a `--`.
Note also that only the `-n` flag is supported.
ANSI color codes are forwarded automatically.

## License

Licensed under

 * Apache License, Version 2.0
   ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
