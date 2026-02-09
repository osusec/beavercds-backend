# Challenge Quickstart

This will walk through the steps needed to create a new challenge and configure
its `challenge.yaml` config file. This is what sets all of the information about
the challenge and how it is presented to players.

For the impatient, there are a few [complete example config files](#examples) at
the bottom of the page.

::: info
We have an `new-chal` command planned that will prompt for most of this
information, but for now the challenge config needs to be created manually.
:::


## 1. Create challenge directory

Challenges are expected to follow the directory structure of
`/<category>/<name>/challenge.yaml`. Create the directory and challenge config
file following that convention:

```sh
$ mkdir -p $CATEGORY/$CHAL_NAME
$ touch $CATEGORY/$CHAL_NAME/challenge.yaml
```


## 2. Add metadata

Add the name, author, and description fields for your challenge to the `challenge.yaml` file. These will be shown to players on the scoreboard.

```yaml [challenge.yaml]
name: My First Pyjail 🙂
author: John Author
description: |
  how will you get out of this one?

  {{ nc }}
```

The challenge name and author will be shown to players as written.

The description supports Markdown formatting and Jinja-style templating for challenge info. Most of the time, you will need either `{{ nc }}` for netcat challenges or `{{ link }}` for web challenges. The full list of template options are available in [the reference](/reference/challenge-yaml-reference#description).


## 3. Set the flag

If the flag for your challenge is stored in a file, read it in with:

```yaml [challenge.yaml]
flag:
  file: ./flag
```

For static file challenges or others that don't need it to be on disk

```yaml [challenge.yaml]
flag: example{s0m3-fl4g}
```


## 4. Create container and pod

::: tip
If your challenge **does not** need a container, skip this section.
:::

If your challenge has a service that is exposed to players, it needs to run as a
container. BeaverCDS will take care of building and deploying the container, but
you need to create the `Dockerfile` to build your challenge source into one.

The pod `name` should be kept to one word, and is used to reference this
container for [providing files to users](#_5-provide-files-to-users) in the next section.

### Build the container

Once you have the container source and Dockerfile ready, add them to the pod
`build` section.
This is a good starting point for most challenges, where the Dockerfile and container source are in the same folder as this `challenge.yaml`:

```yaml [challenge.yaml] {4-5}
pods:
  - name: main
    build:
      context: .
      dockerfile: Dockerfile
    replicas: 2
    ports:
      # todo
```

### Define how the challenge is exposed

To expose the container to players, fill in the `ports` section with the
port the container is listening on, and either a port number or http subdomain.

- For TCP challenges, set `expose.tcp` to a port. This must be a unique port
  from other TCP challenges. The challenge will be exposed at
  `<challenges_domain>:<port>`.
- For web challenges, set `expose.http` to a subdomain. The challenge will be
  exposed at `<subdomain>.<challenges_domain>` with an HTTPS cert.

::: code-group
```yaml [For TCP challenges] {8-11}
pods:
  - name: main
    build:
      context: .
      dockerfile: Dockerfile
    replicas: 2
    ports:
      - internal: 31337   # <-- what your container listens on
        expose:
          tcp: 30124      # <-- would expose at chals.example.ctf:30124
```

```yaml [For web challenges] {8-11}
pods:
  - name: main
    build:
      context: .
      dockerfile: Dockerfile
    replicas: 2
    ports:
      - internal: 31337   # <-- what your container listens on
        expose:
          http: my-chal   # <-- would expose at https://my-chal.chals.example.ctf
```
:::



### 5. Provide files to users

::: tip
If your challenge **does not** have any file handouts, skip this section.
:::

If your challenge needs to provide files to users, add a `provide` block that
lists what files and where to find them.

::: warning
Currently, all `provide` entries need to be files. Directories or globs are not
yet supported.
:::

```yaml [challenge.yaml]
provide:
  # file from the challenge folder in the repo
  - somefile.txt

  # multiple files from src/ in the challenge folder, zipped as together.zip
  - as: together.zip
    include:
      - src/foo
      - src/bar
      - src/baz

  # multiple files pulled from the container image for the `main` pod
  # (see previous Pods section)
  - from: main
    include:
      - /chal/notsh
      - /lib/x86_64-linux-gnu/libc.so.6

  # same as above, but now zipped together
  - from: main
    as: notsh.zip
    include:
      - /chal/notsh
      - /lib/x86_64-linux-gnu/libc.so.6
```


## Examples

::: code-group

```yaml [Full TCP challenge]
name: notsh
author: John Author
description: |-
  This challenge isn't a shell

  {{ nc }}

flag:
  file: ./flag

provide:
  - from: main
    include:
      - /chal/notsh
      - /lib/x86_64-linux-gnu/libc.so.6

pods:
  - name: main
    build: .
    replicas: 2
    ports:
      - internal: 31337
        expose:
          tcp: 30124
```

```yaml [Full HTTP challenge]
name: bar
author: somebody
description: |
  can you order a drink from the webserver?

  {{ url }}

flag:
  file: ./flag

# no file handouts, so no provide: section

pods:
  - name: bar
    build:
      context: .
      dockerfile: Containerfile
    replicas: 1
    ports:
      - internal: 80
        expose:
          http: bar # subdomain only
```

```yaml [Full file-only challenge]
name: GhostINT
author: The Guessers Geo
description: |
  where was this picture taken?

  flag format: `example{LAT__LONG}`, rounded to five places

flag: "example{51.51771__-0.20495}"

provide:
  - the-view-from-your-balcony.png
```
:::
