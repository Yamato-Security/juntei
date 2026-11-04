# Makaleler ve Kaynaklar { #articles-and-references }

Bu sayfa, artefakt kılavuzlarının ötesine geçmek isteyen okuyucular için yüksek kaliteli başlangıç noktalarını
bir araya getirir. Her artefakta özel **Daha fazla ayrıntı** bölümleri, en hedefe yönelik alt düzey bağlantıları içerir.

## Yeni başlayanlar için DFIR yönlendirmesi { #beginner-dfir-orientation }

- [SANS Windows Forensic Analysis posteri](https://www.sans.org/posters/windows-forensic-analysis/) — yaygın Windows artefaktlarının ve konumlarının görsel bir haritası.
- [SANS Hunt Evil posteri](https://www.sans.org/posters/hunt-evil/) — pratik av (hunting) pivot noktaları ve Windows incelemesi için hatırlatmalar.
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum) — öğrenme ve ayrıştırıcı doğrulaması için herkese açık artefakt örnekleri.
- [Velociraptor artefakt başvurusu](https://docs.velociraptor.app/artifact_references/) — birçok Windows artefaktı için toplama odaklı açıklamalar ve VQL uygulamaları.
- [Plaso belgeleri](https://plaso.readthedocs.io/) — artefaktlar arası analiz için zaman çizelgesi kavramları ve ayrıştırıcılar.

## Dosya sistemi artefaktları { #file-system-artifacts }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors) and [SDDL](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) — güvenlik tanımlayıcıları, ACE'ler ve koşullu ifadeler.
- [libfsntfs biçim belgeleri](https://github.com/libyal/libfsntfs/tree/main/documentation)
- [Omer Ben-Amram'ın mft ayrıştırıcısı](https://github.com/omerbenamram/mft)
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) — INDX kaydı ve dizin slack alanı kazıma (carving) özellikli NTFS ayrıştırıcısı.
- [INDXRipper](https://github.com/harelsegev/INDXRipper) — dosya adları için `$I30` slack alanını kazır.
- [LogFileParser](https://github.com/jschicht/LogFileParser) — ayrıntılı NTFS işlem günlüğü araştırması ve ayrıştırması.
- [ntfs-linker](https://github.com/strozfriedberg/ntfs-linker) — `$LogFile`, `$MFT` ve USN geçmişini ilişkilendirir.

## Yürütme ve envanter artefaktları { #execution-and-inventory-artifacts }

- [Eric Zimmerman'ın AmcacheParser aracı](https://github.com/EricZimmerman/AmcacheParser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [libyal libscca](https://github.com/libyal/libscca) — Prefetch biçimi/kitaplığı çalışması.
- [Eric Zimmerman'ın PECmd aracı](https://github.com/EricZimmerman/PECmd)
- [Mandiant: Leveraging the Application Compatibility Cache in Forensic Investigations](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [Eric Zimmerman'ın AppCompatCacheParser aracı](https://github.com/EricZimmerman/AppCompatCacheParser)

## Kullanıcı etkinliği artefaktları { #user-activity-artifacts }

- [Microsoft Open Specifications: Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [Microsoft Open Specifications: Compound File Binary (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Eric Zimmerman'ın LECmd aracı](https://github.com/EricZimmerman/LECmd)
- [Eric Zimmerman'ın JLECmd aracı](https://github.com/EricZimmerman/JLECmd)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
- [Eric Zimmerman'ın SrumECmd aracı](https://github.com/EricZimmerman/Srum)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)

## Kayıt defteri ve kalıcılık { #registry-and-persistence }

- [Microsoft: Registry value types](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
- [Maxim Suhanov'un Windows kayıt defteri biçim belirtimi](https://github.com/msuhanov/regf)
- [yarp](https://github.com/msuhanov/yarp) and [libregf](https://github.com/libyal/libregf) — alt düzey kovan ayrıştırma ve kurtarma başvuruları.
- [Eric Zimmerman'ın Registry Explorer / RECmd belgeleri](https://ericzimmerman.github.io/)
- [Microsoft: Services registry tree](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: Task Scheduler schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [MITRE ATT&CK: Scheduled Task/Job](https://attack.mitre.org/techniques/T1053/)
- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)

## Defender ve kötü amaçlı yazılımların ele alınması { #defender-and-malware-handling }

- [Microsoft: Restore quarantined files in Defender](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [dissect.target Defender plugin](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [defender-dump](https://github.com/knez/defender-dump) — ayrı bir araştırma/çıkarma uygulaması.
- [FIRST Malware Analysis Fundamentals](https://www.first.org/global/sigs/malware/analysis/) — ele alma ve analiz kaynakları.

## Juntei ayrıştırıcı projeleri { #juntei-parser-projects }

- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (`$MFT`, `$I30`, USN Journal, `$LogFile`, `$Boot`, `$Secure:$SDS`)
- [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser)
- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

!!! note "Harici kaynaklar değişir"
    İddiaları etkilenen Windows sürümüne ve topladığınız verilere göre doğrulayın. Faydalı bir makale
    evrensel bir kural yerine belirli bir işletim sistemi kuşağını, ayrıştırıcı sürümünü veya deneyi
    açıklıyor olabilir.
