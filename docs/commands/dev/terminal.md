---
name: Dev terminal
description: The dev terminal is a command prompt that let  you run only Lenra specific commands. It has his own history.
---

The dev terminal is a command prompt that let  you run only Lenra specific commands. It has his own history.

Here is the `help` command result of the dev terminal:

```bash
[lenra]$ help
lenra_cli 
The Lenra interactive command line interface

USAGE:
    lenra <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    check       Checks the running app
    continue    Continue the previous logs command since the last displayed logs
    exit        stop alias. Stop your app previously started with the start command
    expose      Exposes the app ports
    help        Print this message or the help of the given subcommand(s)
    logs        View output from the containers
    reload      Reload the app by rebuilding and restarting it
    stop        Stop your app previously started with the start command
```

## Subcommands

This tools contains many subcommands to help you do what you need.

- [continue](./continue.md): continues the previous logs command since the last displayed logs
- [reload](./reload.md): reloads the app by rebuilding and restarting it
- [stop](../stop.md): stops your app previously started with the start command
- [exit](../stop.md): `stop` alias
- [expose](./expose.md): exposes the app ports
- [logs](../logs.md): displays output from the containers
- [check](../check/index.md): checks the running app