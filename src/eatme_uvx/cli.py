from __future__ import annotations

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
    env["CARGO_TARGET_DIR"] = str(_cargo_target_dir(env))
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


def _cargo_target_dir(env: dict[str, str]) -> Path | str:
    eatme_target_dir = env.get("EATME_CARGO_TARGET_DIR")
    if eatme_target_dir:
        return eatme_target_dir

    cargo_target_dir = env.get("CARGO_TARGET_DIR")
    if cargo_target_dir:
        return cargo_target_dir

    return _uvx_target_dir()


def _uvx_target_dir() -> Path:
    cache_home = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache"))
    target_dir = cache_home / "eatme-uvx" / "target"
    target_dir.mkdir(parents=True, exist_ok=True)
    return target_dir


if __name__ == "__main__":
    raise SystemExit(main())
