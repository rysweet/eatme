#!/usr/bin/env bash
set -euo pipefail

if ! repo_root="$(git rev-parse --show-toplevel 2>&1)"; then
  echo "error: git rev-parse --show-toplevel failed: $repo_root" >&2
  echo "hint: run this guard inside a git worktree or git repository." >&2
  exit 2
fi

cd "$repo_root"

if ! git diff --quiet; then
  echo "error: guard found unstaged changes from resolved git root: $repo_root" >&2
  echo "hint: inspect with git diff and commit or remove the task-scoped changes before declaring a no-op." >&2
  exit 1
fi

if ! git diff --cached --quiet; then
  echo "error: guard found staged changes from resolved git root: $repo_root" >&2
  echo "hint: inspect with git diff --cached and commit or unstage the task-scoped changes before declaring a no-op." >&2
  exit 1
fi

porcelain="$(git status --porcelain)"
if [[ -n "$porcelain" ]]; then
  echo "error: guard found changes with git status --porcelain from resolved git root: $repo_root" >&2
  echo "$porcelain" >&2
  exit 1
fi

echo "no task-scoped changes from resolved git root: $repo_root"
