# 아티팩트 간 조사 절차 { #cross-artifact-workflows }

어떤 아티팩트도 단독으로 중요한 결론을 뒷받침해서는 안 됩니다. 이 절차들은 단서를 방어 가능한 증거의 연결
고리로 바꾸는 방법을 보여 줍니다.

## 의심스러운 실행 파일의 실행 { #suspicious-executable-execution }

1. **Prefetch:** 보존된 실행 시간, 경로/해시 컨텍스트, 실행 횟수 및 로드된 파일을 확인합니다.
2. **`$MFT`:** 경로, 크기, 타임스탬프, ADS, 삭제 상태 및 MFT 참조를 찾습니다.
3. **USN 저널:** 생성, 쓰기, 이름 변경 및 삭제를 재구성합니다.
4. **Amcache:** SHA-1, PE 메타데이터, 버전 및 과거 경로를 추가합니다.
5. **ShimCache:** 호환성 캐시 경로/순서와 지원되는 실행 플래그를 추가합니다.
6. **SRUM:** 같은 실행 파일/사용자의 리소스/네트워크 사용을 찾습니다.
7. **Defender:** 격리 시간, 원래 경로, 해시 및 다운로드 출처를 확인합니다.
8. **이벤트 로그/EDR:** 프로세스의 부모, 명령줄, 계정, 네트워크 및 보안 컨텍스트를 식별합니다.

결론에서는 파일의 존재, 실행, 그리고 특정 인물이나 침입으로의 귀속을 구별해야 합니다.

## 랜섬웨어 파일 활동 { #ransomware-file-activity }

1. `$MFT`에서 암호화된 확장자와 피해 경로를 식별합니다.
2. USN 사유와 순서를 사용하여 대량 쓰기/이름 변경/삭제를 재구성합니다.
3. Prefetch를 사용하여 암호화 프로그램 후보와 실행 시간을 식별합니다.
4. SRUM을 사용하여 후보 프로세스의 CPU/디스크/네트워크 급증을 확인합니다.
5. 배포/지속성 확인을 위해 서비스와 TaskCache를 점검합니다.
6. 탐지/치료 확인을 위해 Defender 격리 데이터와 이벤트 로그를 점검합니다.

MFT/USN은 실행 아티팩트가 남아 있지 않더라도 파일 시스템에 대한 영향을 증명할 수 있습니다. 그러나 일반적으로
프로세스 텔레메트리나 상관 분석 없이는 책임이 있는 프로세스를 식별하지 못합니다.

## 악성 문서 / 바로 가기를 통한 초기 액세스 { #malicious-document-shortcut-initial-access }

1. RecentDocs와 Jump Lists로 사용자/애플리케이션 상호 작용과 MRU 컨텍스트를 식별합니다.
2. 일치하는 Recent 폴더의 LNK를 파싱하여 대상, 볼륨/공유, MFT 참조 및 인수를 확인합니다.
3. `$MFT` 상주 ADS 또는 Defender 콘텐츠의 Zone.Identifier를 사용하여 출처 URL/영역을 확인합니다.
4. Prefetch/이벤트 로그를 사용하여 문서 애플리케이션과 하위 프로세스를 확인합니다.
5. USN/MFT를 사용하여 추출된 페이로드의 생성과 삭제를 추적합니다.

## 서비스를 통한 지속성 { #service-persistence }

1. 서비스 이름, `ImagePath`, `ServiceDll`, 계정, 시작 유형 및 실패 시 동작을 디코딩합니다.
2. 환경 변수와 따옴표가 있는/없는 경로의 동작을 확인합니다.
3. `$MFT`와 USN 기록에서 실행 파일/DLL을 찾습니다.
4. 실제 서비스/프로세스 시작 여부를 Prefetch와 이벤트 로그로 확인합니다.
5. 이전 구성을 확인하기 위해 현재가 아닌 제어 집합과 복구된 레지스트리 레코드를 비교합니다.

## 예약된 작업을 통한 지속성 { #scheduled-task-persistence }

1. TaskCache의 동작, 주체/컨텍스트, 등록/런타임 메타데이터 및 숨김 상태를 디코딩합니다.
2. `C:\Windows\System32\Tasks` XML과 비교합니다.
3. 작업 이름/GUID를 Security 및 Task Scheduler Operational 이벤트와 상관 분석합니다.
4. 실행 파일/스크립트를 찾아 해시를 계산하고, Prefetch와 파일 시스템 기록을 검사합니다.
5. 작업이 제거된 경우 삭제된 TaskCache 레코드를 복구합니다.

## 데이터 유출 의심 { #suspected-exfiltration }

1. SRUM을 사용하여 앱/사용자/프로필별로 비정상적인 전송 바이트와 측정 기간을 식별합니다.
2. Prefetch, Amcache 및 `$MFT`로 실행 파일의 식별 정보를 확인합니다.
3. Jump Lists, LNK, RecentDocs 및 이벤트/EDR 데이터를 통해 액세스되거나 열린 파일을 식별합니다.
4. 대상/프로토콜은 프록시, 방화벽, DNS, 패킷 또는 EDR 텔레메트리로 확인합니다.

SRUM의 데이터량은 유출 가설을 뒷받침하지만, 그것만으로 전송된 콘텐츠를 식별하거나 악의적인 의도를 증명하지는
못합니다.

## 보존할 가치가 있는 상관 분석 키 { #correlation-keys-worth-preserving }

| 키 | 아티팩트 |
|---|---|
| 전체 경로 / 파일 이름 | 거의 모든 아티팩트 |
| SHA-1 / SHA-256 | Amcache, Defender, 복구된 파일 |
| MFT 엔트리 + 시퀀스 | `$MFT`, Prefetch, LNK, Jump Lists, USN |
| USN | Amcache, `$MFT`, USN 저널 |
| 볼륨 일련번호 / droid | Prefetch, LNK, Jump Lists, MFT 객체 ID |
| SID / 사용자 이름 | SRUM, SAM, 레지스트리, 이벤트 로그 |
| 작업 GUID / 작업 이름 | TaskCache, 작업 XML, 이벤트 로그 |
| 서비스 이름 | 서비스, 이벤트 로그, 프로세스/드라이버 데이터 |
| Defender EntryId / ScanId | 격리 데이터, Defender 기록, 이벤트 로그 |
