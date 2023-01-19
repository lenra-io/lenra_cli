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
    -t, --terminal           Open the dev terminal instead of starting the interactive mode
```

While displaying logs in dev mode, you can run some [interactive commands](dev/interactive.md) by pressing the good keys.
You also can press `Ctrl + C` to start our [dev terminal](dev/terminal.md).


