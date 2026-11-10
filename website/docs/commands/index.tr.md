# Komut Listesi { #command-list }

Ayrı ayrı ayrıştırıcılar kaynak türlerine göre gruplandırılmıştır. Bunları keşfetmek için `juntei --help`,
`juntei <SOURCE> --help` veya `juntei <SOURCE> <ARTIFACT> --help` komutlarını çalıştırın:

```bash
juntei parse-ntfs --help
juntei parse-registry amcache --help
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Birden fazla bilgisayarı kapsayan bir Windows kanıt toplaması için
[`juntei auto-parse -d <EVIDENCE> -o <OUTPUT>`](auto-parse.md) komutunu kullanın. Bu komut aynı gruplanmış
komutları çağırır ve sonuçları ve günlükleri bilgisayara ve inceleme kategorisine göre düzenler.

[`juntei super-timeline -d <PARSED> -o SuperTimeline.csv`](super-timeline.md) ayrıştırılmış artefaktlardan zaman anlamları etiketli, sıralanmış olaylar üretir; `auto-parse -j` yerel kayıtları korur.

## [Disk İmajları](mount.md) { #disk-images }

| Komut | Girdi | Ayrıntılı kılavuz |
|---|---|---|
| `mount` | VHDX / E01 / Ex01 disk imajı | [Disk İmajlarını Bağlama](mount.md) |

## [ESE Komutları](user-activity.md) { #ese-commands }

| Komut | Girdi artefaktı | Ayrıntılı kılavuz |
|---|---|---|
| `parse-ese srum` | ESE `SRUDB.dat` | [SRUM](../artifacts/srum.md) |
| `parse-ese ual` | ESE `Current.mdb`, `{GUID}.mdb`, `SystemIdentity.mdb` | [Kullanıcı Erişim Günlüğü (UAL)](../artifacts/ual.md) |
| `parse-ese webcache` | ESE `WebCacheV01.dat` | [Internet Explorer ve Eski Edge](../artifacts/webcache.md) |

## [Dosya Komutları](execution.md) { #file-commands }

| Komut | Girdi artefaktı | Ayrıntılı kılavuz |
|---|---|---|
| `parse-file defender-quarantine` | Defender karantina ağacı veya girdisi | [Defender Karantinası](../artifacts/defender-quarantine.md) |
| `parse-file jump-lists` | `*.automaticDestinations-ms`, `*.customDestinations-ms` | [Jump Lists](../artifacts/jump-lists.md) |
| `parse-file lnk` | `*.lnk` | [LNK Kısayolları](../artifacts/lnk.md) |
| `parse-file outlook` | `*.pst`, `*.ost`, `*.msg`, `*.oft` | [Outlook](../artifacts/outlook.md) |
| `parse-file prefetch` | `*.pf` | [Prefetch](../artifacts/prefetch.md) |

## [NTFS Komutları](filesystem.md) { #ntfs-commands }

| Komut | Girdi artefaktı | Ayrıntılı kılavuz |
|---|---|---|
| `parse-ntfs boot` | `$Boot` veya bir NTFS birim imajı | [Önyükleme Sektörü ($Boot)](../artifacts/boot.md) |
| `parse-ntfs i30` | `$I30` (`$INDEX_ALLOCATION`) veya bir disk imajı | [Dizin İndeksleri ($I30)](../artifacts/i30.md) |
| `parse-ntfs logfile` | NTFS `$LogFile` | [NTFS İşlem Günlüğü](../artifacts/logfile.md) |
| `parse-ntfs mft` | NTFS `$MFT` | [Ana Dosya Tablosu](../artifacts/mft.md) |
| `parse-ntfs secure-sds` | `$Secure:$SDS` (`$Secure_$SDS`) | [Güvenlik Tanımlayıcıları ($Secure:$SDS)](../artifacts/secure-sds.md) |
| `parse-ntfs usn-journal` | `$UsnJrnl:$J` | [USN Günlüğü](../artifacts/usn-journal.md) |

## [Kayıt Defteri Komutları](security-registry.md) { #registry-commands }

| Komut | Girdi artefaktı | Ayrıntılı kılavuz |
|---|---|---|
| `parse-registry amcache` | `Amcache.hve` | [Amcache](../artifacts/amcache.md) |
| `parse-registry networks` | `SOFTWARE`, `SYSTEM` | [Ağlar](../artifacts/networks.md) |
| `parse-registry lsa-secrets` | `SECURITY` | [LSA Sırları](../artifacts/lsa-secrets.md) |
| `parse-registry bcd` | `BCD` (ve `BCD-Template`) | [Önyükleme Yapılandırma Verileri (BCD)](../artifacts/bcd.md) |
| `parse-registry app-compat` | `SOFTWARE`, `NTUSER.DAT` | [Uygulama Uyumluluğu](../artifacts/appcompat.md) |
| `parse-registry auditpol` | `SECURITY` | [Denetim İlkesi](../artifacts/auditpol.md) |
| `parse-registry office` | `NTUSER.DAT` | [Microsoft Office](../artifacts/office.md) |
| `parse-registry feature-usage` | `NTUSER.DAT` | [FeatureUsage, RecentApps ve Görev Çubuğuna Sabitlenmiş Öğeler](../artifacts/featureusage.md) |
| `parse-registry app-history` | `NTUSER.DAT` | [Uygulama Geçmişi](../artifacts/app-history.md) |
| `parse-registry mru` | `NTUSER.DAT` | [En Son Kullanılan (MRU) Listeleri](../artifacts/mru.md) |
| `parse-registry bam` | `SYSTEM` (kullanıcı adları için `SAM`, `SOFTWARE` ile birlikte) | [BAM / DAM](../artifacts/bam.md) |
| `parse-registry muicache` | `UsrClass.dat` (Windows XP'de `NTUSER.DAT`) | [MuiCache](../artifacts/muicache.md) |
| `parse-registry installed-programs` | `SOFTWARE`, `NTUSER.DAT` | [Yüklü Programlar](../artifacts/installed-programs.md) |
| `parse-registry open-save-mru` | `NTUSER.DAT` | [Aç ve Farklı Kaydet İletişim Kutuları](../artifacts/opensavemru.md) |
| `parse-registry persistence` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT`, `UsrClass.dat` | [Kayıt Defteri Otomatik Başlatmaları](../artifacts/persistence.md) |
| `parse-registry read` | Herhangi bir birincil kayıt defteri kovanı | [Kayıt Defteri Kovanları](../artifacts/registry.md) |
| `parse-registry recent-docs` | `NTUSER.DAT` | [RecentDocs](../artifacts/recentdocs.md) |
| `parse-registry remote-access` | `NTUSER.DAT` (kullanıcı SID'leri için `SOFTWARE` ile birlikte) | [Uzaktan Erişim (Kullanıcı Tarafı)](../artifacts/remote-access.md) |
| `parse-registry sam` | `SAM` | [SAM Yerel Hesapları](../artifacts/sam.md) |
| `parse-registry security-settings` | `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` | [Güvenlik Ayarları](../artifacts/security-settings.md) |
| `parse-registry services` | `SYSTEM` | [Windows Hizmetleri](../artifacts/services.md) |
| `parse-registry shimcache` | `SYSTEM` veya dışa aktarılmış bir AppCompatCache değeri | [ShimCache](../artifacts/shimcache.md) |
| `parse-registry system-info` | Tek bir sistemin `SYSTEM`, `SOFTWARE`, `SAM`, `SECURITY` kovanları | [Sistem Bilgileri](../artifacts/system-info.md) |
| `parse-registry taskcache` | `SOFTWARE` | [Görev Zamanlayıcı Önbelleği](../artifacts/taskcache.md) |
| `parse-registry usb` | `SYSTEM`; `SOFTWARE` ve `NTUSER.DAT` ile birlikte | [USB Aygıtları](../artifacts/usb.md) |
| `parse-registry user-assist` | `NTUSER.DAT` | [UserAssist](../artifacts/userassist.md) |
| `parse-registry shell-bags` | `UsrClass.dat`, `NTUSER.DAT` | [ShellBags](../artifacts/shellbags.md) |

## [SQLite Komutları](sqlite.md) { #sqlite-commands }

| Komut | Girdi artefaktı | Ayrıntılı kılavuz |
|---|---|---|
| `parse-sqlite browser` | `History`, `Cookies`, `Web Data`, `places.sqlite`, `cookies.sqlite`, `formhistory.sqlite` | [Web Tarayıcıları](../artifacts/browser.md) |

## Kaynak türleri ve biçim değişiklikleri { #source-types-and-format-changes }

Gruplar, Juntei'nin şu anda desteklediği biçimleri tanımlar. `parse-registry amcache`, `Amcache.hve`
dosyasını okur; bu, SQLite Amcache envanter veritabanlarının desteklendiği anlamına gelmez. Artefakt
ayrıştırıcı crate'leri ayrı tutulur; böylece başka bir depolama biçimi için destek bağımsız olarak
eklenebilir.

Eski düz ayrıştırıcı adlarının yerini yukarıdaki kaynak ve artefakt yolları almıştır. Mevcut ayrıştırıcı
seçeneklerini artefakt adından sonra yazmaya devam edin. `auto-parse` ve mevcut kayıt defteri alt komutları
adlarını korur.

## [Persistence Hunter](persistence-hunter.md) { #persistence-hunter }

`juntei persistence-hunter` birden fazla kaynak biçiminde tarama yapar; bu nedenle üst düzey bir analiz
komutudur. `parse-registry persistence` komutunun sunduğu tam yapılandırma envanterini 131 kontrol, izin/engel
sınıflandırması, yük (payload) meta verileri, kapsam ve zaman damgası ilişkilendirmesiyle tamamlar.

## Üst düzey kullanım { #top-level-usage }

```text
Usage: juntei <COMMAND>

Commands:
  auto-parse          Discover and parse Windows evidence
  help                Print this message or the help of the given subcommand(s)
  mount               Mount a VHDX or E01 disk image read-only (FUSE)
  parse-ese           Parse ESE databases
  parse-file          Parse file artifacts
  parse-ntfs          Parse NTFS artifacts
  parse-registry      Parse registry hives
  parse-sqlite        Parse SQLite databases
  persistence-hunter  Hunt offline Windows persistence

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Kaynak sayfaları komut kullanımını yeniden verir ve kısa örnekler sunar. Artefakt kılavuzları kanıtı, çıktı
alanlarını, sınırlamaları ve inceleme stratejisini açıklar.
