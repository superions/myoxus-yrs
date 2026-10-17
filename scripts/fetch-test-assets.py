#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Daniel Scharf
"""Fetch the exact upstream datasets required by the inherited tests."""

import hashlib
import json
from pathlib import Path
from urllib.request import urlopen


ROOT = Path(__file__).resolve().parent.parent
MANIFEST = json.loads((ROOT / "upstream/test-assets.json").read_text())


def git_blob_id(data):
    # Git hashes the object header as well as the file bytes.
    header = b"blob " + str(len(data)).encode("ascii") + b"\0"
    return hashlib.sha1(header + data).hexdigest()


def main():
    for asset in MANIFEST["files"]:
        relative = Path(asset["path"])
        if relative.is_absolute() or ".." in relative.parts or relative.parts[0] != "assets":
            raise ValueError(f"Invalid asset path: {relative}")
        destination = ROOT / relative
        expected = asset["gitBlobSha1"]
        if destination.is_file() and git_blob_id(destination.read_bytes()) == expected:
            print(f"Verified {relative}")
            continue
        url = f"https://raw.githubusercontent.com/y-crdt/y-crdt/{MANIFEST['commit']}/{relative.as_posix()}"
        with urlopen(url, timeout=60) as response:
            data = response.read()
        if git_blob_id(data) != expected:
            raise ValueError(f"Upstream asset hash mismatch: {relative}")
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
        print(f"Fetched and verified {relative}")


if __name__ == "__main__":
    main()
