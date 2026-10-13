# beavercds-backend

A modern, cloud-native framework for managing CTF challenge deployment.

## What is this?


## Getting Started

Install like any other Rust binary:
```
cargo install --git https://github.com/osusec/beavercds-backend
```

See the documentation and guides for [challenge authors](https://beavercds.info) and [infrastructure admins](https://beavercds.info).

## Contributing

Contributions are welcome! Check out some of the TODO work on the issue tracker
for inspiration.

We use the standard `cargo build` process for building `beavercds` itself, and
there is a test repository under `tests/repo/` to use during development.

Dependencies:
- `rust` @ latest (currently 1.88)
- `cargo`

```bash
# build and run `beavercds check-access` with the config in the test repo
$ (cd tests/repo/ && cargo run -- check-access --profile test)
```

## Documentation

Our documentation is available at [https://beavercds.info] built from source
under `docs/`.

Dependencies (can be installed via `mise`):
- `node` @ lts
- `pnpm` (should be handled by node's corepack)

```bash
cd docs/

mise install      # for node/pnpm
pnpm install      # install vitepress

pnpm docs:build   # build only
pnpm docs:dev     # build and serve
```
