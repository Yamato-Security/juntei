# Статті та джерела { #articles-and-references }

На цій сторінці зібрано якісні відправні точки для читачів, які хочуть вийти за межі посібників з
артефактів. Розділи **Додаткові відомості** на сторінці кожного артефакту містять найбільш цільові посилання на низькорівневі матеріали.

## Вступ до DFIR для початківців { #beginner-dfir-orientation }

- [Плакат SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/) — наочна карта поширених артефактів Windows та їх розташувань.
- [Плакат SANS Hunt Evil](https://www.sans.org/posters/hunt-evil/) — практичні точки переходу для пошуку загроз і нагадування щодо розслідувань у Windows.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — загальнодоступні зразки артефактів для навчання та перевірки парсерів.
- [Довідник артефактів Velociraptor](https://docs.velociraptor.app/artifact_references/) — орієнтовані на збирання описи та реалізації VQL для багатьох артефактів Windows.
- [Документація Plaso](https://plaso.readthedocs.io/) — концепції часових шкал і парсери для аналізу за кількома артефактами.

## Артефакти файлової системи { #file-system-artifacts }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: журнали змін (Change Journals)](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: дескриптори безпеки](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) і [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — дескриптори безпеки, ACE та умовні вирази.
- [Документація формату libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Парсер mft від Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — парсер NTFS з вирізанням (carving) записів INDX і залишкового простору індексів.
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — вирізає імена файлів із залишкового простору `$I30`.
- [LogFileParser](https://github.com/jschicht/LogFileParser) — докладне дослідження та розбір журналу транзакцій NTFS.
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — зіставляє історію `$LogFile`, `$MFT` і USN.

## Артефакти виконання та інвентаризації { #execution-and-inventory-artifacts }

- [amcache-forensic від Security Ronin](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — дослідження формату Prefetch і бібліотека.
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)

## Артефакти активності користувача { #user-activity-artifacts }

- [Microsoft Open Specifications: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## Реєстр і закріплення { #registry-and-persistence }

- [Microsoft: типи значень реєстру](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Специфікація формату реєстру Windows від Maxim Suhanov](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) і [libregf](https://github.com/libyal/libregf) — довідкові матеріали щодо низькорівневого розбору та відновлення кущів реєстру.
- [Microsoft: дерево реєстру служб](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: схема Task Scheduler](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender і поводження зі шкідливим програмним забезпеченням { #defender-and-malware-handling }

- [Microsoft: відновлення файлів із карантину в Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [Плагін Defender для dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — окрема дослідницька реалізація для видобування.
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/) — ресурси з поводження зі шкідливим ПЗ та його аналізу.

## Проєкти парсерів Juntei { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`, `$I30`, журнал USN, `$LogFile`, `$Boot`, `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "Зовнішні матеріали змінюються"
    Перевіряйте твердження з урахуванням відповідної версії Windows і зібраних вами даних. Корисна стаття може
    описувати конкретне покоління операційної системи, версію парсера чи експеримент, а не
    універсальне правило.
