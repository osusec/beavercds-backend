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
final container image and tag for each challenge. Almost all registries work
with the default format string.

Default, works for most registries (Docker, GHCR, DigitalOcean, self-hosted, ...):
- `"{{domain}}/{{challenge}}-{{container}}:{{profile}}"`

For registries like AWS ECR that require all image registries to be precreated
ahead-of-time, this keeps all the challenge info in the tag so only one ECR
registry needs to be created:
- `"{{domain}}:{{challenge}}-{{container}}-{{profile}}"`

Format: Jinja-style double-braces around field name (`{{ field_name }}`)

The required and only fields are:

- `domain`: the domain config field above; the repository base URL
- `challenge`: challenge name, slugified
- `container`: name of the specific pod in the challenge this image is for
- `profile`: the current deployment profile, for isolating images between environments

If setting a custom format, you must use all four of these fields in order for challenge images to not overwrite each other.

Example:

For challenge `pwn/notsh`, chal pod container `main`, profile `prod`, and domain `registry.io/myctf`:

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

Registry credentials that will be used locally to push up challenge container images. This must have push permissions. 

Format: `{ user: "registry-username", pass: "registry-password" }`

```yaml
registry:
  build:
    user: fakeuser
    pass: notrealpass
```

### `cluster`

Registry credentials that will be used in the Kubernetes cluster to pull the challenge container images. This must have pull permissions, but does not need push. 

Format: `{ user: "registry-username", pass: "registry-password" }`

```yaml
registry:
  cluster:
    user: alsofake
    pass: stillnotreal
```

## `defaults`

### `difficulty`

### `resources`


## `points`


## `deploy`


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
