---
title: <code>rcds.yaml</code> Reference
---

# `rcds.yaml` Config Reference

`rcds.yaml` is the 'global' config for beaverCDS. This defines what challenges
are available, where to deploy them to, and what credentials to use for building
and deploying them.

This will always be at `/rcds.yaml` in the challenges repository.

Available fields:

[[toc]]

## `flag_regex`

Regex for the flag format. This is used to validate challenges' flags, and in
the future will be sent to the scoreboard to help validate submissions.

```yaml
flag_regex: 'example{.+}'
```

## `registry`

Challenge container registry config. This is where the container images will be
stored. This registry should be kept private in order to prevent leaks of
challenge secrets or hidden challenges.

```yaml
registry:
  domain: registry.io/myctf
  tag_format: "{{domain}}/{{challenge}}-{{container}}:{{profile}}"
  build: 
    user: pushuser
    pass: fakepassword
  cluster: 
    user: pulluser
    pass: alsofake
  
```

### `domain`

This is the shared portion of the container image spec for the registry that
will be used in the `tag_format` template. This should include the hostname and
any persistent components.

Examples: `docker.io/yourorg`, `ghcr.io/examplesec`

### `tag_format`

Specifies the container image and tag that challenge containers will be built
as. This is used as a template with the challenge information to produce the
final container image and tag for each challenge. 

- Default, works for almost all registries (Docker, GHCR, DigitalOcean,
  self-hosted, ...):
  
  `"{{domain}}/{{challenge}}-{{container}}:{{profile}}"`

- For registries like AWS ECR that require all image registries to be precreated
  ahead-of-time, this keeps all the challenge info in the tag so only one ECR
  registry needs to be created:
  
  `"{{domain}}:{{challenge}}-{{container}}-{{profile}}"`

Format: Jinja-style double-braces around field name (`{{ field_name }}`)

The required and only fields are:

- `domain`: the domain config field above; the repository base URL
- `challenge`: challenge name, slugified
- `container`: name of the specific pod in the challenge this image is for
- `profile`: the current deployment profile, for isolating images between environments

If setting a custom format, you must use all four of these fields in order for challenge images to not overwrite each other.

Example:

For challenge `pwn/notsh`, chal pod container `main`, and profile `prod`:

```yaml
registry:
  domain: registry.io/myctf
  # default tag_format
# --> registry.io/myctf/pwn-notsh-main:prod
```

```yaml
registry:
  domain: registry.gitlab.com/ourteam/challenges-2025
  tag_format: "{{domain}}/{{challenge}}/{{container}}:{{profile}}"
# --> registry.gitlab.com/ourteam/challenges-2025/pwn-notsh/main:prod
```

### `build`

Registry credentials that will be used locally to push up challenge container
images. This must have push permissions. 

Format: `{ user: "registry-username", pass: "registry-password" }`

```yaml
registry:
  build:
    user: fakeuser
    pass: notrealpass
```

### `cluster`

Registry credentials that will be used in the Kubernetes cluster to pull the
challenge container images. This must have pull permissions, but does not need
push. 

Format: `{ user: "registry-username", pass: "registry-password" }`

```yaml
registry:
  cluster:
    user: alsofake
    pass: stillnotreal
```

## `points`

Defines the available difficulty classes for challenges. This allows challenges
to be worth different points, e.g. for harder challenges or a survey with
minimal points.

```yaml
points:
  - difficulty: "normal"
    max: 500
    min: 100
  - difficulty: "hard"
    max: 600
    min: 200
  - difficulty: "survey"
    max: 1
    min: 1
```

### `difficulty`

::: info
Not implemented yet, does nothing. Requires upcoming scoreboard integration.
:::

Name of this difficulty class. Challenges will use this name to set their
difficulty class via the [`difficulty` field in their
`challenge.yaml`](./challenge-yaml-reference.md#difficulty).

### max, min

Maximum and minimum points that challenges with this difficulty will be scored
as. Points are done via dynamic scoring; challenges start at max points and as
more people solve a challenge it approaches the minimum.

## `defaults`

These set the default difficulty class and cluster resource requests for
challenges that do not have them set in their `challenge.yaml`.

```yaml
defaults:
  difficulty: easy
  resources: { cpu: 1, memory: 500Mi }
```

### `difficulty`

Default difficulty class name to use for challenges that do not explicitly set
one.

### `resources`

Default resource request/limits to use for challenges that do not explicitly set
one.

## `deploy`

Controls what challenges are enabled for each environment. 

::: warn
Currently any challenges that have been previously enabled and deployed will 
*not* be un-deployed if they are disabled here.
:::

## `profiles`

### `frontend_url`
### `frontend_token`
### `challenges_domain`
### `kubeconfig`
### `kubecontext`
### `s3`
#### `bucket_name`
#### `endpoint`
#### `region`
#### `access_key`
#### `secret_key`
### `dns`
