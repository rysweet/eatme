from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import textwrap
import tomllib
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock


REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_ROOT = REPO_ROOT / "src"

sys.path.insert(0, str(SRC_ROOT))

from eatme_uvx import cli  # noqa: E402


class CargoProfileTests(unittest.TestCase):
    def test_dev_and_test_profiles_use_line_table_debug_info(self) -> None:
        cargo_toml = tomllib.loads((REPO_ROOT / "Cargo.toml").read_text())

        self.assertEqual(cargo_toml["profile"]["dev"]["debug"], "line-tables-only")
        self.assertEqual(cargo_toml["profile"]["test"]["debug"], "line-tables-only")


class UvxCargoTargetDirTests(unittest.TestCase):
    def run_main_with_env(self, env: dict[str, str]) -> dict[str, str]:
        captured_env: dict[str, str] = {}

        def fake_run(command: list[str], cwd: Path, env: dict[str, str]) -> SimpleNamespace:
            captured_env.update(env)
            return SimpleNamespace(returncode=0)

        with (
            mock.patch.dict(os.environ, env, clear=True),
            mock.patch.object(sys, "argv", ["eatme", "--help"]),
            mock.patch.object(cli.Path, "exists", return_value=True),
            mock.patch.object(subprocess, "run", side_effect=fake_run),
        ):
            self.assertEqual(cli.main(), 0)

        return captured_env

    def test_eatme_cargo_target_dir_overrides_cargo_target_dir(self) -> None:
        env = self.run_main_with_env(
            {
                "EATME_CARGO_TARGET_DIR": "/cache/eatme",
                "CARGO_TARGET_DIR": "/cache/cargo",
            }
        )

        self.assertEqual(env["CARGO_TARGET_DIR"], "/cache/eatme")

    def test_existing_cargo_target_dir_is_preserved_without_eatme_override(self) -> None:
        env = self.run_main_with_env({"CARGO_TARGET_DIR": "/cache/cargo"})

        self.assertEqual(env["CARGO_TARGET_DIR"], "/cache/cargo")

    def test_uvx_cache_target_dir_is_used_when_no_target_env_is_set(self) -> None:
        with tempfile.TemporaryDirectory() as cache_home:
            with mock.patch.object(cli.Path, "home", side_effect=AssertionError):
                env = self.run_main_with_env({"XDG_CACHE_HOME": cache_home})

        self.assertEqual(env["CARGO_TARGET_DIR"], f"{cache_home}/eatme-uvx/target")

    def test_empty_xdg_cache_home_uses_home_cache_target_dir(self) -> None:
        with tempfile.TemporaryDirectory() as home_dir:
            with mock.patch.object(cli.Path, "home", return_value=Path(home_dir)):
                env = self.run_main_with_env({"XDG_CACHE_HOME": ""})

        self.assertEqual(env["CARGO_TARGET_DIR"], f"{home_dir}/.cache/eatme-uvx/target")


class QualityGateTargetDirTests(unittest.TestCase):
    def run_quality_gates(self, env_overrides: dict[str, str]) -> list[tuple[str, str, str]]:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            bin_dir = temp_path / "bin"
            bin_dir.mkdir()
            log_path = temp_path / "cargo-env.log"
            cargo_shim = bin_dir / "cargo"
            cargo_shim.write_text(
                textwrap.dedent(
                    f"""\
                    #!/usr/bin/env sh
                    printf '%s|%s|%s\\n' "$1" "${{CARGO_TARGET_DIR-}}" "${{EATME_CARGO_TARGET_DIR-}}" >> "{log_path}"
                    exit 0
                    """
                )
            )
            cargo_shim.chmod(0o755)

            env = os.environ.copy()
            env["PATH"] = f"{bin_dir}{os.pathsep}{env['PATH']}"
            env["TMPDIR"] = str(temp_path / "tmp")
            env["HOME"] = str(temp_path / "home")
            Path(env["HOME"]).mkdir()
            env.pop("CARGO_TARGET_DIR", None)
            env.pop("EATME_CARGO_TARGET_DIR", None)
            env.pop("XDG_CACHE_HOME", None)
            env.update(env_overrides)

            subprocess.run(
                ["bash", str(REPO_ROOT / "scripts" / "quality-gates.sh")],
                cwd=REPO_ROOT,
                env=env,
                check=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )

            return [
                tuple(line.split("|"))
                for line in log_path.read_text().splitlines()
                if line
            ]

    def test_eatme_cargo_target_dir_is_exported_as_cargo_target_dir(self) -> None:
        calls = self.run_quality_gates({"EATME_CARGO_TARGET_DIR": "/cache/eatme"})

        self.assertGreaterEqual(len(calls), 4)
        self.assertTrue(all(cargo_target == "/cache/eatme" for _, cargo_target, _ in calls))

    def test_existing_cargo_target_dir_is_preserved_when_eatme_override_is_absent(self) -> None:
        calls = self.run_quality_gates({"CARGO_TARGET_DIR": "/cache/cargo"})

        self.assertGreaterEqual(len(calls), 4)
        self.assertTrue(all(cargo_target == "/cache/cargo" for _, cargo_target, _ in calls))

    def test_xdg_cache_home_shared_target_is_used_when_no_override_is_configured(self) -> None:
        with tempfile.TemporaryDirectory() as cache_home:
            calls = self.run_quality_gates({"XDG_CACHE_HOME": cache_home})

        self.assertGreaterEqual(len(calls), 4)
        self.assertTrue(
            all(
                cargo_target == f"{cache_home}/eatme/cargo-target"
                for _, cargo_target, _ in calls
            )
        )

    def test_home_cache_shared_target_is_used_when_xdg_cache_home_is_empty(self) -> None:
        with tempfile.TemporaryDirectory() as home_dir:
            calls = self.run_quality_gates({"XDG_CACHE_HOME": "", "HOME": home_dir})

        self.assertGreaterEqual(len(calls), 4)
        self.assertTrue(
            all(
                cargo_target == f"{home_dir}/.cache/eatme/cargo-target"
                for _, cargo_target, _ in calls
            )
        )


if __name__ == "__main__":
    unittest.main()
