# Install Juntei

## Release binaries

Download the zip for your system from the
[Juntei releases page](https://github.com/Yamato-Security/juntei/releases). If no release binary is
published yet, compile from source as described below. Keep the archive or source commit and
checksum with your case notes so another analyst can reproduce the run.

| System | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` or `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` or `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

Each zip holds only the binary, named with the version and platform (for example
`juntei-1.0.0-win-x64.exe`): Juntei needs no other files, and the Windows binary does not need the
Visual C++ Redistributable. The musl Linux binaries are statically linked and run on any
distribution; the gnu ones need a recent glibc. On macOS, allow the downloaded binary in
System Settings > Privacy & Security, or remove its quarantine attribute with
`xattr -d com.apple.quarantine juntei-<version>-mac-*`.

```bash
juntei --version
juntei --help
```

The examples use `juntei` for the binary; rename it, or type its full name.

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
juntei parse-file prefetch --help
juntei parse-registry read --help
```

Continue with [First Investigation](first-investigation.md) for a safe workflow.
