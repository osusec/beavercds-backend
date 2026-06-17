# Challenge repo for my CTF

Challenges repo for this CTF.

## Adding a new challenge

Run `scripts/new-chal.py` and answer the prompts (no venv required):

```sh
# Have it prompt for details:
python3 scripts/new-chal.py

# Or via flags:
python3 scripts/new-chal.py --help # for available options
```

This will create a new folder under `<category>/<name>/` with the challenge
info. Make sure to edit the generated `challenge.yaml` to fill in more details!

## Deployment tooling

This repo uses [`beavercds`](https://beavercds.info) to manage and deploy
challenges. Install it from https://github.com/osusec/beavercds-ng:

```
cargo install --git https://github.com/osusec/beavercds-ng.git
```

See the documentation at <https://beavercds.info> for more information.
