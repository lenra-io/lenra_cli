<div id="top"></div>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

# Lenra cli

The Lenra's command line interface.

[Report Bug](https://github.com/lenra-io/lenra_cli/issues)
·
[Request Feature](https://github.com/lenra-io/lenra_cli/issues)

<!-- GETTING STARTED -->
## Getting Started

### Prerequisites

To build and run the Lenra elements that handle your app, the Lenra CLI needs [Docker](https://docs.docker.com/engine/install/) and [Docker Compose](https://docs.docker.com/compose/install/).

You can also install the [Docker buildx command](https://docs.docker.com/build/buildx/install/) to use the [Buildkit optimization given by Dofigen](https://github.com/lenra-io/dofigen).

Install the Lenra CLI using one of the next possibilities.

#### Download the binary

You can download the binary from [the release page](https://github.com/lenra-io/lenra_cli/releases) and add it to your path environment variable.

#### Cargo install

First install Cargo, the Rust package manager: https://doc.rust-lang.org/cargo/getting-started/installation.html

Then use the next command to install the Lenra's cli:

```bash
cargo install lenra_cli
```

#### Build it from sources

First install Cargo, the Rust package manager: https://doc.rust-lang.org/cargo/getting-started/installation.html

Then clone this repository and install it with Cargo:

```bash
git clone https://github.com/lenra-io/lenra_cli.git
cargo install --path .
```

<p align="right">(<a href="#top">back to top</a>)</p>

### How to use it

Use the help options to understand how to use it:

```bash
$ lenra --help
lenra_cli 0.0.0
The Lenra command line interface

USAGE:
    lenra <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    build    Build your app in release mode
    dev      Start the app in an interactive mode
    help     Print this message or the help of the given subcommand(s)
    init     Generates dockerfile and docker compose file with the init command
    logs     View output from the containers
    new      Create a new Lenra app project
    start    Start your app previously built with the build command
    stop     Stop your app previously started with the start command
```

### Subcommands

This tools contains many subcommands to help you doing what you need.

- [new](#new): creates a new Lenra app project
- [build](#build): builds the Lenra app of the current directory
- [start](#start): starts your app previously built with the build command
- [logs](#logs): displays output from the containers
- [stop](#stop): stops your app previously started with the start command
- [dev](#dev): starts your app in dev mode
- [init](#init): Generates Docker and Docker Compose files

#### new

This subcommand creates a new Lenra app project from a given template and in a given directory.
The target directory must not exist.

```bash
$ lenra new --help
lenra-new 
Create a new Lenra app project

USAGE:
    lenra new <TEMPLATE> <PATH>

ARGS:
    <TEMPLATE>    The project template from which your project will be created. For example,
                  defining `rust` or `template-rust` will use the next one:
                  https://github.com/lenra-io/template-rust You can find all our templates at
                  this url:
                  https://github.com/orgs/lenra-io/repositories?q=&type=template&language=&sort=stargazers
                  You also can set the template project full url to use custom ones
    <PATH>        The project path

OPTIONS:
    -h, --help    Print help information
```

#### build

This subcommand builds the Lenra app of the current directory.
The app configuration are defined by a [configuration file](#configuration-file).

```bash
$ lenra build --help
lenra-build 
Build your app in release mode

USAGE:
    lenra build [OPTIONS]

OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
        --expose             Exposes all services ports
    -h, --help               Print help information
```

#### start

This subcommand starts the Lenra app of the current directory previously built.

```bash
$ lenra start --help
lenra-start 
Start your app previously built with the build command

USAGE:
    lenra start [OPTIONS]

OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
        --expose             Exposes all services ports
    -h, --help               Print help information
```

#### logs

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

#### stop

This subcommand stops the Lenra app of the current directory and removes the Docker Compose elements.

```bash
$ lenra stop --help
lenra-stop 
Stop your app previously started with the start command

USAGE:
    lenra stop [OPTIONS]

OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
    -h, --help               Print help information
```

#### dev

This subcommand starts the Lenra app of the current directory in dev mode.

```bash
$ lenra dev --help
lenra-dev 
Start the app in an interactive mode

USAGE:
    lenra dev [OPTIONS]

OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
        --expose             Exposes all services ports
    -h, --help               Print help information
```

#### init

This subcommand generates Docker and Docker Compose files.

```bash
$ lenra init --help
lenra-init 
Generates dockerfile and docker compose file with the init command

USAGE:
    lenra init [OPTIONS]

OPTIONS:
        --config <CONFIG>    The app configuration file [default: lenra.yml]
    -h, --help               Print help information
```

### Configuration file

The Lenra's configuration file describes your Lenra app configurations, like API versions or how to build it.

Here is an example using a Dofigen file:

```yaml
componentsApi: "1.0"
generator:
  dofigen: dofigen.yml
```

#### Configuration

The configuration is the main element of the file:

| Field            | Type             | Description                   |
|------------------|------------------|-------------------------------|
| `componentsApi`  | String           | The components API version    |
| `generator`      | [Generator](#generator)  The generator configuration |

#### Generator

The generator define your application is built. There are many configurators:

- [Dofigen](#dofigen)
- [Docker](#docker)

##### Dofigen

The Dofigen generator use a [Dofigen](https://github.com/lenra-io/dofigen) configuration to generate the Docker image.

The Dofigen configuration can be the path to a Dofigen file or it content directly.

##### Docker

The Docker generator use a Dockerfile to generate the Docker image.

The Dockerfile can be the path to a file or it content directly.

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please open an issue with the tag "enhancement" or "bug".
Don't forget to give the project a star! Thanks again!

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- LICENSE -->
## License

Distributed under the **MIT** License. See [LICENSE](./LICENSE) for more information.

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Lenra - [@lenra_dev](https://twitter.com/lenra_dev) - contact@lenra.io

Project Link: [https://github.com/lenra-io/lenra_cli](https://github.com/lenra-io/lenra_cli)

<p align="right">(<a href="#top">back to top</a>)</p>


<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/lenra-io/lenra_cli.svg?style=for-the-badge
[contributors-url]: https://github.com/lenra-io/lenra_cli/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/lenra-io/lenra_cli.svg?style=for-the-badge
[forks-url]: https://github.com/lenra-io/lenra_cli/network/members
[stars-shield]: https://img.shields.io/github/stars/lenra-io/lenra_cli.svg?style=for-the-badge
[stars-url]: https://github.com/lenra-io/lenra_cli/stargazers
[issues-shield]: https://img.shields.io/github/issues/lenra-io/lenra_cli.svg?style=for-the-badge
[issues-url]: https://github.com/lenra-io/lenra_cli/issues
[license-shield]: https://img.shields.io/github/license/lenra-io/lenra_cli.svg?style=for-the-badge
[license-url]: https://github.com/lenra-io/lenra_cli/blob/master/LICENSE.txt
