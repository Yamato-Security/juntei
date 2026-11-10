# 문서 및 참고 자료 { #articles-and-references }

이 페이지는 아티팩트 가이드보다 더 깊이 알아보고자 하는 독자를 위해 양질의 출발점을 모아 둔 것입니다.
아티팩트별 **추가 정보** 섹션에는 가장 구체적인 저수준 링크가 정리되어 있습니다.

## 초보자를 위한 DFIR 입문 { #beginner-dfir-orientation }

- [SANS Windows Forensic Analysis 포스터](https://www.sans.org/posters/windows-forensic-analysis/) — 일반적인 Windows 아티팩트와 위치를 시각적으로 정리한 지도입니다.
- [SANS Hunt Evil 포스터](https://www.sans.org/posters/hunt-evil/) — 실용적인 헌팅 피벗과 Windows 조사 시 유의 사항을 정리한 자료입니다.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — 학습과 파서 검증을 위한 공개 아티팩트 샘플입니다.
- [Velociraptor 아티팩트 레퍼런스](https://docs.velociraptor.app/artifact_references/) — 다양한 Windows 아티팩트에 대한 수집 중심의 설명과 VQL 구현입니다.
- [Plaso 문서](https://plaso.readthedocs.io/) — 아티팩트 간 분석을 위한 타임라인 개념과 파서입니다.

## 파일 시스템 아티팩트 { #file-system-artifacts }

- [Microsoft: 마스터 파일 테이블](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: 변경 저널(Change Journals)](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: 보안 설명자](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) 및 [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — 보안 설명자, ACE 및 조건식에 대한 사양입니다.
- [libfsntfs 형식 문서](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Omer Ben-Amram의 mft 파서](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — INDX 레코드 및 인덱스 슬랙 카빙을 지원하는 NTFS 파서입니다.
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — `$I30` 슬랙에서 파일 이름을 카빙합니다.
- [LogFileParser](https://github.com/jschicht/LogFileParser) — NTFS 트랜잭션 로그에 대한 상세한 연구 및 분석 도구입니다.
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — `$LogFile`, `$MFT` 및 USN 기록을 상관 분석합니다.

## 실행 및 인벤토리 아티팩트 { #execution-and-inventory-artifacts }

- [Eric Zimmerman의 AmcacheParser](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — Prefetch 형식/라이브러리 작업입니다.
- [Eric Zimmerman의 PECmd](https://github.com/EricZimmerman/PECmd)
- [Mandiant: 포렌식 조사에서 Application Compatibility Cache 활용하기](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Eric Zimmerman의 AppCompatCacheParser](https://github.com/EricZimmerman/AppCompatCacheParser)

## 사용자 활동 아티팩트 { #user-activity-artifacts }

- [Microsoft Open Specifications: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Eric Zimmerman의 LECmd](https://github.com/EricZimmerman/LECmd)
- [Eric Zimmerman의 JLECmd](https://github.com/EricZimmerman/JLECmd)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Eric Zimmerman의 SrumECmd](https://github.com/EricZimmerman/Srum)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## 레지스트리와 지속성 { #registry-and-persistence }

- [Microsoft: 레지스트리 값 형식](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Maxim Suhanov의 Windows 레지스트리 형식 사양](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) 및 [libregf](https://github.com/libyal/libregf) — 저수준 하이브 분석 및 복구에 대한 참고 자료입니다.
- [Eric Zimmerman의 Registry Explorer / RECmd 문서](https://ericzimmerman.github.io/)
- [Microsoft: 서비스 레지스트리 트리](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: 작업 스케줄러 스키마](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender와 악성 코드 처리 { #defender-and-malware-handling }

- [Microsoft: Defender에서 격리된 파일 복원하기](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [dissect.target Defender 플러그인](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — 별도의 연구/추출 구현입니다.
- [FIRST 악성 코드 분석 기초](https://www.first.org/global/sigs/malware/analysis/) — 취급 및 분석 자료입니다.

## Juntei 파서 프로젝트 { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`, `$I30`, USN Journal, `$LogFile`, `$Boot`, `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "외부 자료는 변경될 수 있습니다"
    주장 내용은 해당 Windows 버전과 수집한 데이터를 기준으로 검증하십시오. 유용한 문서라도 보편적인
    규칙이 아니라 특정 운영 체제 세대, 파서 버전 또는 실험을 설명하는 것일 수 있습니다.
