#!/usr/bin/env python3

# Standardlib only!
import argparse
import hashlib
import os
import re
import sys
import time
from pathlib import Path

parser = argparse.ArgumentParser(
    description="""
    Create a new challenge directory and challenge.yaml config.
    This will prompt for any info not passed as cli flags.
    """
)

parser.add_argument(
    "-n",
    "--name",
    help="Name of the challenge as shown to the user on the scoreboard",
)
parser.add_argument(
    "-c",
    "--category",
    help="Category to put the challenge under (e.g. 'pwn', 'misc')",
)
parser.add_argument(
    "-a",
    "--author",
    help="Challenge author(s), if multiple put all as one string",
)
parser.add_argument("-f", "--flag", help="Flag for the challenge")
parser.add_argument(
    "-t", "--type", help="Type of challenge, either tcp, web, or static"
)

args = parser.parse_args()


# If no cli flags were specified, prompt for them instead

print(f"DBG: raw args: {args}")

while args.name is None or args.name == "":
    args.name = input("Name of challenge? ")

while args.category is None or args.category == "":
    args.category = input("Category? ").lower()

while args.author is None or args.author == "":
    args.author = input("Author(s)? ")

while args.type not in ["tcp", "web", "static"]:
    args.type = input("Type of challenge (tcp, web, static)? ").lower()

# make sure category dir exists
if not os.path.isdir(args.category):
    print(
        f"WARN: category directory '{args.category}' does not exist!", file=sys.stderr
    )
    # offer to create if its missing
    create = input(f"Create category '{args.category}' now? [y/N]").lower()
    if create == "y":
        os.mkdir(args.category)
    else:
        # otherwise error out
        print("not creating")
        exit(1)

# create challenge name in category dir
slug = args.name.lower().replace(" ", "-")
chaldir = Path(args.category) / slug

if os.path.isdir(chaldir):
    print(f"ERR: challenge directory '{chaldir}' already exists")
    exit(1)
os.mkdir(chaldir)

# generate challenge id from info and timestamp, git-style truncated hash
h = hashlib.sha256(args.name.encode())
h.update(str(time.time()).encode())
id = h.hexdigest()[:8]

# add connection info template to challenge description if needed
conn = {"tcp": "{{ nc }}", "http": "{{ link }}", "static": ""}[args.type]

# template out challenge.yaml
yaml = f"""
name: "{args.name}"
author: "{args.author}"
description: |
  Your description here.

  {conn}

challenge_id: "{id}" # DON'T CHANGE THIS!

flag:
  file: ./flag
"""

with open(chaldir / "challenge.yaml", "w") as f:
    f.write(yaml)

print(f"Created new challenge at {chaldir}. Make sure to edit the challenge.yaml!")
