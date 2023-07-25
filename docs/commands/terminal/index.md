---
description: The terminal is a command prompt that let you run only Lenra specific commands. It has his own history and keep the same command context during it lifetime.
---

The terminal is a command prompt that let you run only Lenra specific commands. It has his own history and keep the same command context during it lifetime.

To start the terminal, run the `lenra` command without any subcommand:

```bash
$ lenra
```

You can use the global options to configure the terminal context:

```bash
OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
        --expose <EXPOSE>    Exposes services ports [possible values: app, devtool, postgres, mongo]
    -v, --verbose            Run the commands as verbose
```

## Commands

The terminal let run all the `lenra` subcommands (excpet the `terminal` itself and the `new` one) without the need of write `lenra` before them each time and even more:


- [dev](../dev.md): starts your app in dev mode
- [update](../update.md): updates the tools Docker images
- [upgrade](../upgrade.md): upgrades the app with the last template updates
- [build](../build.md): builds the Lenra app of the current directory
- [start](../start.md): starts your app previously built with the build command
- [reload](../reload.md): starts your app previously built with the build command
- [logs](../logs.md): displays output from the containers
- [stop](../stop.md): stops your app previously started with the start command
- [check](../check/index.md): checks the running app
- [expose](./expose.md): exposes the services ports and keep it in the terminal context
- [exit](./exit.md): exits the terminal

Here is the help result in the terminal:

```bash
[lenra]$ help
lenra_cli 
The Lenra interactive command line interface

USAGE:
    lenra <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    build      Build your app in release mode
    check      Checks the running app
    dev        Start the app in an interactive mode
    exit       Exits the terminal
    expose     Exposes the app ports
    help       Print this message or the help of the given subcommand(s)
    logs       View output from the containers
    reload     Reload the app by rebuilding and restarting it
    start      Start your app previously built with the build command
    stop       Stop your app previously started with the start command
    update     Update the tools Docker images
    upgrade    Upgrade the app with the last template updates
```
