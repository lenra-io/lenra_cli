<div id="top"></div>

<div align="center">
  <!-- Keep one empty line -->

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]
</div>

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <!-- <a href="https://github.com/lenra-io/template-hello-world-node12">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a> -->

<h1 align="center">Lenra CLI</h1>

  <p align="center">
    The Lenra's command line interface.
  </p>
</div>

[Report Bug](https://github.com/lenra-io/lenra_cli/issues)
·
[Request Feature](https://github.com/lenra-io/lenra_cli/issues)


## What is Lenra

Lenra is an open source framework to create your app using any language, and deploy it without any Ops scale, built on ethical values.

[Discover our framework](https://www.lenra.io/discover.html)

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

Since the CLI is not released yet, you have to target a [pre-release version](https://github.com/lenra-io/lenra_cli/releases) like that:

```bash
cargo install lenra_cli@v1.0.0-beta.24
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
    build     Build your app in release mode
    check     Checks the running app
    dev       Start the app in an interactive mode
    help      Print this message or the help of the given subcommand(s)
    init      Generates dockerfile and docker compose file with the init command
    logs      View output from the containers
    new       Create a new Lenra app project
    start     Start your app previously built with the build command
    stop      Stop your app previously started with the start command
    update    Update the tools Docker images
```

<p align="right">(<a href="#top">back to top</a>)</p>

## Subcommands

This tool contains many subcommands to help you doing what you need.

- [new](docs/new.md): creates a new Lenra app project
- [dev](docs/dev.md): starts your app in dev mode
- [update](docs/update.md): updates the tools Docker images
- [build](docs/build.md): builds the Lenra app of the current directory
- [start](docs/start.md): starts your app previously built with the build command
- [logs](docs/logs.md): displays output from the containers
- [stop](docs/stop.md): stops your app previously started with the start command
- [check](docs/check.md): checks the running app
- [init](docs/init.md): generates Docker and Docker Compose files


<p align="right">(<a href="#top">back to top</a>)</p>

## Configuration file

The Lenra's configuration file describes your Lenra app configurations, like API versions or how to build it.

Here is an example using a Dofigen file:

```yaml
componentsApi: "1.0"
generator:
  dofigen: dofigen.yml
```

### Configuration

The configuration is the main element of the file:

| Field           | Type                                                 | Description                |
| --------------- | ---------------------------------------------------- | -------------------------- |
| `componentsApi` | String                                               | The components API version |
| `generator`     | [Generator](#generator)  The generator configuration |

### Generator

The generator define your application is built. There are many configurators:

- [Dofigen](#dofigen)
- [Docker](#docker)

#### Dofigen

The Dofigen generator use a [Dofigen](https://github.com/lenra-io/dofigen) configuration to generate the Docker image.

The Dofigen configuration can be the path to a Dofigen file or it content directly.

#### Docker

The Docker generator use a Dockerfile to generate the Docker image.

The Dockerfile can be the path to a file or it content directly.

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please open an issue with the tag "enhancement" or "bug".
Don't forget to give the project a star! Thanks again!

### Run tests

In order to have more advanced unit tests, we use [Mocktopus](https://github.com/CodeSandwich/Mocktopus) that is based on the nightly Rust toolchain.
To run them you have to install the toolchain and run them with it:

```bash
rustup install nightly
cargo +nightly test
```

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
