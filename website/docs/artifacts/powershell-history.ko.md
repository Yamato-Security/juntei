# PowerShell 기록 { #powershell-history }

## 기록에 저장되는 내용 { #what-the-history-records }

PSReadLine은 대화형 PowerShell 호스트에 입력한 명령줄을 저장합니다. 파일에는 저장 순서대로 텍스트가 들어 있으며 명령별 타임스탬프는 없습니다.

## 위치 및 수집 { #locations-and-collection }

Windows PowerShell 5.1과 PowerShell 7+의 Windows 기본 위치는 다음과 같습니다.

```text
%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\<Host>_history.txt
C:\Users\<user>\AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt
```

다른 호스트는 해당 디렉터리에 `Visual Studio Code Host_history.txt` 같은 이름을 사용합니다. Linux와 macOS의 PowerShell은 보통 `~/.local/share/powershell/PSReadLine/<Host>_history.txt`를 사용합니다. PSReadLine의 `HistorySavePath`로 파일 위치를 변경할 수 있습니다. 이 기본 경로들은 탐색 시의 가정이며 모든 위치를 망라하지 않습니다.

디렉터리 입력과 `auto-parse`는 `PSReadLine` 폴더 바로 아래의 `<Host>_history.txt`를 재귀적으로 선택하며 ASCII 대소문자를 구분하지 않습니다. 다른 위치의 임의의 `*_history.txt` 파일은 선택하지 않습니다. 이름을 바꾼 파일이나 `PSReadLine` 디렉터리 밖의 사본에는 명시적으로 `-f` 입력을 사용하세요. 사용자 식별을 위해 가능하면 원래 프로필 구조를 보존하세요.

## 분석 시 주의 사항 { #analyst-cautions }

- 어떤 줄에도 시간이 없습니다. `FileLastWriteTime`은 파일의 수정 시각입니다. PSReadLine은 기본적으로 명령을 실행할 때마다 기록을 저장하므로(`HistorySaveStyle` `SaveIncrementally`) 이 시각은 보통 마지막 줄이 추가된 시점과 가깝지만, 그보다 앞선 줄에 대해서는 아무것도 알려 주지 않습니다.
- 저장된 줄은 명령이 실행되었거나 성공적으로 완료되었다는 증거가 아닙니다. 실행 여부와 시각은 PowerShell 이벤트 로그, 프로세스 원격 분석 및 다른 실행 아티팩트와 대조하세요.
- 기록은 편집, 삭제하거나 끌 수 있습니다(`HistorySaveStyle` `SaveNothing`). 또한 PSReadLine은 `password`, `token`, `apikey`가 들어 있는 줄처럼 비밀 정보가 포함된 것으로 보이는 줄을 저장하지 않으며, Windows PowerShell ISE는 PSReadLine을 사용하지 않습니다. 스크립트와 `-Command` 또는 `-EncodedCommand`로 실행한 명령은 직접 입력한 것이 아니므로 저장되지 않습니다.
- 여러 줄로 된 명령은 마지막 줄을 제외한 모든 줄 끝에 백틱을 붙여 저장됩니다. Juntei는 이 줄들을 별도의 행으로 유지하고 `EntryStartLine`으로 연결합니다. 직접 입력한 백틱으로 끝나는 줄도 똑같이 보이므로 한 항목의 줄들은 함께 읽으세요.

## Juntei로 파싱 { #parse-with-juntei }

```bash
juntei parse-file powershell-history -d evidence/ -o powershell-history.csv
juntei parse-file powershell-history -f copied-history.txt -t jsonl
```

기본 파일 형식은 CSV입니다. `-t csv`, `-t json` 또는 `-t jsonl`을 지정하고 `-o`를 생략하면 레코드를 stdout으로 출력합니다. 두 옵션 모두 없으면 요약만 표시합니다. JSON은 읽기 좋게 정렬된 개별 객체로 구성되며 JSONL은 한 줄에 객체 하나를 씁니다. 기존 출력을 덮어쓰려면 `-C`를 추가하세요. 소스 파일을 가리키는 별칭 경로는 교체되지 않도록 보호됩니다.

## 인코딩과 줄 처리 { #encoding-and-line-handling }

- 파일 시작의 UTF-8, UTF-16LE 또는 UTF-16BE 바이트 순서 표시로 인코딩을 결정하고 표시를 제거합니다. BOM이 없으면 항상 UTF-8로 처리합니다. BOM 없는 UTF-16은 추측하지 않으며 NUL 문자는 보존하고 인코딩 손상 가능성을 보고합니다.
- 잘못된 UTF-8 시퀀스와 짝이 없는 UTF-16 서로게이트는 U+FFFD로 바꾸고 `Problems`에 손상을 보고합니다. 끝에 남은 UTF-16 바이트 하나와 읽기 오류도 보고하며 읽을 수 있는 줄은 유지합니다. 완전성을 판단하기 전에 `Problems`와 파서 로그를 모두 확인하세요.
- 파일은 스트리밍으로 처리합니다. 물리적 줄당 최대 1,048,576 입력 바이트를 유지하며 BOM과 LF 구분자는 제외합니다(LF 앞의 CR은 제한에 포함). 더 긴 줄은 완전한 문자 경계에서 자르고 나머지 바이트도 읽어 소비합니다. `Problems`는 생략한 바이트 수를 보고합니다. 제한은 문자 수가 아니라 인코딩된 입력 바이트에 적용됩니다.
- LF와 CRLF 줄 끝을 제거하며 줄바꿈 없이 끝나는 줄의 마지막 CR도 제거합니다. 그 밖의 공백과 탭은 보존합니다. 빈 줄은 레코드를 생성하지 않지만 줄 번호에 포함되고 연속 줄 체인을 끝냅니다. 공백 문자만 있는 줄은 레코드로 남습니다. 읽기 오류가 나면 읽기가 중단된 위치를 표시하는 빈 진단 행이 생성될 수 있습니다.
- 줄은 분리된 상태로 백틱과 들여쓰기를 보존합니다. `EntryStartLine`은 앞선 줄들이 백틱으로 끝나는 연속 체인의 첫 줄을 가리킵니다. 셸 구문을 해석하거나 명령을 재구성하지 않습니다. 잘림은 연속 관계 식별에 영향을 줄 수 있으므로 이런 행을 묶기 전에 `Problems`를 확인하세요.

## CSV 출력 필드 { #csv-output-fields }

| 필드 | 해석 |
|---|---|
| `SourceFile` | 입력 디렉터리 기준 상대 경로이며 `/`를 사용합니다. `-f`에서는 파일명만 출력합니다. 절대 소스 경로는 내보내지 않습니다. |
| `User` | 경로에서 판단한 프로필 이름. `AppData\Roaming\Microsoft\Windows\PowerShell\PSReadLine`(또는 `.local/share/powershell/PSReadLine`) 바로 위의 폴더 이름이 사용자 이름이며, 입력이 해당 프로필이나 `PSReadLine` 폴더 자체인 경우에도 마찬가지입니다. 디렉터리 입력은 입력 루트 아래의 `Users`/`Documents and Settings` 구조도 인식합니다. 알 수 없는 값은 CSV에서 비어 있고 JSON에서는 `null`입니다. |
| `ShellHost` | `_history.txt` 앞의 파일명 접두부(예: `ConsoleHost`). 이름 변경으로 해당 접미사가 없으면 알 수 없습니다. |
| `LineNumber` | 1부터 시작하는 물리적 줄 번호. 건너뛴 빈 줄도 포함합니다. |
| `EntryStartLine` | 연속 줄 체인의 시작 물리적 줄 번호 또는 이 줄 자체의 번호. |
| `Command` | 줄 끝을 제거한 디코딩된 물리적 줄. 문서화된 제한이 적용됩니다. |
| `FileLastWriteTime` | 각 행에 반복되는 소스 파일 수정 시각. **파일 메타데이터이며 명령 실행 시각이 아닙니다.** 이미지 파싱은 임시 준비 시각이 아닌 원래 소스 메타데이터를 사용합니다. |
| `FileSize` | 소스 파일 크기(바이트). |
| `Encoding` | `UTF-8`, `UTF-8-BOM`, `UTF-16LE` 또는 `UTF-16BE`. |
| `Problems` | 줄 손상과 잘림 진단. CSV에서는 세미콜론으로 구분하고 JSON/JSONL에서는 배열입니다. 파일 단위 진단은 로그에도 표시됩니다. |

JSON과 JSONL에는 같은 필드가 있으며, 숫자는 타입이 지정되고, 알 수 없는 값은 `null`, `Problems`는 목록입니다. 모든 형식에서 명령 텍스트는 저장된 그대로 유지됩니다.

`FileLastWriteTime`은 기본적으로 UTC의 ISO 8601 형식(예: `2026-10-20T08:46:38.5Z`)을 사용합니다. 표시 오프셋을 바꾸는 `-Z, --timezone`은 `auto-parse`만 지원합니다. 독립 실행 명령인 `parse-file powershell-history`에는 시간대 옵션이 없습니다.

## 자동 분석 { #automatic-parsing }

`auto-parse`는 `ShellHistory/PowerShellHistory.csv`(`-t json` 또는 `-t jsonl`을 사용하면 `PowerShellHistory.json` 또는 `PowerShellHistory.jsonl`)와 `ShellHistory/PowerShellHistory-logs.txt`를 생성합니다. VHDX 및 E01 이미지에서는 기록 파일을 복사하며, `FileLastWriteTime`은 이미지에 기록된 시각입니다. 결합된 `-j` 타임라인에는 모든 행이 `Artifact` 식별자 `powershell-history`와 함께 포함됩니다. 이 줄들에는 시간이 없으므로 [`super-timeline`](../commands/super-timeline.md)은 이 줄들로부터 이벤트를 만들지 않고 시간이 없는 레코드로 셉니다.

## 조사 전략 { #investigation-strategy }

1. `Windows\System32\config\systemprofile`과 `Windows\ServiceProfiles` 아래의 프로필을 포함해 모든 프로필의 기록을 읽으세요. 이 프로필에는 해당 계정으로 실행된 셸에서 입력한 명령이 들어 있습니다.
2. 여러 줄 항목은 함께 읽고, 다운로드, 인코딩된 명령, 보안 설정 변경, 계정 및 자격 증명 관련 명령, 원격 연결을 찾으세요.
3. PowerShell 이벤트 로그(스크립트 블록 로깅, 이벤트 4104), 프로세스 생성 이벤트, `powershell.exe`와 `pwsh.exe`의 Prefetch 파일과 대조하세요.
4. `FileLastWriteTime`을 기록 파일의 `$MFT` 및 USN 저널 시간, 그리고 로그온과 비교해 마지막 줄들이 추가된 시기의 범위를 좁히세요.

## 추가 정보 { #further-details }

- [Microsoft — about_PSReadLine: 명령 기록](https://learn.microsoft.com/en-us/powershell/module/psreadline/about/about_psreadline)
- [Microsoft — Set-PSReadLineOption: HistorySavePath 및 HistorySaveStyle](https://learn.microsoft.com/en-us/powershell/module/psreadline/set-psreadlineoption)
- [Microsoft — about_Logging_Windows: 스크립트 블록 로깅](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [PSReadLine — 기록 파일을 저장하고 읽는 방식](https://github.com/PowerShell/PSReadLine/blob/master/PSReadLine/History.cs)
