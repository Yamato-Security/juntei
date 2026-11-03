# Copyright (C) 2026 Shirofune Security Co., Ltd.
#
# Unless explicitly acquired and licensed from Licensor under another license, the contents of this
# file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
# allowed by the RPL, and You may not copy or use this file in either source code or executable
# form, except in compliance with the terms and conditions of the RPL.
#
# All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
# OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
# INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
# PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
# and limitations under the RPL.

"""Validate literal examples and link destinations in the translated documentation."""
from collections import Counter
from pathlib import Path
import re
import sys

DOCS = Path(__file__).resolve().parents[1] / "docs"
LOCALES = (
    "ja", "zh-TW", "ko", "de", "tr", "fr", "es", "pt-BR",
    "uk", "hi", "id", "my", "th", "ar",
)
LOCALE_SUFFIX = re.compile(r"\.([a-z]{2}(?:-[A-Z]{2})?)\.md$")


def literals(text):
    fences, prose, current = [], [], []
    delimiter = None
    for line in text.splitlines(keepends=True):
        marker = re.match(r"\s*(`{3,}|~{3,})", line)
        if delimiter is not None:
            current.append(line)
            if marker and marker[1][0] == delimiter[0] and len(marker[1]) >= len(delimiter):
                fences.append("".join(current))
                current, delimiter = [], None
        elif marker:
            current, delimiter = [line], marker[1]
        else:
            prose.append(line)
    if delimiter is not None:
        raise ValueError("Unclosed code fence")
    prose = "".join(prose)
    inline = Counter(
        re.sub(r"\s*\n\s*", " ", match[2])
        for match in re.finditer(r"(`+)((?:(?!\1)[\s\S])*?)\1", prose)
    )
    inline.update(re.findall(r"<code>(.*?)</code>", prose, flags=re.S))
    links = Counter(re.findall(r"\]\(([^()\n]*(?:\([^()\n]*\)[^()\n]*)*)\)", prose))
    return fences, inline, links


def main():
    sources = sorted(p for p in DOCS.rglob("*.md") if not LOCALE_SUFFIX.search(p.name))
    failures, coverage = [], Counter()
    for source in sources:
        reference = literals(source.read_text())
        for locale in LOCALES:
            path = source.with_name(f"{source.stem}.{locale}.md")
            if not path.exists():
                continue  # The language plugin explicitly supports English fallback.
            coverage[locale] += 1
            actual = literals(path.read_text())
            for index, label in enumerate(("code blocks", "inline literals", "link targets")):
                if actual[index] != reference[index]:
                    failures.append(f"{path.relative_to(DOCS)}: changed {label}")
                    if index > 0:
                        failures.append(f"  missing: {dict(reference[index] - actual[index])}")
                        failures.append(f"  extra: {dict(actual[index] - reference[index])}")
            if re.search(r"(?:XQZ\d+ZQX|ZXQSEPARATEQXZ)", path.read_text()):
                failures.append(f"{path.relative_to(DOCS)}: unresolved translation marker")
    print(f"English pages: {len(sources)}")
    for locale in LOCALES:
        print(f"{locale}: {coverage[locale]} translated pages")
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    print("Commands, inline literals, and link destinations match the English sources.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
