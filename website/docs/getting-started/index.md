# Install Juntei

## Release binaries

Check the [Juntei releases page](https://github.com/Yamato-Security/juntei/releases) for a binary
for your operating system. If no release binary is published yet, compile from source as described
below. Keep the archive or source commit and checksum with your case notes so another analyst can
reproduce the run.

```bash
juntei --version
juntei --help
```

On Windows, the executable may be named `juntei.exe`; the examples use `juntei` for readability.

## Compile from source

Juntei currently requires the Rust version declared in `Cargo.toml` (Rust 1.99 at the time this
page was written).

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` uses the exact dependency revisions recorded in `Cargo.lock`.

## Basic command shape

Every parser accepts either one file (`-f`) or a directory collection (`-d`):

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

Display current options rather than copying an old command from a report:

```bash
juntei parse-prefetch --help
juntei parse-registry read --help
```

Continue with [First Investigation](first-investigation.md) for a safe workflow.
