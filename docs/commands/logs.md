This subcommand displays output from the containers.

```bash
$ lenra logs --help
lenra-logs 
View output from the containers

USAGE:
    lenra logs [OPTIONS] [SERVICES]...

ARGS:
    <SERVICES>...    The logged service list [default: app] [possible values: app, devtool,
                     postgres]

OPTIONS:
    -f, --follow           Follow log output
    -h, --help             Print help information
        --no-color         Produce monochrome output
        --no-log-prefix    Don't print prefix in logs
        --since <SINCE>    Show logs since timestamp (e.g. 2013-01-02T13:23:37Z) or relative (e.g.
                           42m for 42 minutes)
    -t, --timestamps       Show timestamps
        --tail <TAIL>      Number of lines to show from the end of the logs for each container
                           [default: all]
        --until <UNTIL>    Show logs before a timestamp (e.g. 2013-01-02T13:23:37Z) or relative
                           (e.g. 42m for 42 minutes)
```
