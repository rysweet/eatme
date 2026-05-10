from __future__ import annotations

import hashlib
import os
import subprocess
import sys
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
    source_key = hashlib.sha256(str(source_root).encode()).hexdigest()[:16]
    target_dir = cache_home / "eatme-uvx" / "targets" / source_key
    target_dir.mkdir(parents=True, exist_ok=True)
    return target_dir


if __name__ == "__main__":
    raise SystemExit(main())
