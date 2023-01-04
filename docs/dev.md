# dev

This subcommand starts the Lenra app of the current directory in dev mode.

The dev mode builds and starts the app and then displays its logs.

```bash
$ lenra dev --help
lenra-dev 
Start the app in an interactive mode

USAGE:
    lenra dev [OPTIONS]

OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
        --expose <EXPOSE>    Exposes services ports [possible values: app, devtool, postgres, mongo]
    -h, --help               Print help information
```

When the CLI receive a `Ctrl + C` signal while displaying logs in dev mode, it displays an [interactive command prompt](#interactive-commands).

## Interactive commands

Here is the `help` interactive command result:

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

### Subcommands

This tools contains many subcommands to help you do what you need.

- [continue](docs/dev-continue.md): continues the previous logs command since the last displayed logs
- [reload](docs/dev-reload.md): reloads the app by rebuilding and restarting it
- [stop](docs/stop.md): stops your app previously started with the start command
- [exit](docs/stop.md): `stop` alias
- [expose](docs/dev-expose.md): exposes the app ports
- [logs](docs/logs.md): displays output from the containers
- [check](docs/check.md): checks the running app
