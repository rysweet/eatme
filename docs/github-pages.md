# GitHub Pages

The eatme documentation site is built with MkDocs and published with GitHub
Pages.

## Local build

Install the docs dependencies:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements-docs.txt
```

Build the site:

```bash
mkdocs build --strict
```

The generated static files are written to:

```text
site/
```

Do not edit generated files in `site/`. Edit Markdown in `docs/` and rebuild.

## Site configuration

The site is configured by:

```text
mkdocs.yml
```

The configuration defines:

- site name and description
- repository URL
- `docs/` as the source directory
- `site/` as the build output directory
- the built-in MkDocs theme
- navigation for new and existing guides
- strict build mode

## Pull request behavior

On pull requests targeting `master`, the Pages workflow builds the MkDocs site
as validation only. It does not deploy pull request content.

## Publish behavior

On pushes to `master` and manual workflow dispatches, the workflow:

1. Checks out the repository.
2. Sets up Python.
3. Installs `requirements-docs.txt`.
4. Runs `mkdocs build --strict`.
5. Uploads the generated `site/` directory as a Pages artifact.
6. Deploys the artifact to GitHub Pages.

Repository Pages settings should use **GitHub Actions** as the source.

## Required permissions

The build job needs read access to repository contents. The deploy job needs
Pages and OIDC token permissions so GitHub can publish the artifact.

## When to update docs

Update this site when any of these change:

- CLI command names or flags
- scenario schema expectations
- asset validation behavior
- Gadugi adapter generation behavior
- Alice launch-smoke manifest fields
- quality gates or CI behavior
- instructor or student mission contracts

Run `mkdocs build --strict` before opening the PR.
