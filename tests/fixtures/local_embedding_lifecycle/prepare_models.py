"""Prepare pinned public model assets only after explicit operator approval.

This is a standalone preparation command. Importing it starts no work. It does
not run inference, tests, builds, hosted model requests, or credential lookup.
"""

import argparse
import hashlib
import json
from pathlib import Path
import urllib.request


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--primary-dir", type=Path, required=True)
    parser.add_argument("--incompatible-dir", type=Path, required=True)
    args = parser.parse_args()
    if not args.primary_dir.is_absolute() or not args.incompatible_dir.is_absolute():
        raise SystemExit("Fixture asset directories must be absolute paths")
    lock_path = Path(__file__).with_name("incompatible-model.lock.json")
    lock = json.loads(lock_path.read_text(encoding="utf-8"))
    weight = next(asset for asset in lock["assets"] if asset["sourcePath"] == "onnx/model.onnx")
    primary_digest = hashlib.sha256()
    with (args.primary_dir / weight["installedPath"]).open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            primary_digest.update(chunk)
    if primary_digest.hexdigest() == weight["integrity"]["value"]:
        raise SystemExit("Primary weights equal the pinned incompatible weights")
    parent = args.incompatible_dir.parent.resolve(strict=True)
    target = parent / args.incompatible_dir.name
    target.mkdir()
    for asset in lock["assets"]:
        installed = Path(asset["installedPath"])
        if installed.name != asset["installedPath"] or installed.name in ("", ".", ".."):
            raise SystemExit("Pinned asset must have a single local filename")
        identity = asset["integrity"]
        if identity["algorithm"] == "sha256":
            digest = hashlib.sha256()
        elif identity["algorithm"] == "git-blob-sha1":
            digest = hashlib.sha1()
            digest.update(f'blob {asset["sizeBytes"]}\0'.encode("ascii"))
        else:
            raise SystemExit("Unknown pinned asset integrity algorithm")
        written = 0
        with urllib.request.urlopen(asset["sourceUrl"]) as response, (target / installed).open("xb") as output:
            for chunk in iter(lambda: response.read(1024 * 1024), b""):
                written += len(chunk)
                if written > asset["sizeBytes"]:
                    raise SystemExit(f"Oversized pinned asset: {asset['sourcePath']}")
                digest.update(chunk)
                output.write(chunk)
        if written != asset["sizeBytes"] or digest.hexdigest() != identity["value"]:
            raise SystemExit(f"Asset identity mismatch: {asset['sourcePath']}")
        print(f"Prepared {asset['installedPath']}: {written} bytes; {identity['algorithm']}={identity['value']}")
    print(f"Prepared pinned local model {lock['modelId']} at {target}; inference and acceptance remain pending")


if __name__ == "__main__":
    main()
