# Third-party notices

Juntei is released under the Reciprocal Public License 1.5 ([`LICENSE`](../LICENSE)). It is built
with code and data from others under their own licenses, whose notices are kept here. Ship this
directory with every copy of Juntei, in source or binary form.

| File | Covers |
|---|---|
| [`crates.txt`](crates.txt) | Every Rust crate Juntei is built with, Yamato Security's parsers included: each license text once, followed by the crates under it, with the license files found inside the crates. |
| [`PyrsistenceSniper-LICENSE.txt`](PyrsistenceSniper-LICENSE.txt) | `persistence-hunter` (`src/persistence_hunter/`), adapted from Hexastrike's PyrsistenceSniper (MIT): its check catalogue, default profile and LOLBAS name snapshot. |
| [`mft-csv-LICENSE.txt`](mft-csv-LICENSE.txt) | The parts of `parse-ntfs mft`'s CSV (`src/mft_csv.rs`) adapted from MIT-licensed code. |
| [`lnk-guid-names-LICENSE.txt`](lnk-guid-names-LICENSE.txt) | The GUID name table that ys-lnk-parser compiles in (its `src/guid_names/`, MIT). |
| [`jump-lists-appid-LICENSE.txt`](jump-lists-appid-LICENSE.txt) | The AppID table that ys-jump-lists-parser compiles in (its `src/appid/`, MIT). |

The test files in `test_files/` are not part of the program; their licenses are next to them.

## Updating crates.txt

`crates.txt` is generated from `Cargo.lock` by
[cargo-about](https://github.com/EmbarkStudios/cargo-about), and CI fails when it is out of date.
After adding or updating a dependency, regenerate it from the repository's root:

```bash
cargo install cargo-about --locked --features cli
cargo about generate --frozen --fail -c THIRD_PARTY_NOTICES/about.toml THIRD_PARTY_NOTICES/about.hbs -o THIRD_PARTY_NOTICES/crates.txt
```

`--fail` stops on a license that [`about.toml`](about.toml) does not accept. Only add a license
there once you have checked that it allows Juntei to be distributed under the RPL and under a
commercial license.

When a new file adapts someone else's code or data, add its license here and point to it from the
file.
