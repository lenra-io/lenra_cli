---
description: This CLI contains many subcommands to help you doing what you need.
---

This CLI contains many subcommands to help you doing what you need.

- [new](./new.md): creates a new Lenra app project from a template
- [dev](./dev/index.md): starts your app in dev mode
- [update](./update.md): updates the tools Docker images
- [upgrade](./upgrade.md): upgrades the app with the last template updates
- [build](./build.md): builds the Lenra app of the current directory
- [start](./start.md): starts your app previously built with the build command
- [logs](./logs.md): displays output from the containers
- [stop](./stop.md): stops your app previously started with the start command
- [check](./check/index.md): checks the running app

Use the help options or help subcommand to understand how to use them:

```bash
$ lenra --help
lenra_cli 0.0.0
The Lenra command line interface

USAGE:
    lenra <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    build      Build your app in release mode
    check      Checks the running app
    dev        Start the app in an interactive mode
    help       Print this message or the help of the given subcommand(s)
    logs       View output from the containers
    new        Create a new Lenra app project from a template
    start      Start your app previously built with the build command
    stop       Stop your app previously started with the start command
    update     Update the tools Docker images
    upgrade    Upgrade the app with the last template updates
```
