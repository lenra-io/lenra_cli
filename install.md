## Prerequisites

To build and run the Lenra elements that handle your app, the Lenra CLI needs [Docker](https://docs.docker.com/engine/install/) and [Docker Compose](https://docs.docker.com/compose/install/).

To create a new project and upgrade it later, the CLI also needs [git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) (that we hope you already use ^^).

You can also install the [Docker buildx command](https://docs.docker.com/build/buildx/install/) to use the [Buildkit optimization given by Dofigen](https://github.com/lenra-io/dofigen).

Install the Lenra CLI using one of the next possibilities.

## Install

### Download the binary

You can download the binary from [the release page](https://github.com/lenra-io/lenra_cli/releases) and add it to your path environment variable.

On linux you can do this quickly by running the following :
```
curl -fSLo /usr/local/bin/lenra https://github.com/lenra-io/lenra_cli/releases/latest/download/lenra-linux-x86_64
chmod +x /usr/local/bin/lenra
```

PS: You can replace x86-64 by aarch64 if you run ARMv8 device.

### Cargo install

First install Cargo, the Rust package manager: https://doc.rust-lang.org/cargo/getting-started/installation.html

Then use the next command to install the Lenra's cli:

```bash
cargo install lenra_cli
```

Since the CLI is not released yet, you have to target a [pre-release version](https://github.com/lenra-io/lenra_cli/releases) like that:

```bash
cargo install lenra_cli@~1.0.0-beta.0
```

### Build it from sources

First install Cargo, the Rust package manager: https://doc.rust-lang.org/cargo/getting-started/installation.html

Then clone this repository and install it with Cargo:

```bash
git clone https://github.com/lenra-io/lenra_cli.git
cargo install --path .
```

## And now ?

Learn how to use it with [our docs website](https://docs.lenra.io).
