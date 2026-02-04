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
while args.name is None or args.name == "":
    args.name = input("Name of challenge? ")

while args.category is None or args.category == "":
    args.category = input("Category? ").lower()

while args.author is None or args.author == "":
    args.author = input("Author(s)? ")

while args.type not in ["tcp", "web", "static"]:
    args.type = input("Type of challenge (tcp, web, static)? ").lower()

while args.flag is None or args.flag == "":
    args.flag = input("Flag? ")


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
# and a port number somewhere in 3xxxx
port = 30000 + int(h.hexdigest(), base=16) % 10000

# add connection info template to challenge description if needed
connstr = {"tcp": "{{ nc }}", "web": "{{ link }}", "static": ""}[args.type]

# template out challenge.yaml
yaml = f"""
name: "{args.name}"
author: "{args.author}"
description: |
  Your description here.

  {connstr}

challenge_id: "{id}" # DON'T CHANGE THIS!

flag:
  file: ./flag

"""


# fill in more relevant provide/pods info per challenge type
if args.type == "static":
    yaml += """
# Files that will get provided to the player on the scoreboard:
provide:
  # single file from this directory
  - example.png

  # multiple files in an archive
  - as: multiple.zip
    include:
      - foo
      - bar
"""

else:
    if args.type == "tcp":
        exposestr = f"tcp: {port}"
    else:
        exposestr = f"http: {slug}"

    yaml += f"""
# Files that will get provided to the player on the scoreboard:
provide:
  # single file from this directory
  - example.png

  # files from inside the container from `pods:`
  - from: mainctr
    include:
      - /usr/lib/x86_64-linux-gnu/libc.so.6

  # or multiple in an archive
  - as: multiple.zip
    include:
      - foo
      - bar

pods:
  - name: mainctr           # Used by `provide` includes above
    build: .                # Where your container Dockerfile is
    replicas: 2
    ports:
      - internal: 1337      # What your container listens on
        expose:
          # What it will be exposed as publically
          {exposestr}
"""


with open(chaldir / "challenge.yaml", "w") as f:
    f.write(yaml)

with open(chaldir / "flag", "w") as f:
    f.write(args.flag + "\n")

print(f"Created new challenge at {chaldir}. Make sure to edit the challenge.yaml!")
