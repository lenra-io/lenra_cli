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
- [reload](./reload.md): starts your app previously built with the build command
- [logs](./logs.md): displays output from the containers
- [stop](./stop.md): stops your app previously started with the start command
- [check](./check/index.md): checks the running app

Use the help options or help subcommand to understand how to use them:

```bash
$ lenra --help
lenra_cli 0.0.0
The Lenra command line interface

USAGE:
    lenra [OPTIONS] [SUBCOMMAND]

OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
        --expose <EXPOSE>    Exposes services ports [possible values: app, devtool, postgres, mongo]
    -h, --help               Print help information
    -v, --verbose            Run the commands as verbose
    -V, --version            Print version information

SUBCOMMANDS:
    build      Build your app in release mode
    check      Checks the running app
    dev        Start the app in an interactive mode
    help       Print this message or the help of the given subcommand(s)
    logs       View output from the containers
    new        Create a new Lenra app project from a template
    reload     Reload the app by rebuilding and restarting it
    start      Start your app previously built with the build command
    stop       Stop your app previously started with the start command
    update     Update the tools Docker images
    upgrade    Upgrade the app with the last template updates
```

Some global options are available for all subcommands:

```bash
OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
        --expose <EXPOSE>    Exposes services ports [possible values: app, devtool, postgres, mongo]
    -v, --verbose            Run the commands as verbose
```

They won't have effect on all subcommands but can be used for most of them.
Also, you will be able to set them in the [terminal context](./terminal/index.md) since they are defined for the whole terminal lifetime (except the `--expose` option that can be redefined by the [`expose` command](./terminal/expose.md)).
