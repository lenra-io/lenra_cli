---
description: This subcommand creates a new Lenra app project from a given template and in a given directory.
---

This subcommand creates a new Lenra app project from a given template and in a given directory.
The target directory must not exist.

```bash
$ lenra new --help
lenra-new 
Create a new Lenra app project

USAGE:
    lenra new <TEMPLATE> [PATH]

ARGS:
    <TEMPLATE>    The project template from which your project will be created. For example,
                  defining `rust` or `template-rust` will use the next one:
                  https://github.com/lenra-io/template-rust You can find all our templates at
                  this url:
                  https://github.com/orgs/lenra-io/repositories?q=&type=template&language=&sort=stargazers
                  You also can set the template project full url to use custom ones
    <PATH>        The project path [default: .]

OPTIONS:
    -h, --help    Print help information
```
