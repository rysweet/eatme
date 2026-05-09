from __future__ import annotations

import os
import subprocess
import sys
from hashlib import sha256
from pathlib import Path


def main() -> int:
    source_root = Path(__file__).resolve().parent / "_source"
    manifest_path = source_root / "Cargo.toml"
    if not manifest_path.exists():
        print(
            f"eatme source bundle is missing Cargo.toml at {manifest_path}",
            file=sys.stderr,
        )
        return 127

    env = os.environ.copy()
    env.setdefault("CARGO_TARGET_DIR", str(_target_dir(source_root)))
    command = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(manifest_path),
        "-p",
        "eatme-cli",
        "--",
        *sys.argv[1:],
    ]
    try:
        return subprocess.run(command, cwd=source_root, env=env).returncode
    except FileNotFoundError:
        print("cargo is required to run eatme from uvx", file=sys.stderr)
        return 127


def _target_dir(source_root: Path) -> Path:
    cache_home = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache"))
    target_dir = cache_home / "eatme-uvx" / "target" / _source_fingerprint(source_root)
    target_dir.mkdir(parents=True, exist_ok=True)
    return target_dir


def _source_fingerprint(source_root: Path) -> str:
    hasher = sha256()
    source_files = [
        source_root / "Cargo.toml",
        source_root / "Cargo.lock",
        *sorted((source_root / "crates").glob("**/Cargo.toml")),
        *sorted((source_root / "crates").glob("**/*.rs")),
    ]
    for source_file in source_files:
        if not source_file.is_file():
            continue
        hasher.update(source_file.relative_to(source_root).as_posix().encode())
        hasher.update(b"\0")
        hasher.update(source_file.read_bytes())
        hasher.update(b"\0")
    return hasher.hexdigest()[:16]


if __name__ == "__main__":
    raise SystemExit(main())
