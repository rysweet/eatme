# Local hook artifacts

Eatme keeps local agent hook runtime files out of the repository. Hook files that
belong to a developer workstation, an amplihack runtime, or a single worktree are
not portable project configuration and are not committed.

## Contents

- [Repository contract](#repository-contract)
- [Feature scope](#feature-scope)
- [Usage](#usage)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Repository contract

The committed repository has no tracked files under:

```text
.github/hooks/
```

Local hook artifacts are excluded because they can contain workstation-specific
paths such as a home directory, a worktree path, or a local agent runtime
location. Those values are valid only for the machine that generated them.

No executable file, asset, generated adapter, CI file, or repository
configuration should refer to local hook artifact names, local hook directories,
or workstation runtime paths. "No committed references" means no references
outside this policy page. This page may name the blocked markers because it
documents the cleanup policy and verification commands.

The repository may add portable hook configuration later only when there is a
documented project need. Portable configuration must use repository-relative
paths, avoid local runtime assumptions, and describe why the hook belongs in the
shared source tree.

## Feature scope

The hook-artifact hygiene feature is a repository cleanup contract, not a shared
hook runtime feature.

It will:

- remove tracked local hook artifacts from the branch
- keep local hook runtime paths out of project-owned files
- document how reviewers verify that the cleanup stayed scoped

It will not:

- install or enable hooks for contributors
- add repository-owned hook scripts
- change scenario assets or generated Gadugi adapters
- change validation semantics, CI behavior, or release metadata

## Usage

Before opening or updating a pull request, confirm that no local hook artifacts
are tracked:

```bash
git ls-files .github/hooks
```

The command should print no file paths.

Check the committed tree for local hook artifact references:

```bash
git --no-pager grep -n -E '(amplihack-hooks|[.]github/hooks|[$][{]HOME[}]/[.]amplihack)' -- ':!docs/local-hook-artifacts.md' ':!target'
```

The command should find no matches outside this policy page. It excludes this
page because the page documents the exact strings that should not appear in
executable configuration, assets, generated adapters, CI, or other
project-owned files.

If a review cites a specific absolute workstation path, search for that exact
literal path as a separate check. Do not use a broad home-directory search:
validation tests may intentionally contain synthetic absolute paths.

Run the normal repository gates after removing hook artifacts from a branch:

```bash
./scripts/quality-gates.sh
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Pair those checks with a scoped diff review. The checks cover Rust behavior,
asset validity, and generated Gadugi freshness; the diff review confirms the
cleanup did not edit canonical scenarios, adapters, validation semantics, or
release metadata.

## Configuration

Eatme has no repository-level hook configuration by default.

Local agent runtimes may generate hook files for a developer's own session. Those
files are local workspace state, not eatme configuration. Keep them untracked and
delete them from a branch if they appear in `git status` as staged or committed
changes.

Do not replace local hook artifacts with shared hook scripts unless the project
also documents:

| Requirement | Reason |
| --- | --- |
| The hook's project-owned purpose | Prevents accidental runtime coupling |
| Repository-relative paths only | Keeps the checkout portable |
| Required environment variables | Makes execution reproducible |
| Failure behavior | Avoids silent local-only policy changes |
| Validation commands | Proves the hook does not change asset or adapter contracts |

## Examples

### Remove accidental hook artifacts from a branch

Delete committed hook artifacts from the branch:

```bash
git rm -r .github/hooks
```

If a local runtime still needs hooks, regenerate them outside the repository
worktree or keep them in local-only state that will not appear in `git status`.
Avoid leaving a copy under the worktree after `git rm --cached`, because that
makes the files easy to stage again by accident.

If untracked hook files already exist and are not needed locally, delete them
from the worktree:

```bash
rm -r .github/hooks
```

Then confirm the branch no longer tracks the directory:

```bash
git ls-files .github/hooks
```

### Review the cleanup diff

The cleanup diff should be limited to deleting local hook artifacts. It should
not edit:

```text
assets/scenarios/eatme/
assets/scenarios/gadugi/
crates/eatme-assets/
crates/eatme-cli/
```

Review the branch before pushing:

```bash
git --no-pager diff --stat
git --no-pager diff --name-only
```

### Keep asset behavior unchanged

Hook artifact cleanup does not regenerate scenarios or adapters. If the Gadugi
freshness check fails, fix the asset or adapter drift as a separate scoped
change:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| `git ls-files .github/hooks` prints paths | Delete the tracked hook files with `git rm -r .github/hooks` |
| `git grep` finds a workstation path in hook config | Remove the local artifact instead of rewriting it into another absolute path |
| A hook is required for all contributors | Add a documented portable configuration with repository-relative paths and validation coverage |
| Gadugi freshness fails after hook cleanup | Treat it as asset drift, not hook cleanup; regenerate or fix adapters in a separate scoped change |
