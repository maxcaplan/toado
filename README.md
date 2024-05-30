# Toado 🐸 📋

A simple interactive task and project manager for the command line built in rust.

# Installation

Currently, toado can only be installed through rusts package manager cargo.
Install rust and cargo [here](https://www.rust-lang.org/tools/install).

To install toado through cargo, run the following command:
```bash
$ cargo install toado
```
To check that toado is installed, run the following command:
```bash
$ toado --version
```
This command will print the version of the application if installed correctly.

# Usage

Information about toado's commands can be viewed by running the `help` command as follows:
```bash
$ toado help
A simple interactive task and project manager for the command line

Usage: toado [OPTIONS] [SEARCH] [COMMAND]

Commands:
  search  Search for items
  add     Add a new item
  delete  Remove an item
  ls      Display a list of items
  check   Complete a task
  help    Print this message or the help of the given subcommand(s)

Arguments:
  [SEARCH]  Search term for item

Options:
  -t, --task         Execute search for tasks (default behaviour)
  -p, --project      Execute search for projects
  -v, --verbose      List all item information
  -f, --file <FILE>  Path to database file
  -h, --help         Print help
  -V, --version      Print version
```
You can view information about specific commands using the `help` as follows:
```
$ todo help ls
Display a list of items

Usage: toado ls [OPTIONS] [ORDER_BY]

Arguments:
  [ORDER_BY]  List item order [possible values: id, name, priority]

Options:
  -t, --task             List tasks (default behaviour)
  -p, --project          List projects
  -v, --verbose          List all item information
  -a, --asc              List in ascending order
  -d, --desc             List in descending order
  -l, --limit <LIMIT>    Limit the number of items listed
  -o, --offset <OFFSET>  Offset start of list
  -f, --full             List all items
  -h, --help             Print help
  -V, --version          Print version
```  

# Configuration

Toado's look and behaviour can be configured TOML config file as follows:

```TOML
# config.toml
[table]
horizontal = "-"
vertical = "|"
```

Config files can be passed as cli arguments as follows:

```bash
$ toado -c path/to/config.toml
```

If no config file is supplied, toado will look for a config file at `~/.config/toado/config.toml`

To generate a default config file, run the following commands:

```bash
$ rm ~/.config/toado/config.toml
$ toado
```

This will generate a config file at `~/.config/toado/config.toml`

# Building

To build the project, you must have [Rust](https://www.rust-lang.org/) version 1.77.2 or later.
Run the following commands to build toado:
```bash
$ git clone https://github.com/maxcaplan/toado.git
$ cd toado
$ cargo build --release
$ ./target/release/toado --version
```
