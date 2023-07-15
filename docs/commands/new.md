---
description: This subcommand creates a new Lenra app project from a template.
---

This subcommand creates a new Lenra app project from a template.
The target directory must not exist.

```bash
$ lenra new --help
lenra-new 
Create a new Lenra app project from a template

USAGE:
    lenra new [OPTIONS] [TOPICS]...

ARGS:
    <TOPICS>...    The project template topics from which your project will be created. For
                   example, defining `rust` look for the next API endpoint:
                   https://api.github.com/search/repositories?q=topic:lenra+topic:template+topic:rust&sort=stargazers
                   You can find all the templates at this url:
                   https://github.com/search?q=topic%3Alenra+topic%3Atemplate&sort=stargazers&type=repositories
                   You also can set the template project full url to use custom ones

OPTIONS:
    -h, --help           Print help information
    -p, --path <PATH>    The new project path [default: .]
```
