# Juntei documentation site

The site uses [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/).
Its top tabs are the main documentation areas; each tab has a focused left sidebar.

## Languages

The site uses `mkdocs-static-i18n` with suffix-based translations, matching Hayabusa's
documentation setup. English is the default language. Translated pages keep the same paths
and add a locale suffix, such as `artifacts/prefetch.ja.md` or
`commands/auto-parse.pt-BR.md`.

The site is published in the same fifteen languages as the Hayabusa documentation: English,
Japanese, Traditional Chinese, Korean, German, Turkish, French, Spanish, Brazilian Portuguese,
Ukrainian, Hindi, Indonesian, Burmese, Thai, and Arabic. Every page has a translation in every
language, and navigation labels for each language are configured in `website/mkdocs.yml`. The
language plugin keeps the same page open when switching languages. Arabic pages render right to
left; code blocks stay left to right.

When updating a page, update its translations too. Leave commands, code blocks, field names,
registry paths, filenames, and link destinations unchanged. Translated headings retain explicit
English anchor IDs so cross-page section links continue to work. English remains the reference
when reviewing a translation, and a new page needs a translation in every language. The website
translations were machine-assisted; corrections from speakers of each language are welcome.

`README-Japanese.md` is the Japanese version of the repository README. Its language links point
back to the English README and to the Japanese documentation.

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

Check that translated pages preserve literal examples, field names, link destinations, and heading
anchor IDs with:

```bash
python website/scripts/check_translations.py
```

The generated `website/site/` directory is ignored by Git.

## Publishing

The workflow in `.github/workflows/docs.yml` validates documentation changes in pull requests and
publishes `main` to GitHub Pages. In the repository settings, select **GitHub Actions** as the Pages
source once before the first deployment.
