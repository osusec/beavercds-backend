#!/usr/bin/env python3

# Standardlib only!
import argparse
import os
import readline
import sys

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

args = parser.parse_args()


# If no cli flags were specified, prompt for them instead

print(f"DBG: raw args: {args}")

while args.name is None or args.name == "":
    args.name = input("Name of challenge? ")

while args.category is None or args.category == "":
    args.category = input("Category? ")

while args.author is None or args.author == "":
    args.author = input("Author(s)? ")

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

# create challenge.yaml


print(f"DBG: after prompt: {args}")
