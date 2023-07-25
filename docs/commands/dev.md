---
description: This subcommand starts the Lenra app of the current directory in dev mode.
---

This subcommand starts the Lenra app of the current directory in dev mode.

The dev mode builds and starts the app and then displays its logs.

```bash
$ lenra dev --help
lenra-dev 
Start the app in an interactive mode

USAGE:
    lenra dev [OPTIONS]

OPTIONS:
        --attach             Attach the dev mode without rebuilding the app and restarting it
        --config <CONFIG>    The app configuration file [default: lenra.yml]
        --expose <EXPOSE>    Exposes services ports [possible values: app, devtool, postgres, mongo]
    -h, --help               Print help information
    -v, --verbose            Run the commands as verbose
```

When your app is in dev mode, you can run interactive commands through keyboard shortcuts.

Here is the help interactive command result displayed pressing `H` key:

```bash
SHORTCUTS: (Command  Key(s)  Description)
    Help      H                Print this message
    Reload    R                Reload the app by rebuilding and restarting it
    Quit      Q, Ctrl+C        Quit the interactive mode
    Stop      S                Stop your app previously started with the start command
```
