# build

This subcommand builds the Lenra app of the current directory.
The app configuration is defined by a [configuration file](#configuration-file).

```bash
$ lenra build --help
lenra-build 
Build your app in release mode

USAGE:
    lenra build [OPTIONS]

OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
        --expose <EXPOSE>    Exposes services ports [default: app postgres mongo] [possible values:
                             app, devtool, postgres, mongo]
    -h, --help               Print help information
```
