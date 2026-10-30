# Juntei documentation site

The site uses [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/).
Its top tabs are the main documentation areas; each tab has a focused left sidebar.

## Preview locally

```bash
python3 -m venv .venv-docs
. .venv-docs/bin/activate
pip install -r website/requirements.txt
mkdocs serve --config-file website/mkdocs.yml
```

Open <http://127.0.0.1:8000>. Build the same strict production site as CI with:

```bash
mkdocs build --strict --config-file website/mkdocs.yml
```

The generated `website/site/` directory is ignored by Git.

## Publishing

The workflow in `.github/workflows/docs.yml` validates documentation changes in pull requests and
publishes `main` to GitHub Pages. In the repository settings, select **GitHub Actions** as the Pages
source once before the first deployment.
