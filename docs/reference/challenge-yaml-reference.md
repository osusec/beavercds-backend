---
title: <code>challenge.yaml</code> Reference
---

# `challenge.yaml` Config Reference

Challenge configuration is expected to be at `<category>/<name>/challenge.yaml`.

There are some examples available on the [challenge quickstart guide](/guides/challenge-quickstart#examples).

Available fields:

[[toc]]

`*` denotes required fields.

## `name`*

- type: `string`
- no default

The name of the challenge, as shown to players in the frontend UI.

```yaml
name: notsh

# can have spaces:
name: Revenge of the FIPS
```

## `author`*

- type: `string`
- no default

Author or authors of the challenge, as shown to players in the frontend UI. If there are multiple authors, specify them as one string.

```yaml
author: John Author

# multiple authors:
author: Alice, Bob, and others
```

## `description`*

- type: `string`
- no default

Description and flavortext for the challenge, as shown to players in the frontend UI. Supports templating to include information about the challenge, such as the link or command to connect.

Most challenges only need `{{ nc }}` or `{{ link }}`.

| Available fields | Description                                                                                                              |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `domain`         | Full domain the challenge is exposed at, e.g. `<subdomain>.chals.example.ctf`                                            |
| `port`           | Port the challenge is listening on                                                                                       |
| `nc`             | `nc` command to connect to TCP challenges, with Markdown backticks <br> (equivalent to `` `nc {{domain}} {{port}}` ``)   |
| `url`            | URL to the exposed web domain for web challenges, plus port if needed <br> (equivalent to `https://{{domain}}:{{port}}`) |
| `link`           | Markdown link to `url`                                                                                                   |
| `challenge`      | The full challenge.yaml config object for this challenge, with subfields                                                 |

```yaml
description: |
  Some example challenge. Blah blah blah flavor text.

  In case you missed it, this was written by {{ challenge.author }}
  and is called {{ challenge.name }}.

  {{ nc }}      # `nc somechal.chals.example.ctf 12345`
  {{ link }}    # [https://somechal.chals.example.ctf](https://somechal.chals.example.ctf)
```

## `category`

- type: `string`
- default: from folder structure

The category for the challenge, parsed from the directory structure.

::: warning
This is automatically set from the expected directory structure of `<category>/<name>/challenge.yaml` and should not be set in the file.
:::

## `difficulty`

- type: `integer`
- no default

::: info
Not implemented yet, does nothing
:::

The difficulty from the challenge, used to set point values. Values correspond to entries in the [rcds.yaml difficulty settings](rcds-yaml-reference#difficulty).

```yaml
difficulty: 1 # the current default
```

## `flag`*

- type: `string` | `dict`
- no default

Where to find the flag for the challenge. The flag can be in a file, a regex, or a direct string.

```yaml
# directly set
flag: ctf{example-flag}

# from a file in in the challenge directory
flag:
  file: ./flag

# regex
flag:
  regex: /ctf\{(foo|bar|ba[xyz])\}/
```

::: info
Regex flags are not implemented yet and setting one does nothing
:::

## `provide`

- type: list of `string`/`dict`
- default: `[]` (no files)

List of files to provide to the players on the frontend UI. These files can be from the challenge directory or from a container image built for a [challenge pod](#pods), and uploaded individually or zipped together.

If there are no files to upload for this challenge, this can be omitted or set to an empty array.

```yaml
provide:
  # files from the challenge folder in the repo
  - somefile.txt
  - otherfile.txt

  # these are all equivalent
  - foo.txt
  - include: foo.txt
  - include: [ foo.txt ]

  # rename a really long name as something shorter for upload
  - as: short.h
    include: some_really_long_name.h

  # multiple files from src/ in the challenge folder, zipped as together.zip
  - as: together.zip
    include:
      - src/file1
      - src/file2
      - src/file3

  # multiple files pulled from the container image for the `main` pod,
  # uploaded individually as `notsh` and `libc.so.6`
  - from: main
    include:
      - /chal/notsh
      - /lib/x86_64-linux-gnu/libc.so.6

  # single file pulled from the main container and renamed
  - from: main
    as: libc.so
    include: /lib/x86_64-linux-gnu/libc.so.6

  # multiple files pulled from the main container and zipped together
  - from: main
    as: notsh.zip
    include:
      - /chal/notsh
      - /lib/x86_64-linux-gnu/libc.so.6


# if no files need to be provided:
provide: []
# or omit entirely
```

### `.include`

- type: list of `string`
- no default

File or list of files to upload individually, or include in a zip if `as` is set.

When uploading, only the basename is used and the path to the file is discarded.

If a provide item is specified as a single string, it is interpreted as an `include:`.

### `.as`

- type: `string`
- no default

If `.include` is a single file, rename to this name while uploading.

If multiple files, zip them together into the given zip file.

### `.from`

- type: `string`
- no default

Fetch these files from the corresponding [challenge pod](#pods) image.

## `pods`

- type: list of `dict`
- default: `[]` (no pods)

Defines how to build and deploy any services needed for the challenge.

Challenge pods can be built from a local Dockerfile in the challenge folder or use an upstream image directly.

If there are no pods or images needed for this challenge, this can be omitted or set to an empty array.

```yaml
pods:
  - name: main
    build: .
    ports:
      - internal: 1337        # expose a container listening on port 1337 ...
        expose:
          http: examplechal   # as a web chal at https://examplechal.<chals_domain>

  - name: db
    image: postgres:alpine
    architecture: arm64
    env:
      POSTGRES_USER: someuser
      POSTGRES_PASSWORD: notsecure

# if no containers or pods need to be deployed:
pods: []
# or omit entirely
```

### `.name`

- type: `string`
- no default

Name of the pod, used to refer to this container as [a source for `provide` files](#provide) and for generated resource names.

Cannot contain spaces or punctuation, only alphanumeric and `-`.

### `.build`

- type: `string` | `dict`
- no default

Build the container image for this pod from a local `Dockerfile`. Supports a subset of the [docker-compose build spec](https://docs.docker.com/reference/compose-file/build/#illustrative-example),
either:
  - a string path to the build context folder
  - yaml with explicit `context` path, `dockerfile` path within context folder,
    and `args` build args (only `context`, `dockerfile`, and `args` are
    supported)

The build context directory is relative to the `challenge.yaml`, and the `dockerfile` is relative to the context directory.

Conflicts with [`image`](#image).

```yaml
    # build a container from a Dockerfile in the challenge folder
    build: .

    # equivalent to the above but with explicit build context and Dockerfile name
    build:
      context: .
      dockerfile: Dockerfile

    # build from a subfolder with a custom Dockerfile and some build args
    build:
      context: src/
      dockerfile: Containerfile.remote
      args:
        CC_OPTS: "-Osize"
```

### `.image`

- type: `string`
- no default

Use an available container image for the pod instead of building one from source.

Conflicts with [`build`](#build).

### `.env`

- type: `dict`
- default: `{}` (no envvars)

Any environment variables to set for the running pod. Specify as `name: value`.

```yaml
env:
  SOME_ENVVAR: foo bar
```

### `.architecture`

- type: `string`
- default: `"amd64"`

Set the desired CPU architecture to run this pod on. Kubernetes uses GOARCH architecture names.

```yaml
architecture: amd64
architecture: arm64
```

### `.resources`

- type: `dict`
- default: global default from `rcds.yaml`

The CPU and memory resources that will be reserved for this pod. Kubernetes will make sure the requested amounts will be available for this pod to use, and will also restart the pod if it goes over these limits.

Uses the [Kubernetes resource units](https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/#resource-units-in-kubernetes).

If not set, the default set in [`rcds.yaml`](rcds-yaml-reference#resources) is used.

```yaml
resources:
  cpu: 1
  memory: 512Mi
```

### `.replicas`

- type: `number`
- default: `2`

How many instances of the pod to run. Traffic is load-balanced between instances.

Default is 2 and this is probably fine unless the challenge is very resource intensive.

```yaml
replicas: 2 # the default
```

### `.ports`

- type: list of `dict`
- default: `[]`

List of ports to expose to players.

#### `.ports[].internal`

- type: `number`
- no default

The port that the challenge container (i.e. `xinetd`/`nginx`/etc inside) is listening on.

#### `.ports[].expose`

- type: `dict`
- no default

How to expose the internal container port to players -- either as a TCP port or a subdirectory for web challenges. Must be one of the following:

**`.ports[].expose.tcp`**

- type: `number`
- no default

The port to expose the challenge over raw TCP at on the challenge subdomain. Must be unique across all other exposed TCP challenges.

```yaml [For TCP challenges] {8-10}
pods:
  - #...
    ports:
      - internal: 31337   # the port the container listens on
        expose:
          tcp: 30124      # exposed at <challenges-domain>:30124
```

**`.ports[].expose.http`**

- type: `string`
- no default

The subdomain to expose the challenge at as a website (port 80/443). This is prepended to the global challenge subdomain. The cluster will provision an SSL certificate for the site.

Must be a valid DNS domain name (alphanumeric, `_`, `-`).

```yaml [For web challenges] {8-10}
pods:
  - name: main
    build:
      context: .
      dockerfile: Dockerfile
    replicas: 2
    ports:
      - internal: 31337   # the port the container listens on
        expose:
          http: my-chal   # exposed at https://my-chal.<challenges-domain>
```

### `.volume`

- type: `string`
- no default

::: info
Not implemented yet, does nothing
:::
