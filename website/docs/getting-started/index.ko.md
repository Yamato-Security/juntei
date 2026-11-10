# Juntei 설치 { #install-juntei }

## 릴리스 바이너리 { #release-binaries }

[Juntei 릴리스 페이지](https://github.com/Yamato-Security/juntei/releases)에서 사용 중인 시스템에 맞는
zip 파일을 다운로드하십시오. 아직 릴리스 바이너리가 게시되지 않았다면 아래 설명에 따라 소스에서
컴파일하십시오. 다른 분석가가 실행을 재현할 수 있도록 아카이브 또는 소스 커밋과 체크섬을 사건 기록과 함께
보관하십시오.

| 시스템 | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` 또는 `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` 또는 `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

각 zip에는 버전과 플랫폼이 이름에 포함된 바이너리(예: `juntei-1.0.0-win-x64.exe`)만 들어 있습니다.
Juntei에는 다른 파일이 필요하지 않으며, Windows 바이너리에는 Visual C++ 재배포 가능 패키지가 필요하지
않습니다. musl Linux 바이너리는 정적으로 링크되어 있어 모든 배포판에서 실행되며, gnu 바이너리는 최신
glibc가 필요합니다. macOS에서는 다운로드한 바이너리를 시스템 설정 > 개인정보 보호 및 보안에서 허용하거나,
`xattr -d com.apple.quarantine juntei-<version>-mac-*`로 격리 속성을 제거하십시오.

```bash
juntei --version
juntei --help
```

예제에서는 바이너리 이름으로 `juntei`를 사용합니다. 바이너리의 이름을 바꾸거나 전체 이름을 입력하십시오.

## 소스에서 컴파일 { #compile-from-source }

Juntei는 현재 `Cargo.toml`에 선언된 Rust 버전(이 페이지 작성 시점 기준 Rust 1.99)이 필요합니다.

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked`는 `Cargo.lock`에 기록된 정확한 의존성 리비전을 사용합니다.

## 기본 명령 형식 { #basic-command-shape }

모든 파서는 단일 파일(`-f`) 또는 디렉터리 수집본(`-d`) 중 하나를 입력으로 받습니다.

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

보고서에 있는 오래된 명령을 복사하기보다 현재 옵션을 표시하여 확인하십시오.

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

안전한 워크플로는 [첫 번째 조사](first-investigation.md)에서 계속 확인하십시오.
