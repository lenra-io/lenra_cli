---
description: This subcommand stops the Lenra app of the current directory and removes the Docker Compose elements.
---

This subcommand stops the Lenra app of the current directory and removes the Docker Compose elements.

```bash
$ lenra stop --help
lenra-stop 
Stop your app previously started with the start command

USAGE:
    lenra stop [OPTIONS]

OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
        --expose <EXPOSE>    Exposes services ports [possible values: app, devtool, postgres, mongo]
    -h, --help               Print help information
    -v, --verbose            Run the commands as verbose
```
