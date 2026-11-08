# Outlook { #outlook }

## Outlook의 파일이란 { #what-outlooks-files-are }

Windows용 Outlook은 메일, 연락처, 일정, 작업을 자체 파일에 저장합니다.

- **OST**(`.ost`): 사서함의 오프라인 사본입니다. Outlook은 Exchange 및 Microsoft 365 계정에 대해 이 파일을
  유지하며, Outlook 2013부터는 IMAP 계정에 대해서도 유지합니다. Outlook 2013 이상의 OST는 4KiB 페이지와
  압축된 블록을 사용합니다.
- **PST**(`.pst`): 개인 저장소입니다. POP3 계정, 보관 파일(`archive.pst`), 내보낸 파일, 그리고 사용자가 만든
  저장소가 여기에 해당합니다. ANSI PST는 Outlook 97~2002의 것이고, Unicode PST는 Outlook 2003 이상의
  것입니다.
- **MSG**(`.msg`): 파일로 저장된 하나의 항목입니다. 항목을 Outlook 밖으로 끌어 놓거나, **다른 이름으로 저장**으로
  저장하거나, 다른 프로그램이 내보낼 때 만들어집니다. 첨부된 메시지도 MSG 파일입니다.
- **OFT**(`.oft`): Outlook 서식 파일입니다. 새 항목을 만드는 데 쓰이는 MSG 파일입니다.

각 저장소에는 폴더가 있고, 각 폴더에는 그 항목이 있습니다. 전자 메일, 모임 요청과 그 응답, 연락처와 배포 목록,
일정 항목, 작업, 메모, 업무 일지 항목, 보고서입니다. 각 폴더에는 숨겨진 항목, 즉 폴더 연결 정보(FAI)도 있습니다.
보기, 규칙, 양식, 그리고 Outlook의 설정(Exchange 사서함의 자동 완성 목록 같은 `IPM.Configuration.*` 항목)입니다.
항목의 첨부 파일은 파일, 파일에 대한 참조, OLE 개체, 또는 다른 항목(첨부 파일로 전달된 메시지, 되풀이 약속의
예외)일 수 있으며, 이런 항목에도 각자의 첨부 파일이 있을 수 있습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 수집 시점에 저장소 또는 파일이 해당 항목을 기록된
보낸 사람, 받는 사람, 시각, 첨부 파일과 함께 보관하고 있었습니다. 사용자가 그 항목을 읽었거나, 작성했거나,
보냈다는 사실도, 보낸 사람이 항목에 나타난 그 사람이라는 사실도 알려 주지 않습니다.

## 위치와 수집 { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

PST는 어디에나 있을 수 있습니다. 사용자는 보관 파일을 다른 폴더, 드라이브, 파일 공유로 옮깁니다. Outlook은
각 프로필의 저장소 경로를 사용자의 레지스트리에 보관합니다. 사용자가 Outlook에서 연 첨부 파일은
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` 아래의 폴더(Windows 8.1 이전에는
`Temporary Internet Files\Content.Outlook`)에 복사되며, 그곳에 남아 있을 수 있습니다.

Outlook은 실행 중에 저장소를 열어 두며, OST는 수십 GB에 이를 수 있습니다. Outlook을 종료한 상태에서
수집하거나, 디스크 이미지 또는 볼륨 섀도 복사본에서 수집하십시오. Juntei는 각 저장소를 스트림으로 읽으며,
저장소 전체를 메모리에 올려 두지 않습니다.

## 분석 시 주의 사항 { #analyst-cautions }

- Juntei는 관대하게 읽습니다. 손상된 부분을 건너뛰어 읽을 수 있는 것은 읽고, 나머지는 문제로 보고합니다. 문제는
  화면에 출력되고(파일마다 처음 10개), 요약과 Files 테이블에 집계되며, 해당 항목과 첨부 파일의 `Problems`
  필드에 나열됩니다. 전혀 읽을 수 없는 항목도 Items 테이블에 `ItemID`와 그 이유가 담긴 행이 있습니다.
- 시각은 UTC이며 100나노초 단위입니다. `SubmitTime`은 보낸 사람의 클라이언트가 메시지를 제출한 시각으로, 그
  컴퓨터의 시계를 따릅니다. `DeliveryTime`은 메시지가 이 사서함에 도착한 시각입니다. `CreationTime`은 항목의
  이 사본이 만들어진 시각, 즉 사서함에 배달되거나 사서함에서 만들어진 시각, OST로 동기화된 시각, PST로
  복사되거나 가져와진 시각, 또는 MSG 파일로 저장된 시각입니다. 메시지가 작성된 시각이 아닙니다.
  `LastModifiedTime`은 항목이 저장소에서 마지막으로 변경된 시각입니다.
- 작업의 `StartTime`과 `DueTime`은 사용자의 표준 시간대 기준 날짜이며, Outlook은 이를 자정으로 저장합니다.
  Juntei는 이를 오프셋 없이 기록하며(`2017-02-19T00:00:00`), `-Z`도 그대로 둡니다.
- Outlook은 작업의 기한이나 미리 알림 시각 같은 일부 날짜에서 **없음**을 나타내기 위해 4501년 1월 1일 자정을
  기록합니다. Juntei는 이를 저장된 그대로 `4501-01-01T00:00:00Z`로 기록하며, 작업의 `StartTime`과 `DueTime`에서는
  `Z`가 붙지 않습니다.
- `Read`는 저장소가 수집된 시점의 읽음 플래그입니다. 누군가 또는 규칙이 항목을 읽음으로 표시했다는 뜻이며,
  누가 언제 그렇게 했는지는 알려 주지 않습니다.
- 보낸 사람과 받는 사람의 이름 및 주소는 보내는 프로그램과 메일 서버가 기록한 것이며, 위조될 수 있습니다. `EX`
  주소는 Exchange 고유 이름(`/o=…/ou=…/cn=Recipients/cn=…`)입니다. 항목이 SMTP 주소를 추가로 저장하고 있으면
  그 주소는 `SenderSMTP-Address`와 `SMTP-Address`에 들어갑니다. 메시지가 어디에서 왔는지 확인하려면 그 인터넷
  헤더, 즉 Properties 테이블의 `0x007D001F` 또는 `0x007D001E`(`PidTagTransportMessageHeaders`)를 읽으십시오.
- Outlook에서 삭제한 항목은 비울 때까지 **지운 편지함**(Deleted Items)에 있으며, 다른 항목과 똑같이 읽힙니다.
  더 이상 어느 폴더의 테이블에도 없는 항목은 저장소가 그 공간을 재사용할 때까지 남아 있을 수 있지만, 아직 읽지
  않습니다.
- `ItemID`는 PST 또는 OST에서 메시지의 노드 ID입니다. 저장소의 다른 사본에서는 같은 항목이 다른 ID를 가질 수
  있습니다. 포함된 항목의 ID는 부모의 ID와 첨부 파일의 인덱스(`2097348/0`, `0/0/1`)이며, `.msg` 또는 `.oft`
  파일의 항목은 `0`입니다.
- `User`는 경로에서만 가져오며(`Users` 다음 폴더), 파일이 다른 곳으로 복사되면 사라집니다. PST는 어느 사용자의
  것이든 될 수 있습니다.
- Unicode가 아닌 텍스트는 항목의 코드 페이지(항목 자체의 코드 페이지, 로캘의 코드 페이지 또는 인터넷 코드
  페이지)로 읽습니다. `--code-page`는 코드 페이지를 알려 주지 않는 항목의 코드 페이지를 지정합니다. 그 항목을
  기록한 시스템의 ANSI 코드 페이지로, 일본어 시스템이면 `932`입니다.
- PST의 암호는 파일을 암호화하지 않습니다. Juntei는 암호로 보호된 PST도 다른 PST와 똑같이 읽습니다. 저장소의
  `Encryption`(`Permute`, 즉 Outlook의 "압축 가능 암호화", 또는 `Cyclic`)은 블록을 난독화할 뿐입니다. S/MIME으로
  암호화된 메시지는 본문과 첨부 파일을 암호화된 첨부 파일(`smime.p7m`)에 담고 있으며, 이는 복호화되지 않습니다.
- 첨부 파일은 악성코드인 경우가 많습니다. `--extract-attachments`는 그 데이터를 SHA-256으로 이름을 붙인 파일에
  실행 권한 없이 기록합니다. 절대 실행하지 말고, 격리된 분석 시스템에 보관하십시오.

### 아직 읽지 않는 것 { #not-read-yet }

오프라인 주소록(OAB), 자동 완성 스트림(`Stream_Autocomplete_*.dat`)과 `.nk2` 파일, 보내기/받기 설정(`.srs`),
Outlook의 설정 캐시(RoamCache, Autodiscover), Outlook의 SQLite 데이터베이스, 그리고 더 이상 어느 폴더의
테이블에도 없는 메시지.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pst, *.ost, *.msg and *.oft
  -f, --file <FILE>      Parse one PST, OST, MSG or OFT file, whatever its name
      --code-page <CP>   Code page of items that don't give theirs (default 1252)

Output:
  -o, --output <PATH>              Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>         csv, json, or jsonl
      --properties                 Add the Properties table: every property as stored
      --extract-attachments <DIR>  Write each attachment's data once, named by its SHA-256
  -C, --clobber                    Overwrite files when saving or extracting
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl \
  | jq 'select(.Table == "Items" and .Hidden == false) | {DeliveryTime, SenderAddress, Subject}'
juntei parse-file outlook -f suspicious.msg -o msg-csv --properties --extract-attachments isolated/
```

`-d`는 대소문자를 구분하지 않고 `.pst`, `.ost`, `.msg`, `.oft`로 끝나는 파일을 찾습니다. 파일을 어떤 방식으로
지정하든, 파일의 종류는 첫 바이트로 판별합니다. PST 또는 OST는 `!BDN`, MSG 또는 OFT는 복합 파일 서명입니다. 그
밖의 파일은 파싱할 수 없었던 파일로 보고됩니다. 읽을 수 있었던 파일이 하나도 없으면 종료 상태는 1입니다.

`--extract-attachments`는 각 첨부 파일의 데이터를 한 번만 기록합니다. 파일 이름은 `<SHA-256>`이며, 첨부 파일의
확장자가 1~10자의 ASCII 영문자와 숫자로 되어 있으면 그 확장자가 붙습니다(`0c7d…1311.jpg`). 항목에서 온 이름은
경로에 전혀 쓰이지 않습니다. `-C`를 지정하지 않으면 디렉터리가 비어 있어야 합니다. `-C`를 지정하면 이름이 같고
내용이 다른 파일은 교체됩니다.

## 출력 테이블 { #output-tables }

CSV는 여덟 개의 파일을 생성하며, `--properties`를 사용하면 아홉 개가 됩니다. 행이 없으면 헤더만 있는 파일이
됩니다. JSON 및 JSONL 행에는 `SourceFile` 다음에 `Table`(`Files`, `Folders`, `Items`, `Recipients`,
`Attachments`, `Contacts`, `Calendar`, `Tasks` 또는 `Properties`)이 포함됩니다. 항목의 행 다음에는 그 항목에
속한 행이 이어지고, 그다음에 포함된 항목의 행이 이어집니다. 파일의 Files 행은 그 파일의 나머지 모든 행 뒤에
옵니다.

| CSV 파일 | 내용 |
|---|---|
| `Outlook-Files.csv` | 읽은 파일마다 한 행: 저장소 또는 파일과 그 안에 들어 있는 것. |
| `Outlook-Folders.csv` | PST와 OST의 폴더. |
| `Outlook-Items.csv` | 모든 항목: 폴더의 메시지, 폴더의 숨겨진 항목, `.msg` 및 `.oft` 파일의 항목, 그리고 이들의 첨부 파일에 포함된 항목. |
| `Outlook-Recipients.csv` | 항목의 받는 사람. |
| `Outlook-Attachments.csv` | 항목의 첨부 파일. |
| `Outlook-Contacts.csv` | 연락처와 배포 목록의 필드. |
| `Outlook-Calendar.csv` | 일정 항목과 모임 메시지의 필드. |
| `Outlook-Tasks.csv` | 작업과 작업 요청의 필드. |
| `Outlook-Properties.csv` | `--properties` 사용 시: 저장소, 폴더, 항목, 받는 사람, 첨부 파일의 모든 속성(저장된 그대로). |

## 필드 { #fields }

모든 테이블은 다음 필드로 시작합니다.

| 필드 | 해석 |
|---|---|
| `SourceFile` | 행을 읽어 온 파일. |
| `User` | 경로상의 프로필 소유자인 Windows 사용자(`Users` 다음 폴더). |

### 파일 { #files }

| 필드 | 해석 |
|---|---|
| `FileType` | `PST`, `OST`, `MSG`, `OFT`, 또는 `CFB`(항목으로 읽었지만 클래스 ID가 메시지의 것도 서식 파일의 것도 아닌 복합 파일. Outlook 이외의 많은 프로그램은 클래스 ID를 비워 둡니다). |
| `Format` | 저장소의 레이아웃: `ANSI`(Outlook 97~2002), `Unicode`(Outlook 2003 이상) 또는 `Unicode 4K`(Outlook 2013 이상의 OST). MSG 또는 OFT 파일은 복합 파일 버전(`CFB v3`, `CFB v4`). |
| `Version`, `ClientVersion` | 저장소 헤더의 `wVer`와 `wVerClient`. |
| `Encryption` | 저장소 블록의 인코딩 방식: `None`, `Permute` 또는 `Cyclic`. |
| `DisplayName` | 저장소의 이름(`Personal Folders`, 전자 메일 주소). |
| `RecordKey` | 저장소의 식별자(16진수). 저장소에 있는 항목의 엔트리 ID에 이 값이 들어 있습니다. |
| `Folders` | 저장소의 폴더 수. |
| `Items` | 폴더에 있는 메시지 수. MSG 또는 OFT 파일은 1. |
| `HiddenItems` | 폴더의 숨겨진 항목(폴더 연결 정보) 수. |
| `EmbeddedItems` | 모든 수준에서 첨부 파일에 포함된 항목 수. |
| `Attachments` | 이 모든 항목의 첨부 파일 수. |
| `Problems` | 파일에서 발견된 문제 수. |

### 폴더 { #folders }

| 필드 | 해석 |
|---|---|
| `FolderID`, `ParentID` | 폴더의 노드 ID와 부모 폴더의 노드 ID(루트 폴더는 비어 있음). |
| `FolderPath` | 루트부터 아래로 이어지는 폴더 이름: `/Top of Personal Folders/Inbox`, `/Root - Mailbox/IPM_SUBTREE/Sent Items`. |
| `Name` | 폴더의 이름. |
| `ContainerClass` | 폴더에 들어 있는 것의 종류: `IPF.Note`(메일), `IPF.Appointment`, `IPF.Contact`, `IPF.Task`, `IPF.Imap` 등. |
| `ContentCount`, `UnreadCount` | 폴더 자체가 밝히는 메시지 수와 읽지 않은 메시지 수. |
| `Messages`, `HiddenItems`, `Subfolders` | 폴더의 내용 테이블, 연결된 내용 테이블, 계층 테이블의 행 수. Juntei가 읽는 것은 이 행들입니다. `ContentCount`와 차이가 있으면 살펴볼 가치가 있습니다. |
| `CreationTime`, `LastModifiedTime` | 폴더가 만들어진 시각과 마지막으로 변경된 시각(UTC). 저장소가 보관하고 있을 때만 있습니다. |
| `Problem` | 폴더를 열 수 없었던 이유. 이 경우 하위 폴더는 읽지 않습니다. |

### 항목 { #items }

| 필드 | 해석 |
|---|---|
| `FolderPath` | 항목의 폴더. MSG 또는 OFT 파일에서는 비어 있습니다. |
| `ItemID` | 항목의 ID: PST 또는 OST에서는 노드 ID, MSG 또는 OFT 파일에서는 `0`, 포함된 항목은 부모의 ID와 첨부 파일의 인덱스(`2097348/0`). |
| `Hidden` | 항목이 폴더 연결 정보, 즉 숨겨진 항목인지 여부. |
| `EmbeddedIn` | 이 항목이 첨부된 항목의 ID. 그 밖의 항목은 비어 있습니다. |
| `ItemType` | 메시지 클래스가 나타내는 항목의 종류: `Mail`, `MeetingMessage`, `Appointment`, `Contact`, `DistributionList`, `Task`, `TaskRequest`, `StickyNote`, `Journal`, `Post`, `Report`, `Configuration` 또는 `Other`. |
| `MessageClass` | 저장된 그대로의 메시지 클래스(`IPM.Note`, `IPM.Note.SMIME`, `IPM.Schedule.Meeting.Request`, `REPORT.IPM.Note.NDR` 등). |
| `SubmitTime` | 보낸 사람의 클라이언트가 메시지를 제출한 시각(UTC, 보낸 사람의 시계 기준). |
| `DeliveryTime` | 이 사서함에 배달된 시각(UTC). |
| `CreationTime`, `LastModifiedTime` | 이 사본이 저장소 또는 파일에 만들어진 시각과 마지막으로 변경된 시각(UTC). |
| `Subject` | 접두사(`RE: `)를 포함한 제목. |
| `SenderName`, `SenderAddressType`, `SenderAddress` | 보낸 사람: 이름, 주소 유형(`SMTP`, `EX`), 그리고 그 유형의 주소. |
| `SenderSMTP-Address` | 보낸 사람의 SMTP 주소. 다른 유형의 주소와 함께 추가로 저장되어 있을 때만 있습니다. |
| `SentRepresentingName`, `SentRepresentingAddress` | 누구를 대신해 보냈는지(대리인이 관리자를 대신해 보내는 경우, 공유 사서함). 주소는 저장되어 있으면 SMTP 주소, 그렇지 않으면 해당 유형의 주소입니다. |
| `To`, `Cc`, `Bcc` | 항목에 표시되는 그대로의 받는 사람 이름(주소는 Recipients 참조). `Bcc`는 보낸 사람의 사본에만 있습니다. |
| `InternetMessageID`, `InReplyTo` | 메시지의 `Message-ID`와, 이 메시지가 회신하는 원래 메시지의 ID. |
| `ConversationTopic` | 접두사를 뺀 제목. 같은 대화의 메시지가 공유합니다. |
| `Importance`, `Sensitivity` | 중요도는 `Low`, `Normal` 또는 `High`. 민감도는 `Normal`, `Personal`, `Private` 또는 `Confidential`. |
| `Read` | 항목이 읽음으로 표시되어 있는지 여부. |
| `Flags` | 메시지 플래그(`PidTagMessageFlags`, 16진수): `0x1` 읽음, `0x2` 수정되지 않음, `0x4` 제출됨, `0x8` 보내지 않음(임시 보관), `0x10` 첨부 파일 있음, `0x20` 나에게서, `0x40` 숨김, `0x80` 다시 보냄, `0x400` 읽은 적 있음, `0x2000` 인터넷에서. |
| `Size` | 첨부 파일을 포함한, 저장소에서의 항목 크기. |
| `Recipients`, `Attachments` | 받는 사람 수와 첨부 파일 수. |
| `Categories` | 항목의 범주. `;`로 구분합니다. |
| `BodyPreview` | 일반 텍스트 본문의 앞부분. 공백은 하나로 줄이며 최대 256자입니다. HTML 또는 RTF 본문만 있으면 비어 있습니다. |
| `Problems` | 항목에서 읽을 수 없었던 것(최대 10개, 그 이상이면 나머지 개수). |

### 받는 사람 { #recipients }

| 필드 | 해석 |
|---|---|
| `ItemID` | 항목의 ID. |
| `Index` | 항목의 받는 사람 테이블에서 받는 사람의 행 번호(0부터). |
| `RecipientType` | `To`, `Cc`, `Bcc`, `Originator`, 또는 저장된 그대로의 값(16진수). |
| `Name`, `AddressType`, `Address` | 받는 사람의 이름, 주소 유형, 주소. |
| `SMTP-Address` | SMTP 주소. 다른 유형의 주소와 함께 추가로 저장되어 있을 때만 있습니다. |

### 첨부 파일 { #attachments }

| 필드 | 해석 |
|---|---|
| `ItemID`, `Index` | 항목의 ID와, 항목의 첨부 파일 테이블에서 첨부 파일의 행 번호(0부터). |
| `Method` | `ByValue`(파일), `ByReference`, `ByReferenceResolve`, `ByReferenceOnly`(파일의 경로), `EmbeddedMessage`(항목), `OLE`(OLE 개체), `ByWebReference`, 또는 숫자. |
| `Filename` | 첨부 파일의 이름: 긴 파일 이름, 짧은 파일 이름 또는 표시 이름. |
| `Extension` | 저장된 그대로의 확장자(`.pdf`). 없으면 이름의 확장자. |
| `MimeType`, `ContentID` | MIME 유형과, HTML 본문이 첨부 파일을 참조할 때 쓰는 콘텐츠 ID. |
| `Inline` | 첨부 파일이 목록에 나열되지 않고 본문에 표시되는지 여부(HTML 본문의 이미지). |
| `Hidden` | Outlook이 첨부 파일 목록에서 숨기는지 여부. |
| `Size` | 저장소가 알려 주는 크기(속성 포함). |
| `DataSize`, `SHA256` | 데이터의 크기와 SHA-256. 데이터는 파일이거나, PST 또는 OST에서는 OLE 개체의 복합 파일입니다. 포함된 항목과 참조 방식의 첨부 파일은 비어 있습니다. |
| `Pathname` | 참조 방식 첨부 파일의 경로. |
| `CreationTime`, `LastModifiedTime` | 첨부 파일의 시각(UTC). 대개 원본 파일의 시각입니다. |
| `EmbeddedSubject` | 포함된 항목의 제목. 그 항목 자체의 행에는 `ItemID` `<ItemID>/<Index>`가 있습니다. |
| `ExtractedFile` | `--extract-attachments` 사용 시, 데이터가 기록된 그 디렉터리 안의 파일. |
| `Problems` | 첨부 파일에서 읽을 수 없었거나 기록할 수 없었던 것. |

### 연락처 { #contacts }

| 필드 | 해석 |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | Items 테이블과 같습니다. `Contact` 또는 `DistributionList`. |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | 연락처의 이름. |
| `FileUnder` | 연락처가 분류되어 정리되는 이름. |
| `Company`, `Title`, `Department` | 연락처의 근무처: 회사, 직함, 부서. |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | 연락처의 전자 메일 주소 세 개와 각각의 표시 이름. |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | 연락처의 전화번호. |
| `InstantMessaging` | 연락처의 메신저 주소. |
| `CreationTime`, `LastModifiedTime` | Items 테이블과 같습니다. |

### 일정 { #calendar }

| 필드 | 해석 |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Items 테이블과 같습니다. `Appointment` 또는 `MeetingMessage`. |
| `StartTime`, `EndTime` | 약속의 시작 시각과 종료 시각(UTC). 되풀이 약속은 첫 번째 일정의 시각. |
| `DurationMinutes` | 약속의 길이(분). |
| `Location` | 약속 장소. |
| `AllDay`, `Recurring` | 종일 이벤트인지 여부와 되풀이되는지 여부(그 예외는 포함된 항목입니다). |
| `BusyStatus` | `Free`, `Tentative`, `Busy`, `OutOfOffice` 또는 `WorkingElsewhere`. |
| `ResponseStatus` | 사용자의 응답: `None`, `Organized`, `Tentative`, `Accepted`, `Declined` 또는 `NotResponded`. |
| `Organizer` | 항목을 보낸 사람의 이름. |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | 항목에 나열된 그대로의 참석자. |
| `GlobalObjectID` | 사서함 간에 모임을 식별하는 ID(16진수). 이끌이의 일정과 참석자의 일정에 있는 같은 모임이 공유합니다. |
| `ReminderSet`, `ReminderTime` | 미리 알림이 설정되어 있는지 여부와, 미리 알림의 대상 시각(UTC). |
| `CreationTime`, `LastModifiedTime` | Items 테이블과 같습니다. |

### 작업 { #tasks }

| 필드 | 해석 |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | Items 테이블과 같습니다. `Task` 또는 `TaskRequest`. |
| `Status` | `NotStarted`, `InProgress`, `Complete`, `Waiting` 또는 `Deferred`. |
| `PercentComplete` | 진행률(0~100). |
| `StartTime`, `DueTime` | 시작일과 기한: 사용자의 표준 시간대 기준 날짜이며, 오프셋이 없습니다. |
| `CompletedTime` | 완료된 시각(UTC). |
| `Complete` | 완료되었는지 여부. |
| `Owner`, `Assigner` | 작업의 소유자와, 작업을 할당한 사람. |
| `ReminderSet`, `ReminderTime` | Calendar 테이블과 같습니다. |
| `CreationTime`, `LastModifiedTime` | Items 테이블과 같습니다. |

### 속성 { #properties }

| 필드 | 해석 |
|---|---|
| `ItemID` | 항목의 ID. 저장소 자체의 속성은 `store`, 폴더의 속성은 `folder:<FolderID>`. |
| `Scope` | 속성의 소유자: `Store`, `Folder`, `Item`, `Recipient` 또는 `Attachment`. |
| `Index` | 받는 사람 또는 첨부 파일의 인덱스. 그 밖에는 비어 있습니다. |
| `PropertyTag` | 속성의 ID와 유형. `0x0037001F` 형식입니다(`PidTagSubject`, 문자열). |
| `PropertyName` | 명명된 속성(ID가 `0x8000` 이상)의 속성 집합과 이름: `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords`, 또는 잘 알려진 집합이 아니면 집합의 GUID. |
| `Type` | 속성 유형: `PtypString`, `PtypInteger32`, `PtypTime`, `PtypBinary`, `PtypMultipleString` 등. |
| `Value` | 값: 문자열은 텍스트, 시각은 ISO 8601 UTC(`-Z`는 변환하지 않음), 이진 값은 16진수(처음 4,096바이트, 그 뒤에 `…(N bytes)`), 다중 값 속성은 값의 목록(CSV에서는 `, `로 구분), 포함된 항목이나 첨부 파일의 데이터는 `(object)`, 디코딩하지 않는 유형의 값은 설명이 붙은 16진수 바이트, 손상된 값은 복구할 수 있었던 값 뒤에 16진수 바이트를 포함한 설명(아무것도 복구하지 못한 경우에는 바이트와 설명만). |

## 자동 분석 { #automatic-parsing }

`auto-parse`는 테이블을 새 `Communication` 범주에 `Outlook-Files.csv`, `Outlook-Folders.csv`,
`Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`, `Outlook-Contacts.csv`,
`Outlook-Calendar.csv`, `Outlook-Tasks.csv`로 기록하며(`-t json` 또는 `-t jsonl`을 사용하면 하나의
`Outlook.json` 또는 `Outlook.jsonl` 파일), `Communication/Outlook-logs.txt`도 함께 생성합니다. 파서에
`--code-page`를 전달하지만, Properties 테이블을 추가하거나 첨부 파일을 추출하지는 않습니다. 이것들이 필요하면
`parse-file outlook`을 실행하십시오. 결합된 `-j` 타임라인에는 모든 행이 `Comp`와 `Artifact` 식별자 `outlook`과
함께 포함됩니다. `-Z`는 작업의 `StartTime`과 `DueTime`을 제외하고 `Time`으로 끝나는 모든 필드를 변환합니다.

## 추가 정보 { #further-details }

- [Microsoft — \[MS-PST\]: Outlook 개인 폴더(.pst) 파일 형식](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook 항목(.msg) 파일 형식](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: 메시지 및 첨부 파일 개체 프로토콜](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server 프로토콜 마스터 속성 목록](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Personal Folder File(PFF) 형식](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
