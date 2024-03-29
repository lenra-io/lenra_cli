---
description: This subcommand updates the tools Docker images.
---

This subcommand updates the tools Docker images.

```bash
$ lenra update --help
lenra-update 
Update the tools Docker images

USAGE:
    lenra update [OPTIONS] [SERVICES]...

ARGS:
    <SERVICES>...    The service list to pull [default: devtool postgres mongo] [possible
                     values: app, devtool, postgres, mongo]

OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
        --expose <EXPOSE>    Exposes services ports [possible values: app, devtool, postgres, mongo]
    -h, --help               Print help information
    -v, --verbose            Run the commands as verbose
```
