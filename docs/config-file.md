---
name: lenra.yml config file
description: The Lenra's configuration file describes your Lenra app configurations, like API versions or how to build it.
---

The Lenra's configuration file describes your Lenra app configurations, like API versions or how to build it.

Here is an example using a Dofigen file:

```yaml
path: "."
generator:
  dofigen: dofigen.yml
```

## Configuration

The configuration is the main element of the file:

| Field       | Type                    | Description                    |
| ----------- | ----------------------- | ------------------------------ |
| `path`      | String                  | The project path (default ".") |
| `generator` | [Generator](#generator) | The generator configuration    |

## Generator

The generator define your application is built. There are many configurators:

- [Configuration](#configuration)
- [Generator](#generator)
  - [Dofigen](#dofigen)
  - [Docker](#docker)

### Dofigen

The Dofigen generator use a [Dofigen](https://github.com/lenra-io/dofigen) configuration to generate the Docker image.

The Dofigen configuration can be the path to a Dofigen file or it content directly.

### Docker

The Docker generator use a Dockerfile to generate the Docker image.

The Dockerfile can be the path to a file or it content directly.