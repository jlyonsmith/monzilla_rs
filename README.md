# Monzilla

[![coverage](https://shields.io/endpoint?url=https://raw.githubusercontent.com/jlyonsmith/monzilla_rs/main/coverage.json)](https://github.com/jlyonsmith/monzilla_rs/blob/main/coverage.json)
[![Crates.io](https://img.shields.io/crates/v/monzilla_rs.svg)](https://crates.io/crates/monzilla_rs)
[![Docs.rs](https://docs.rs/monzilla_rs/badge.svg)](https://docs.rs/monzilla_rs)

An uncomplicated file monitoring tool. It runs a program then watches the files you specify for changes, restarting the program if the files change. You can also manually restart the program.  If you need more power I suggest [Watchman](https://facebook.github.io/watchman/).

## Installation

Install the command line tool with:

```sh
cargo install monzilla_rs
```

Then you can run it with something like the following:

```sh
monzilla -g 'scratch/*.fish' 'scratch/xyz.json5' -- scratch/forever.fish
```

Any time the files change the program will be killed and restarted.

## Points of Interest

This tool shows how to do a variety of advanced things in Rust:

- How to use [termios](https://crates.io/crates/termios) to capture CTRL+C and CTRL+R from the terminal
- Using [crossbeam](https://crates.io/crates/crossbeam) and [`select`](https://docs.rs/crossbeam/0.8.2/crossbeam/channel/struct.Select.html) to wait on multiple channels
- How to use the [notify](https://crates.io/crates/notify) to watch for file changes with with debouncing of multiple changes at once.
- Using [command-group](https://crates.io/crates/command-group) to group child processes so you can terminate the entire tree not just the immediate child process.

Plus, all the good stuff found in my [rust_cli_quickstart](https://github.com/jlyonsmith/rust_cli_quickstart).
