#!/usr/bin/env bash
set -u

git_root_output="$(git rev-parse --show-toplevel 2>&1)"
git_root_status=$?

if [ "$git_root_status" -ne 0 ]; then
  echo "status=not-a-git-worktree"
  echo "git rev-parse --show-toplevel failed"
  echo "$git_root_output"
  exit 2
fi

repository_root="$git_root_output"
echo "repository_root=$repository_root"

status_output="$(git -C "$repository_root" status --short 2>&1)"
status_status=$?

if [ "$status_status" -ne 0 ]; then
  echo "status=git-status-failed"
  echo "$status_output"
  exit 3
fi

if [ -n "$status_output" ]; then
  echo "status=dirty"
  echo "$status_output"

  diff_stat="$(git -C "$repository_root" diff --stat 2>&1)"
  diff_stat_status=$?
  if [ "$diff_stat_status" -ne 0 ]; then
    echo "git diff --stat failed"
    echo "$diff_stat"
    exit 4
  fi

  if [ -n "$diff_stat" ]; then
    echo "$diff_stat"
  fi

  exit 1
fi

echo "status=clean-noop"
