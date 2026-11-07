# Browser test databases

Kept gzipped; the tests decompress them.

`chromium150/` and `firefox140/` were made by Yamato Security by driving Chromium 150 and
Firefox 140 (headless, on Linux, over the DevTools protocol and WebDriver BiDi) through a local
test site at `127.0.0.1:8765` and `localhost:8765`: typed visits, links, a redirect, cookies, a
download and two form submissions. The Firefox profile also holds the default bookmarks of the
browser package it was made with.

| Files | Rows (`parse-sqlite browser`) |
| --- | --- |
| `chromium150/History` | 13 visits, 1 download |
| `chromium150/Cookies` | 10 cookies |
| `chromium150/Web Data` | 3 form entries |
| `firefox140/places.sqlite`, `places.sqlite-wal` | 13 visits, 1 download; 2 deleted visits and their 2 deleted pages |
| `firefox140/cookies.sqlite`, `cookies.sqlite-wal` | 6 cookies, all in the `-wal` file only |
| `firefox140/formhistory.sqlite` | 3 form entries |
| `formhistory-wal/formhistory.sqlite`, `formhistory.sqlite-wal` | 4 form entries, 1 deleted entry and its removal record |

The Chromium databases were copied while the browser ran; Chromium keeps them with rollback
journals, which are not included. The Firefox cookies were copied while the browser ran, before it
checkpointed its log. In `places.sqlite`, two pages and their visits (`/page2` and `/page9`) were
then deleted as Firefox deletes, with `secure_delete` on, in the write-ahead log only: without the
`-wal` file the database reads 15 visits.

`formhistory-wal/` is the `wal/` form history of
[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser)'s tests
(`browser/tests/fixtures/formhistory/`), written by its `gen.sh` with the `sqlite3` shell in
Firefox 140's schema from synthetic values.
