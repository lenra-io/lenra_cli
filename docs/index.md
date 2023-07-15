---
name: CLI
description: The Lenra's command line interface helps you building your Lenra app locally and it's gona be your best friend !
---

The Lenra's command line interface helps you building your Lenra app locally and it's gona be your best friend !

We've created many [commands](commands/index.md) to help you managing a local smaller Lenra instance to create and test your app.

## Create a new app

To create an app, you should use the [`lenra new` subcommand](commands/new.md) that will create a new project based on [a Lenra app template](https://github.com/orgs/lenra-io/repositories?q=&type=template&language=&sort=stargazers).

Start it running the [`lenra dev` subcommand](commands/dev/index.md).
Your app will be built, started and exposed on http://localhost:4000/

Here are the steps to start building a JavaScript Lenra app:

```bash
# new app from javascript template in a new 'my-app' directory
lenra new javascript -p my-app
# move to the new app dir
cd my-app
# initialize git versionning
git init
# start your app
lenra dev
```

Look the [`lenra dev` subcommand](commands/dev/index.md) to understand the new dev mode and terminal.

## Configure your app container

Lenra app system is based containers.
Look at our [`lenra.yml` config file](config-file.md) to adapt your app configuration and better understand how you can tools in your app containers.