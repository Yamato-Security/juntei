# Persistence Hunter { #persistence-hunter }

Persistensi adalah cara untuk membuat kode berjalan kembali: saat Windows dinyalakan, pengguna masuk (sign in), aplikasi
dibuka, waktu terjadwal tiba, atau peristiwa lain terjadi. Penyerang dapat menyalahgunakan pengaturan yang sama
yang digunakan perangkat lunak sah untuk pembaruan, dukungan perangkat, keamanan, dan administrasi.

`juntei persistence-hunter` memeriksa bukti Windows yang telah diambil secara offline dengan **131 pemeriksaan** yang mencakup
mekanisme-mekanisme ini serta anomali akun/proteksi yang terkait. Perintah ini membaca hive registry, berkas task,
skrip, pengaturan aplikasi, dan catatan basis data tertentu.

Halaman ini menjelaskan setiap pemeriksaan yang diimplementasikan. Untuk opsi lengkap, field keluaran, profil, dan
penanganan timestamp, buka [panduan perintah](../commands/persistence-hunter.md). Halaman
[ucapan terima kasih](acknowledgements.md) menautkan proyek Python PyrsistenceSniper dan alat PowerShell
asli PersistenceSniper.

## Memulai investigasi { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

Gunakan `--min-severity info` untuk inventaris kandidat yang lengkap. Perintah manual secara default menggunakan
`medium`; profilnya dapat mengklasifikasikan kecocokan allow parsial sebagai `LOW` bahkan ketika bukti payload atau
penanda tangan yang hilang mencegah perbandingan penuh. Baik kecocokan allow maupun entri yang tidak dikenal tidak menentukan
apakah sesuatu berbahaya. `auto-parse` sudah mempertahankan semua tingkat keparahan dan menulis temuan
di bawah kategori `persistence/` milik setiap komputer.

Untuk berfokus pada satu mekanisme, salin **Check ID**-nya dari bagian di bawah:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

Secara default, perintah ini menggunakan worker pool bersama. Tambahkan `-T 4` untuk memilih empat worker, atau `-T 1`
untuk pemindaian single-thread. Masukan dapat berisi beberapa komputer; bukti masing-masing tetap terpisah.

## Membaca temuan sebagai bukti { #read-findings-as-evidence }

| Pertanyaan | Apa yang perlu diperiksa |
|---|---|
| Apa yang dikonfigurasi, dan di mana? | `CheckId`, `Path`, dan `Value` yang lengkap, termasuk argumen. Telusuri `SourceFile` ke hive, berkas, atau basis data yang diambil. |
| Apakah target ada di dalam koleksi? | `ResolvedPath`, `ResolutionStatus`, dan `Exists`. Referensi yang tidak dikumpulkan, ambigu, atau tidak didukung berstatus tidak diketahui; Juntei tidak menggantinya dengan berkas dari mesin analis. |
| Apakah konfigurasinya efektif? | Pemicu yang sebenarnya, flag pengaktifan, control set yang dipilih, versi Windows/aplikasi, pengguna, dan arsitektur. Beberapa pemeriksaan sengaja mempertahankan konfigurasi yang tidak aktif. |
| Apakah ini serangan? | `Severity` dan `PolicyReason` menjelaskan pencocokan profil, bukan putusan. Bandingkan dengan perangkat lunak yang diharapkan, perintah lengkap, hash berkas, bukti eksekusi independen, dan timeline di sekitarnya. |
| Siapa yang dapat menjalankannya? | `AccessGained` adalah petunjuk cakupan/konteks. Konfigurasi tingkat mesin sering kali menyebut `SYSTEM`, tetapi hal itu tidak membuktikan bahwa proses yang diluncurkan berjalan sebagai SYSTEM. Principal task dan SID pemilik BITS memberikan konteks yang lebih spesifik bila tersedia. |
| Kapan ini berubah? | `KeyLastWrite` memberi tanggal pada seluruh kunci registry. `LastChange`, `ChangeSource`, dan `ChangeEvidence` mendeskripsikan korelasi MFT/EVTX yang terpisah, bila tersedia. Tidak satu pun dari field tersebut secara otomatis memberi tanggal pada eksekusi payload. |
| Seberapa andal status yang dipulihkan? | Baca `RecordStatus`, `Coverage.csv`, dan peringatan untuk bukti yang di-replay, di-carve, rusak, hilang, atau terbatas. Catatan historis yang dipulihkan mungkin sudah tidak aktif lagi. |
| Apakah penerbit yang disebutkan dapat dipercaya? | `Signer` dan `SignerStatus` adalah metadata PE/sertifikat/katalog yang bersifat deskriptif. Juntei tidak memvalidasi tanda tangan Authenticode atau kepercayaan sertifikat. |

Tidak adanya temuan tidak membuktikan tidak adanya persistensi. Baris coverage `complete` berarti
sumber terkumpul yang relevan telah dibaca, bukan berarti akuisisi berisi setiap berkas yang relevan,
setiap pengguna, atau setiap mekanisme persistensi Windows yang mungkin. Status coverage lainnya menjelaskan pemeriksaan yang hilang,
parsial, dinonaktifkan, atau gagal. Hunter ini bukan pemindaian memori live, jaringan, atau identitas cloud.

## Katalog dan notasi lokasi { #catalogue-and-location-notation }

Katalog ini didasarkan pada PyrsistenceSniper, ditambah pemeriksaan native `start_menu_com`;
deskripsi di bawah mendokumentasikan
perilaku native Juntei, termasuk keterbatasan buktinya. Terdapat **116 pemeriksaan berbasis
registry dan 15 pemeriksaan berkas/repositori**. Beberapa pemeriksaan berkas juga menggabungkan data registry, sehingga angka ini adalah
hitungan implementasi, bukan persyaratan akuisisi yang saling eksklusif. Mekanisme-mekanisme
dikelompokkan berdasarkan tujuan investigasi di bawah, dan diurutkan secara alfabetis berdasarkan Check ID di dalam setiap kelompok.

- `HKLM\SYSTEM`, `HKLM\SOFTWARE`, dan `HKLM\SAM` merujuk pada hive mesin yang diambil, biasanya
  dikumpulkan dari `Windows\System32\config`, beserta log transaksi bila tersedia.
- `HKU\<user>` berarti hive pengguna terkait yang diambil. Sebagian besar pengaturan pengguna berasal dari NTUSER.DAT;
  registrasi class dapat berasal dari `AppData\Local\Microsoft\Windows\UsrClass.dat` sebagaimana dicatat.
- `<ControlSet>` berarti control set aktif yang dipilih di SYSTEM. Lokasi historis tambahan
  untuk pemeriksaan DSRM dinyatakan secara eksplisit di bawah. Periksa `SYSTEM\Select` saat meninjaunya.
- `<name>` adalah komponen path variabel. Dalam tabel target registry, `<child>` berarti registrasi
  anak langsung; ini tidak berarti penelusuran rekursif tanpa batas. Pemeriksaan rekursif eksplisit
  mendeskripsikan penelusurannya secara terpisah. Tanda kurung siku pada path Office menunjukkan versi opsional.
- Tabel mencantumkan tampilan (view) mesin dan pengguna yang sebenarnya dipilih oleh pemeriksaan. Tampilan 32-bit dicantumkan
  bila diimplementasikan; jangan berasumsi bahwa setiap pemeriksaan memeriksa setiap arsitektur/tampilan yang mungkin.
- Pemeriksaan nilai generik mengeluarkan entri terkonfigurasi yang tidak kosong, termasuk elemen multi-string dan
  angka nol bila berlaku. Pemeriksaan ini, dengan sendirinya, tidak mensyaratkan payload berbahaya, pemicu yang
  aktif, atau eksekusi yang berhasil. Predikat yang lebih selektif dinyatakan per pemeriksaan.

ID ATT&CK adalah pemetaan katalog yang diwarisi dan tautan untuk dipelajari lebih lanjut. ID tersebut mendeskripsikan potensi
penyalahgunaan, bukan temuan bahwa penyerang melakukan teknik tersebut; pemeriksaan pengaturan proteksi dan diagnostik
dapat mendeskripsikan kondisi yang memungkinkan, bukan mekanisme persistensi yang berdiri sendiri.

| Kelompok mekanisme | Pemeriksaan |
|---|---:|
| [Boot, sign-in, dan startup](#boot-sign-in-and-startup) | 32 |
| [Layanan dan ekstensi sistem](#services-and-system-extensions) | 13 |
| [Scheduled task](#scheduled-tasks) | 3 |
| [WMI, BITS, dan pemicu peristiwa lainnya](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorer, COM, dan handler berkas atau URL](#explorer-com-and-file-or-url-handlers) | 15 |
| [Aplikasi dan add-in Office](#office-applications-and-add-ins) | 7 |
| [Debugger, crash handler, dan compatibility shim](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [Runtime .NET dan pengaturan profiler](#net-runtime-and-profiler-settings) | 7 |
| [Override pencarian DLL dan executable](#dll-and-executable-lookup-overrides) | 18 |
| [Akun dan visibilitas sign-in](#accounts-and-sign-in-visibility) | 4 |
| [Komponen autentikasi dan proteksi kredensial](#authentication-components-and-credential-protection) | 8 |
| **Total** | **131** |

## Boot, sign-in, dan startup { #boot-sign-in-and-startup }

Pengaturan-pengaturan ini mengatur agar perintah atau pustaka berjalan selama boot, sign-in, atau inisialisasi desktop. Tinjau versi Windows dan pengguna yang bersangkutan: sebuah pengaturan dapat saja sudah usang, dinonaktifkan, atau ditujukan untuk kiosk, bukan oleh penyerang.

### Active Setup {#active_setup}

**Check ID:** `active_setup` · **ATT&CK dalam katalog:** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**Yang diperiksa Juntei:** Membaca `StubPath` dari registrasi komponen Active Setup tingkat mesin pada tampilan native dan 32-bit. Windows dapat menggunakan perintah-perintah ini saat menyiapkan profil pengguna; pemeriksaan ini melewati nilai placeholder literal `U` dan `/UserInstall`.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` dan tree `Wow6432Node` yang bersesuaian.

**Cara menginvestigasi:** Bandingkan setiap komponen dengan produk yang terinstal dan periksa argumen perintahnya. Registrasi tingkat mesin dapat menyebabkan eksekusi di sesi pengguna yang sedang sign in; label `SYSTEM` pada keluaran mendeskripsikan cakupan konfigurasi, bukan bukti token proses yang dihasilkan.

### Boot Execute {#boot_execute}

**Check ID:** `boot_execute` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca multi-string `BootExecute` milik Session Manager, dengan mempertahankan setiap entri individual yang tidak kosong. Pemeriksaan ini mencatat perintah yang dikonfigurasi untuk fase boot awal.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**Cara menginvestigasi:** `autocheck autochk *` adalah nilai default yang sudah dikenal, dan alat disk dapat menambahkan entri yang sah. Investigasi perintah yang ditambahkan beserta berkasnya tanpa berasumsi bahwa setiap entri non-default telah dijalankan atau berbahaya.

### Boot Verification Program {#boot_verification_program}

**Check ID:** `boot_verification_program` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca `ImagePath` milik `BootVerificationProgram`, yaitu lokasi executable yang terkait dengan verifikasi boot.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**Cara menginvestigasi:** Periksa apakah mekanisme lama ini berlaku untuk versi Windows dan konfigurasi boot yang diambil. Resolusikan berkas target dan bandingkan tujuannya dengan perangkat lunak manajemen boot yang diharapkan.

### Nilai Load Explorer {#explorer_load}

**Check ID:** `explorer_load` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca nilai `Load` yang terkait dengan konfigurasi inisialisasi sign-in Windows dari hive mesin dan hive pengguna.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**Cara menginvestigasi:** Investigasi perintah yang tidak dikenal, tujuan yang dapat ditulisi pengguna, dan argumen yang mengambil atau mendekode konten. Konfirmasikan keberlakuan dan eksekusi secara terpisah; hasil kosong tidak mencakup mekanisme sign-in lainnya.

### Font Drivers {#font_drivers}

**Check ID:** `font_drivers` · **ATT&CK dalam katalog:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Yang diperiksa Juntei:** Mengenumerasi nilai-nilai yang tidak kosong di kunci registry lawas `Font Drivers` sebagai registrasi pustaka yang potensial.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Perlakukan ini sebagai permukaan konfigurasi lawas yang tidak biasa. Periksa dukungan versi Windows dan pustaka yang direferensikan sebelum menyimpulkan bahwa pustaka tersebut dapat dimuat, dan koroborasikan penggunaan sebenarnya dengan bukti proses atau image-load.

### DLL Ekstensi Group Policy {#gp_extension_dlls}

**Check ID:** `gp_extension_dlls` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca `DllName` dari setiap registrasi ekstensi Group Policy langsung di bawah Winlogon. DLL ini memperluas pemrosesan kebijakan.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**Cara menginvestigasi:** Produk manajemen dan keamanan dapat menginstal ekstensi. Cocokkan GUID ekstensi dan DLL dengan produk yang diharapkan, dan periksa bukti pemrosesan kebijakan dan image-load sebelum menyatakan adanya eksekusi.

### Skrip Group Policy {#gp_scripts}

**Check ID:** `gp_scripts` · **ATT&CK dalam katalog:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Yang diperiksa Juntei:** Mengurai pasangan perintah/parameter dari berkas registrasi Group Policy lokal yang diambil, yaitu `scripts.ini` dan `psscripts.ini`, termasuk penyimpanan kebijakan lokal per pengguna. Bagian startup/shutdown dan logon/logoff mempertahankan perintah dan argumen yang dikonfigurasi, dengan bagian mesin dibedakan dari bagian pengguna.

**Bukti yang diperiksa:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` dan berkas `GroupPolicy\User\Scripts` yang setara.
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`.

**Cara menginvestigasi:** Hanya lokasi registrasi kebijakan lokal yang diimplementasikan yang diperiksa; ini bukan audit SYSVOL/GPO yang lengkap. Berkas skrip yang berada di tempat lain di bawah direktori kebijakan tidaklah cukup. Korelasikan registrasinya dengan bukti pemrosesan kebijakan dan eksekusi.

### Skrip Logon (UserInitMprLogonScript) {#logon_scripts}

**Check ID:** `logon_scripts` · **ATT&CK dalam katalog:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Yang diperiksa Juntei:** Membaca nilai lingkungan `UserInitMprLogonScript` milik pengguna, yang dapat memilih skrip untuk sign-in pengguna tersebut.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**Cara menginvestigasi:** Bandingkan dengan skrip logon yang disetujui dan periksa skrip yang sebenarnya. Nilai ini tidak membuktikan adanya sign-in atau eksekusi berikutnya, dan skrip yang disediakan domain memerlukan bukti kebijakan tambahan.

### Platform Execute {#platform_execute}

**Check ID:** `platform_execute` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca setiap entri `PlatformExecute` yang tidak kosong di Session Manager. Pemeriksaan ini mengidentifikasi program yang dipilih untuk inisialisasi boot terkait platform.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**Cara menginvestigasi:** Alur kerja perangkat keras atau provisioning dapat menjelaskan sebuah entri. Tentukan apakah jalur boot pada build Windows ini menggunakannya dan investigasi image yang direferensikan, alih-alih menyimpulkan serangan dari nama nilainya.

### DLL Ekstensi Klien RDP {#rdp_clx_dll}

**Check ID:** `rdp_clx_dll` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca `ClxDllPath` di konfigurasi pengguna default Terminal Server sebagai kemungkinan lokasi ekstensi terkait RDP.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**Cara menginvestigasi:** Periksa versi Windows/RDP dan ekstensi yang terinstal. DLL yang dikonfigurasi merupakan petunjuk untuk analisis image-load; nilai registry saja tidak menunjukkan bahwa koneksi RDP atau pemuatan DLL telah terjadi.

### DLL Virtual Channel RDP {#rdp_virtual_channel}

**Check ID:** `rdp_virtual_channel` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Mengenumerasi nilai-nilai di bawah registrasi `Default\AddIns` Terminal Server Client tingkat mesin. Nilai biner didekode sebagai UTF-16LE bila memungkinkan, sehingga path ekstensi yang dapat digunakan tetap dipertahankan.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: semua nilai.

**Cara menginvestigasi:** Klien RDP dan produk pengalihan perangkat secara sah mendaftarkan add-in. Periksa DLL atau perintah yang sebenarnya serta sesi klien yang relevan; tidak setiap nilai di bawah add-in merupakan path executable.

### Program Startup RDP WDS {#rdp_wds_startup}

**Check ID:** `rdp_wds_startup` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca `StartupPrograms` dari konfigurasi driver RDP Windows Station. Pemeriksaan ini mengidentifikasi program yang dikonfigurasi untuk startup sesi jarak jauh.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**Cara menginvestigasi:** `rdpclip` adalah entri yang sudah dikenal. Investigasi tambahan entri dan konfigurasi RDP host, serta gunakan catatan sesi/proses untuk menentukan apakah program yang dikonfigurasi benar-benar dijalankan.

### Kunci Run Registry {#run_keys}

**Check ID:** `run_keys` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Mengenumerasi nilai Run, RunOnce, RunEx, RunOnceEx, dan Run Explorer kebijakan dari hive mesin/pengguna, ditambah kunci 32-bit mesin yang tercantum. Bagian RunEx/RunOnceEx ditelusuri dengan pemeriksaan batas dan siklus.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Semua nilai yang tidak kosong |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | Semua nilai yang tidak kosong |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Semua nilai yang tidak kosong |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | Semua nilai yang tidak kosong |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Semua nilai yang tidak kosong |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | Semua nilai yang tidak kosong |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Semua nilai yang tidak kosong |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | Semua nilai yang tidak kosong |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Semua nilai yang tidak kosong |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | Semua nilai yang tidak kosong |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | Semua nilai yang tidak kosong |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | Semua nilai yang tidak kosong |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Updater perangkat lunak dan aplikasi tray umumnya muncul di sini. Bandingkan executable, argumen perintah, dan konteks pengguna. Entri RunOnce dapat menghilang setelah digunakan; status yang diambil bukanlah riwayat lengkap dari sign-in sebelumnya.

### RunServices (Lawas) {#run_services}

**Check ID:** `run_services` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Mengenumerasi nilai-nilai di kunci lawas tingkat mesin `RunServices`.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Ini adalah permukaan konfigurasi historis, bukan jaminan bahwa build Windows modern mengeksekusinya. Investigasi target dan keberlakuannya pada sistem operasi, lalu cari bukti eksekusi yang independen.

### RunServicesOnce (Lawas) {#run_services_once}

**Check ID:** `run_services_once` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Mengenumerasi nilai-nilai di kunci lawas tingkat mesin `RunServicesOnce`.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Perlakukan nilai yang tidak kosong sebagai petunjuk konfigurasi. Periksa apakah versi Windows sumber mendukung mekanisme ini dan apakah perangkat lunak deployment atau kompatibilitas dapat menjelaskan entri tersebut.

### S0 Initial Command {#s0_initial_command}

**Check ID:** `s0_initial_command` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca `S0InitialCommand` dari Session Manager sebagai potensi perintah inisialisasi Session 0.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**Cara menginvestigasi:** Tetapkan keberlakuannya pada build Windows dan status boot yang diambil. Perintah yang dikonfigurasi bukanlah catatan eksekusi boot yang memiliki timestamp; periksa berkasnya dan korelasikan dengan telemetri boot/proses.

### Session Manager Execute {#session_manager_execute}

**Check ID:** `session_manager_execute` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca entri `Execute` yang tidak kosong dari Session Manager, sehingga menampilkan program yang dipilih untuk inisialisasi sesi.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**Cara menginvestigasi:** Bandingkan dengan baseline image yang diharapkan dan periksa executable native atau path yang tidak biasa. Tetapkan apakah boot/sesi yang relevan terjadi setelah perubahan sebelum menyatakan adanya eksekusi.

### Session Manager SubSystems {#session_manager_subsystems}

**Check ID:** `session_manager_subsystems` · **ATT&CK dalam katalog:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Yang diperiksa Juntei:** Membaca nilai `Windows` di kunci `SubSystems` milik Session Manager, yang mengonfigurasi perintah subsistem Windows.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**Cara menginvestigasi:** Perintah normalnya panjang dan bergantung pada versi. Bandingkan nilai lengkap dan argumennya dengan baseline tepercaya dari build yang sama; mengubah pengaturan subsistem yang kritis juga dapat merusak proses startup.

### Setup Execute {#setup_execute}

**Check ID:** `setup_execute` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca entri `SetupExecute` di Session Manager, yang mencakup perintah terkait setup yang dikonfigurasi untuk startup.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**Cara menginvestigasi:** Aktivitas instalasi, upgrade, dan pemulihan dapat membuat entri yang sah. Bandingkan dengan log setup dan image yang direferensikan sebelum mengatribusikan konfigurasi tersebut ke persistensi.

### Pengalihan Startup Shell Folders {#shell_folders_startup}

**Check ID:** `shell_folders_startup` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membandingkan `Startup` dan `Common Startup` di Shell Folders maupun User Shell Folders dengan lokasi Startup default. Pengalihan non-default dilaporkan, dan isi folder yang dikumpulkan diperiksa.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` dan `User Shell Folders`: `Common Startup`.
- Pengaturan `Software\Microsoft\Windows\CurrentVersion\Explorer` yang setara di NTUSER.DAT: `Startup`, ditambah berkas yang diambil di lokasi pengalihan.

**Cara menginvestigasi:** Folder Startup yang dialihkan dapat menyembunyikan autostart, tetapi pengalihan bisa saja sah. Path harus dapat diresolusikan di dalam bukti yang diambil; tujuan yang tidak dikumpulkan berstatus tidak diketahui, bukan bukti bahwa folder tersebut kosong.

### Override Shell Launcher {#shell_launcher}

**Check ID:** `shell_launcher` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca nilai `Shell` kebijakan dan pemetaan Shell boot `system.ini` dari SOFTWARE. Keduanya merupakan permukaan alternatif untuk pemilihan shell.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**Cara menginvestigasi:** Kiosk dan appliance aplikasi dapat dengan sengaja mengganti Explorer. Bandingkan shell yang dipilih dengan peran mesin dan konfirmasikan bahwa pengaturan tersebut efektif untuk sesi yang relevan.

### Folder Startup {#startup_folder}

**Check ID:** `startup_folder` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Memeriksa berkas langsung di folder Startup default per pengguna dan folder Startup bersama, dengan mengikuti pengalihan folder Startup pertama yang tersedia. Target pintasan dipulihkan bila `.lnk` yang dikumpulkan dapat diurai.

**Bukti yang diperiksa:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` dan `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`.
- Pengalihan `Common Startup` / `Startup` yang berlaku di pengaturan Shell Folders dan User Shell Folders mesin/pengguna.

**Cara menginvestigasi:** Tautan dan berkas Startup dapat meluncurkan aplikasi yang sah saat sign-in. Periksa argumen target serta nama berkasnya. Folder bersama adalah lokasi konfigurasi bersama; hal ini tidak membuktikan bahwa program yang diluncurkan menggunakan token SYSTEM.

### Terminal Services Initial Program {#ts_initial_program}

**Check ID:** `ts_initial_program` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca `InitialProgram` kebijakan Terminal Services dan konfigurasi Run/RunOnce/RunOnceEx instalasi Terminal Server tingkat mesin, termasuk bagian RunOnceEx bersarang dengan batasan.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | Semua nilai yang tidak kosong |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | Semua nilai yang tidak kosong |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Deployment aplikasi jarak jauh dapat menggunakan pengaturan ini dengan sengaja. Tetapkan sesi yang terdampak dan telusuri perintah yang dipilih ke bukti proses; registrasi saja tidak menunjukkan adanya logon jarak jauh.

### Winlogon AppSetup {#winlogon_appsetup}

**Check ID:** `winlogon_appsetup` · **ATT&CK dalam katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Yang diperiksa Juntei:** Membaca `AppSetup` dari Winlogon, dengan memisahkan entri yang dipisahkan koma menjadi kandidat-kandidat terpisah.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**Cara menginvestigasi:** Bandingkan setiap komponen dengan perangkat lunak setup/sign-in Windows yang diharapkan. Entri daftar bukanlah eksekusi teramati yang terpisah, dan relevansinya dapat bergantung pada versi Windows dan status deployment.

### Winlogon GinaDLL {#winlogon_gina_dll}

**Check ID:** `winlogon_gina_dll` · **ATT&CK dalam katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Yang diperiksa Juntei:** Membaca pengaturan pengganti lawas `GinaDLL` milik Winlogon. GINA adalah model ekstensi sign-in Windows yang lebih lama.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**Cara menginvestigasi:** Ini terutama relevan untuk sistem sebelum Vista. Kunci yang tersisa pada sistem yang lebih baru bukanlah bukti adanya ekstensi logon yang efektif; periksa versi sumber dan DLL apa pun yang direferensikan.

**Detail lebih lanjut:** [Dokumentasi Microsoft](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina).

### Winlogon MPNotify {#winlogon_mpnotify}

**Check ID:** `winlogon_mpnotify` · **ATT&CK dalam katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Yang diperiksa Juntei:** Membaca pengaturan perintah/pustaka `mpnotify` milik Winlogon, sebuah permukaan ekstensi terkait sign-in.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**Cara menginvestigasi:** Bandingkan target dan argumen dengan baseline Windows yang sesuai. Periksa perilaku spesifik versi dan korelasikan dengan sign-in berikutnya sebelum menyatakan bahwa kode telah berjalan.

### Paket Notify Winlogon {#winlogon_notify_packages}

**Check ID:** `winlogon_notify_packages` · **ATT&CK dalam katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Yang diperiksa Juntei:** Membaca `DllName` dari setiap registrasi paket `Notify` Winlogon langsung.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**Cara menginvestigasi:** Paket notifikasi adalah mekanisme ekstensi lawas. Tentukan apakah versi Windows yang diambil memuatnya dan bedakan perangkat lunak autentikasi yang disetujui dari pustaka yang tidak dapat dijelaskan.

### Winlogon Shell {#winlogon_shell}

**Check ID:** `winlogon_shell` · **ATT&CK dalam katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Yang diperiksa Juntei:** Membaca nilai `Shell` Winlogon tingkat mesin dan per pengguna untuk menampilkan pemilihan shell desktop.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**Cara menginvestigasi:** Explorer adalah yang umum, tetapi kiosk dapat menggunakan shell lain. Investigasi perintah yang ditambahkan atau path yang tidak biasa dan tentukan konfigurasi efektif pengguna mana yang berlaku saat sign-in.

### Winlogon System {#winlogon_system}

**Check ID:** `winlogon_system` · **ATT&CK dalam katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Yang diperiksa Juntei:** Membaca nilai `System` milik Winlogon dan memisahkan entri yang dibatasi koma menjadi kandidat-kandidat individual.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**Cara menginvestigasi:** Perlakukan ini sebagai petunjuk konfigurasi sign-in yang sensitif terhadap versi. Periksa setiap komponen yang direferensikan dan koroborasikan dengan bukti eksekusi alih-alih memperlakukan daftar tersebut sebagai riwayat eksekusi.

### Winlogon Taskman {#winlogon_taskman}

**Check ID:** `winlogon_taskman` · **ATT&CK dalam katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Yang diperiksa Juntei:** Membaca nilai `Taskman` Winlogon mesin/pengguna untuk perintah manajemen task alternatif.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**Cara menginvestigasi:** Tentukan perilaku Windows mana yang akan menggunakan pengaturan ini dan apakah targetnya sesuai harapan. Pengganti yang dikonfigurasi bukanlah bukti bahwa pengguna memanggil fitur tersebut.

### Winlogon Userinit {#winlogon_userinit}

**Check ID:** `winlogon_userinit` · **ATT&CK dalam katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Yang diperiksa Juntei:** Membaca nilai `Userinit` Winlogon mesin/pengguna, dengan memisahkan program yang dipisahkan koma sambil mengabaikan item kosong di bagian akhir.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**Cara menginvestigasi:** Entri normal sering kali menunjuk ke `userinit.exe`. Periksa entri tambahan dan path yang diubah. Konfirmasikan pengguna yang bersangkutan dan sign-in setelahnya; `KeyLastWrite` milik seluruh kunci, bukan khusus nilai ini.

### Winlogon VMApplet {#winlogon_vmapplet}

**Check ID:** `winlogon_vmapplet` · **ATT&CK dalam katalog:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Yang diperiksa Juntei:** Membaca pengaturan perintah `VMApplet` milik Winlogon, yang mencakup permukaan peluncuran applet sistem yang lebih lama.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**Cara menginvestigasi:** Periksa keberlakuan pada sistem operasi dan applet yang diharapkan. Perintah yang tidak terduga patut diinvestigasi tetapi tidak membuktikan bahwa antarmuka pemicunya telah dibuka.

## Layanan dan ekstensi sistem { #services-and-system-extensions }

Layanan Windows dan komponen sistem yang dapat diperluas menyediakan banyak cara sah untuk memuat executable atau DLL. Bandingkan target yang dikonfigurasi dengan produk yang diharapkan, akun layanan, pengaturan start, dan riwayat instalasi yang didukung sumber.

### DLL DiagTrack {#diagtrack_dll}

**Check ID:** `diagtrack_dll` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca `ImagePath` milik layanan DiagTrack. Meskipun nama di katalog berorientasi pada DLL, pemeriksaan ini memeriksa perintah image layanan yang dikonfigurasi.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**Cara menginvestigasi:** Bandingkan dengan baseline layanan dan pemeriksaan layanan umum. Konfigurasi yang sama dapat menghasilkan lebih dari satu hasil pemeriksaan; jumlah kandidat bukanlah jumlah serangan yang berbeda.

### DLL DiagTrack Listener {#diagtrack_listener_dll}

**Check ID:** `diagtrack_listener_dll` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca `FileName` dari konfigurasi autologger DiagTrack-Listener. Ini adalah pengaturan autologger ETW, bukan bukti bahwa sebuah DLL dimuat.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**Cara menginvestigasi:** Periksa nilai mentah dan konfigurasi logging Windows. Katalog melabeli ini sebagai teknik DLL, tetapi pencarian ini tidak memvalidasi DLL yang dapat dimuat atau membuktikan eksekusi melalui nilai ini.

### DLL Helper Netsh {#netsh_helper}

**Check ID:** `netsh_helper` · **ATT&CK dalam katalog:** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**Yang diperiksa Juntei:** Mengenumerasi nilai yang tidak kosong di bawah kunci registrasi helper `NetSh` mesin. Helper dapat memperluas `netsh.exe` dengan perintah berbasis DLL.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Produk jaringan dapat menginstal helper. Cocokkan setiap nilai dengan produk yang diharapkan dan periksa pustaka yang telah diresolusikan, lalu cari eksekusi netsh atau image-load yang relevan.

### Print Monitors {#print_monitors}

**Check ID:** `print_monitors` · **ATT&CK dalam katalog:** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**Yang diperiksa Juntei:** Membaca `Driver` di bawah setiap registrasi Print Monitor langsung di control set aktif.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**Cara menginvestigasi:** Perangkat lunak pencetakan dan port monitor secara sah memperluas spooler. Bandingkan DLL dan metadata produk dengan inventaris printer, dan konfirmasikan pemuatannya dengan bukti spooler/image-load.

### Print Processors {#print_processors}

**Check ID:** `print_processors` · **ATT&CK dalam katalog:** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**Yang diperiksa Juntei:** Membaca `Driver` di bawah tree registrasi print processor x64 dan x86.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**Cara menginvestigasi:** Suite printer pihak ketiga dapat menambahkan processor di luar default Windows yang sudah dikenal. Tinjau kedua arsitektur, DLL yang sebenarnya, dan perangkat lunak printer yang terinstal sebelum menetapkan adanya niat jahat.

### Ekstensi Service Control Manager {#scm_extension}

**Check ID:** `scm_extension` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Mengenumerasi nilai yang tidak kosong di `ServiceControlManagerExtension`, sebuah permukaan ekstensi service manager.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Periksa pustaka yang disebutkan dan apakah build Windows ini menggunakan pengaturan tersebut. Korelasikan dengan aktivitas boot/layanan; pemeriksaan ini menginventarisasi registrasi, bukan membuktikan ekstensi yang telah dimuat.

### DLL Plugin Tingkat Server DNS {#server_level_plugin_dll}

**Check ID:** `server_level_plugin_dll` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca pengaturan `ServerLevelPluginDll` milik layanan DNS di control set SYSTEM yang aktif.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**Cara menginvestigasi:** Ini relevan untuk sistem yang menjalankan role DNS Server, termasuk sebagian domain controller. Investigasi DLL, akun layanan, dan bukti restart/pemuatan layanan; role yang tidak aktif atau tidak ada akan mengubah kesimpulan.

### Perintah Kegagalan Layanan {#service_failure_command}

**Check ID:** `service_failure_command` · **ATT&CK dalam katalog:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Yang diperiksa Juntei:** Membaca `FailureCommand` setiap layanan di control set aktif, sehingga menampilkan perintah pemulihan yang dikonfigurasi.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`.

**Cara menginvestigasi:** Perintah hanya dijalankan ketika tindakan pemulihan layanan memilihnya dan kegagalan yang bersangkutan terjadi. Periksa konfigurasi pemulihan dan catatan kegagalan/proses; nilai ini saja tidak membuktikan adanya tindakan pemulihan yang aktif.

### DLL Agen Ekstensi SNMP {#snmp_extension_agent}

**Check ID:** `snmp_extension_agent` · **ATT&CK dalam katalog:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Yang diperiksa Juntei:** Mengenumerasi nilai di bawah kunci registrasi `ExtensionAgents` milik layanan SNMP. Nilai dapat mengidentifikasi lokasi registry lain alih-alih path DLL langsung.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Telusuri registrasi tidak langsung secara manual hingga ke pustakanya dan verifikasi bahwa SNMP terinstal dan digunakan. String yang tidak teresolusi dalam keluaran ini bukanlah bukti bahwa DLL hilang atau berbahaya.

### Time Providers {#time_providers}

**Check ID:** `time_providers` · **ATT&CK dalam katalog:** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**Yang diperiksa Juntei:** Membaca `DllName` dari registrasi `TimeProviders` Windows Time langsung.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**Cara menginvestigasi:** Tinjau pengaturan pengaktifan provider dan status layanan Windows Time secara terpisah; pemeriksaan path DLL tidak mensyaratkan provider yang aktif. Produk sinkronisasi waktu dapat menyediakan pengganti yang sah.

### Layanan Windows (ServiceDll) {#windows_service_dll}

**Check ID:** `windows_service_dll` · **ATT&CK dalam katalog:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Yang diperiksa Juntei:** Membaca `Parameters\ServiceDll` di bawah layanan-layanan di control set SYSTEM yang aktif dan melampirkan bukti timestamp spesifik layanan bila tersedia.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`.

**Cara menginvestigasi:** Host bersama seperti `svchost.exe` bukanlah DLL payload. Periksa pustaka, akun layanan, status start, dan bukti instalasi layanan/image-load yang cocok; registrasi tidak membuktikan bahwa layanan tersebut berjalan.

**Detail lebih lanjut:** [Dokumentasi Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Layanan Windows (ImagePath) {#windows_service_image_path}

**Check ID:** `windows_service_image_path` · **ATT&CK dalam katalog:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Yang diperiksa Juntei:** Membaca perintah `ImagePath` layanan di control set aktif. Tipe driver 1, 2, atau 8 tanpa ImagePath mempertahankan target implisit `System32\drivers\<service>.sys`.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath`, `Type`, dan metadata layanan pendukung.

**Cara menginvestigasi:** Tinjau akun layanan dan pengaturan start secara terpisah dari label cakupan mesin pada keluaran. Registrasi yang dinonaktifkan atau demand-start tetap merupakan inventaris. Untuk timestamp, bukti berbasis nama layanan lebih diutamakan daripada path executable bersama yang ambigu.

**Detail lebih lanjut:** [Dokumentasi Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### DLL Startup Layanan Windows Update {#wu_service_startup_dll}

**Check ID:** `wu_service_startup_dll` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca `Parameters\ServiceDll` milik layanan Windows Update dari control set aktif.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**Cara menginvestigasi:** Bandingkan pustaka dengan build Windows dan status servicing terinstal yang sama. Pemeriksaan ini tumpang tindih dengan inventaris ServiceDll generik dan tidak merepresentasikan kompromi kedua yang terpisah.

## Scheduled task { #scheduled-tasks }

Task Scheduler menyimpan konfigurasi di berkas XML maupun di hive SOFTWARE. Baca sumber-sumber tersebut secara bersamaan. Sebuah task dapat terdaftar tanpa pernah dijalankan, dan koleksi yang tidak lengkap dapat menyerupai task tersembunyi atau ghost task.

### Ghost Scheduled Task {#ghost_task}

**Check ID:** `ghost_task` · **ATT&CK dalam katalog:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Yang diperiksa Juntei:** Membandingkan registrasi `Tree` dan `Tasks` TaskCache di SOFTWARE dan, bila berkas task ada di dalam koleksi, mencari berkas XML task yang bersesuaian.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` dan `TaskCache\Tasks`.
- Berkas `Windows\System32\Tasks\<task path>` yang diambil untuk perbandingan sistem berkas.

**Cara menginvestigasi:** Pasangan yang hilang dapat mencerminkan task yang dihapus/usang, registrasi bawaan, atau koleksi yang tidak lengkap. Tidak adanya XML yang dikumpulkan berarti perbandingan sistem berkas tidak tersedia. Periksa coverage dan ketidaksesuaian yang tepat sebelum menyebut sebuah task tersembunyi atau berbahaya.

### Scheduled Task Tersembunyi {#hidden_scheduled_task}

**Check ID:** `hidden_scheduled_task` · **ATT&CK dalam katalog:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Yang diperiksa Juntei:** Memeriksa registrasi Tree TaskCache untuk nilai `SD` yang hilang atau deskriptor keamanan biner yang kosong, yaitu kondisi yang terkait dengan penyembunyian task dari enumerasi biasa.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: keberadaan/data `SD`, beserta metadata pasangan TaskCache.

**Cara menginvestigasi:** SD yang hilang/kosong adalah anomali konfigurasi spesifik, bukan bukti bahwa task telah dieksekusi atau bahwa task tersebut berbahaya. Pengaturan XML `Hidden` dan masalah ACL sembarang adalah kondisi yang berbeda dan bukan predikat yang digunakan di sini.

**Detail lebih lanjut:** [Dokumentasi Microsoft](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/).

### Scheduled Task (Berkas XML) {#scheduled_task_files}

**Check ID:** `scheduled_task_files` · **ATT&CK dalam katalog:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Yang diperiksa Juntei:** Mengurai XML task yang diambil untuk perintah/argumen Exec serta class ID/data COM-handler. Pemeriksaan ini memilih principal berdasarkan `Actions Context`/`Principal id`, mempertahankan identitas pengguna dan grup secara terpisah, dan melaporkan SYSTEM, USER, atau UNKNOWN. Aksi COM yang tidak teresolusi tetap ditampilkan.

**Bukti yang diperiksa:**

- Berkas XML `Windows\System32\Tasks\<task path>`, termasuk field Exec, ComHandler, dan principal yang tersedia.
- `UsrClass.dat` dan NTUSER `Software\Classes` yang diambil milik pengguna yang cocok, ditambah `Classes\CLSID` SOFTWARE mesin, pada tampilan native dan 32-bit.
- `ProfileList` SOFTWARE yang diambil untuk principal berupa SID. Principal bernama mengutamakan folder `user.DOMAIN` yang cocok secara unik, lalu sufiks host lokal yang diambil, dan terakhir folder `user` biasa. Kecocokan dilabeli secara eksplisit sebagai heuristik; ambiguitas yang sebenarnya tetap tidak teresolusi dan dicatat di temuan dan coverage. Kunci registry volatil tidak diperlukan.
- `CodeBase` terkelola (termasuk subkunci versi), asosiasi AppID, dan `ImagePath` layanan SYSTEM untuk `LocalService` AppID.

Kandidat server COM menyertakan sumber registry-nya di `Description`. Asosiasi AppID yang hanya berupa nama
executable tetap tidak teresolusi alih-alih diberi direktori tebakan. URL lokal
`file:` didekode; URL jarak jauh dipertahankan tanpa akses jaringan. Override class pengguna
dibatasi pada profil tersebut, dan CLSID pengguna yang tidak lengkap tidak jatuh kembali ke registrasi
mesin yang dibayanginya. Task yang hanya berbasis grup mempertahankan kandidat mesin dengan keterbatasan
pengguna individual dicantumkan dalam deskripsinya, tanpa menambahkan pemberitahuan rutin ke ringkasan error coverage.

**Cara menginvestigasi:** Periksa pemicu, status aktif, principal, dan riwayat eksekusi di XML sumber dan log peristiwa. Task yang dibuat mungkin tidak pernah dijalankan. Handler COM yang tidak teresolusi tetap merupakan kandidat, dan data principal yang hilang/hanya berbasis grup tidak boleh diasumsikan berarti SYSTEM.

## WMI, BITS, dan pemicu peristiwa lainnya { #wmi-bits-and-other-event-triggers }

Windows Management Instrumentation (WMI) mendukung manajemen sistem. Background Intelligent Transfer Service (BITS) mendukung transfer di latar belakang. Keduanya dapat menyimpan perintah yang dipicu oleh peristiwa. Beberapa mekanisme menunggu peristiwa manajemen, notifikasi transfer, peluncuran aplikasi, atau tindakan pengguna. Pemicu yang tersimpan merupakan bukti konfigurasi yang berguna, tetapi keberadaannya tidak membuktikan bahwa pemicu tersebut telah aktif.

### Backdoor Fitur Aksesibilitas {#accessibility_tools}

**Check ID:** `accessibility_tools` · **ATT&CK dalam katalog:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Yang diperiksa Juntei:** Membandingkan tujuh binary aksesibilitas System32 yang dikumpulkan menggunakan PE OriginalFilename dan metadata sertifikat/katalog, setelah mensyaratkan adanya metadata katalog Microsoft yang dapat digunakan dari binary referensi yang dikumpulkan.

**Bukti yang diperiksa:**

- `Windows\System32\sethc.exe`, `osk.exe`, `narrator.exe`, `magnify.exe`, `utilman.exe`, `atbroker.exe`, dan `DisplaySwitch.exe`.
- Metadata referensi `Windows\System32\winlogon.exe`, `cmd.exe`, atau `notepad.exe` yang dikumpulkan serta katalog yang dikumpulkan yang berlaku.

**Cara menginvestigasi:** Metadata dapat mengindikasikan pengganti yang diganti namanya, tetapi Juntei tidak memvalidasi tanda tangan atau kepercayaan sertifikat. Koleksi yang minim tanpa katalog yang diperlukan menghasilkan coverage perbandingan yang tidak lengkap, alih-alih memperlakukan setiap berkas yang tampak tidak bertanda tangan sebagai backdoor.

### Fitur Aksesibilitas {#assistive_technology}

**Check ID:** `assistive_technology` · **ATT&CK dalam katalog:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Yang diperiksa Juntei:** Membaca perintah aksesibilitas terdaftar `StartExe` ditambah `StartParams` pada tampilan SOFTWARE native/32-bit, serta entri aksesibilitas `Configuration` yang aktif dari hive pengguna dan DEFAULT. Placeholder StartExe yang berupa angka/bukan string dilewati.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` dan `StartParams`; juga tampilan `Wow6432Node` mesin.
- `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` di NTUSER.DAT dan pengaturan yang setara di hive DEFAULT.

**Cara menginvestigasi:** Perangkat lunak aksesibilitas itu sah. Bedakan nama fitur yang aktif dari registrasi executable-nya yang sebenarnya, lalu periksa target dan bukti eksekusi sign-in/desktop. Hal ini terpisah dari penggantian executable aksesibilitas itu sendiri.

**Detail lebih lanjut:** [Referensi teknik (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/).

### Perintah Notifikasi Job BITS {#bits_notify_command}

**Check ID:** `bits_notify_command` · **ATT&CK dalam katalog:** [T1197](https://attack.mitre.org/techniques/T1197/)

**Yang diperiksa Juntei:** Memulihkan catatan job BITS berbentuk counted-string dari `qmgr.db`, `qmgr0.dat`, atau `qmgr1.dat` yang diambil, dengan mempertahankan program notifikasi yang tidak kosong, parameter, ID job, dan SID pemilik.

**Bukti yang diperiksa:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`, `qmgr0.dat`, dan `qmgr1.dat`.

**Cara menginvestigasi:** Ini adalah catatan hasil carving: status job yang dialokasikan/saat ini tidak divalidasi dan job yang usang/dihapus dapat muncul. Waktu tulis MFT basis data merupakan bukti kontainer yang lemah, bukan waktu eksekusi notifikasi. SID pemilik membedakan SYSTEM, local service, network service, dan pengguna.

### AutoRun Command Processor {#cmd_autorun}

**Check ID:** `cmd_autorun` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Membaca perintah `AutoRun` Command Processor mesin/pengguna, sebuah pengaturan yang dapat memengaruhi startup command prompt.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**Cara menginvestigasi:** Alat pengembangan dan kustomisasi shell dapat menjelaskan sebuah nilai. `cmd.exe /d` mem-bypass AutoRun, sehingga konfigurasi saja tidak membuktikan bahwa perintah tersebut berjalan setiap kali cmd.exe muncul. Periksa isi perintah dan argumen proses pemicunya.

**Detail lebih lanjut:** [Dokumentasi Microsoft](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd).

### Persistensi ErrorHandler.cmd {#error_handler_cmd}

**Check ID:** `error_handler_cmd` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Melaporkan keberadaan skrip `Windows\System32\ErrorHandler.cmd` yang dikumpulkan sebagai kandidat persistensi error-handler.

**Bukti yang diperiksa:**

- `Windows\System32\ErrorHandler.cmd`.

**Cara menginvestigasi:** Keberadaannya saja tidak membuktikan bahwa jalur pemulihan/error Windows yang berlaku telah memanggilnya. Baca skripnya, periksa asal-usulnya, dan korelasikan dengan bukti setup/error/proses.

### Flow Power Automate Desktop {#power_automate}

**Check ID:** `power_automate` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Menemukan direktori flow/skrip yang diambil di bawah penyimpanan `flows` dan `scripts` Power Automate Desktop milik setiap pengguna, dengan melaporkan setiap flow yang ditemukan satu kali.

**Bukti yang diperiksa:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` dan `scripts\<flow>\...`.

**Cara menginvestigasi:** Ini mengidentifikasi otomatisasi yang tersimpan, bukan setiap pemicu terjadwal atau flow yang telah dieksekusi. Periksa isi flow dan konfigurasi peluncuran eksternal sebelum memperlakukannya sebagai persistensi otomatis atau berbahaya.

### Profil PowerShell {#powershell_profiles}

**Check ID:** `powershell_profiles` · **ATT&CK dalam katalog:** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**Yang diperiksa Juntei:** Menemukan berkas bergaya profile.ps1 di direktori instalasi/profil Windows PowerShell dan PowerShell yang dikenali, termasuk Documents pengguna, OneDrive Documents yang ditemukan, dan pengalihan Documents yang dikumpulkan.

**Bukti yang diperiksa:**

- Berkas yang berakhiran `profile.ps1` langsung di `Windows\System32\WindowsPowerShell\v1.0` dan `Windows\SysWOW64\WindowsPowerShell\v1.0`.
- `Documents\WindowsPowerShell` dan `Documents\PowerShell` per pengguna, termasuk pengalihan Documents yang diambil dan folder OneDrive Documents yang ditemukan.
- Berkas yang berakhiran `profile.ps1` langsung di direktori versi di bawah `Program Files\PowerShell` atau `Program Files (x86)\PowerShell`.

**Cara menginvestigasi:** Profil dapat berisi kustomisasi yang tidak berbahaya. Baca skripnya dan telusuri payload yang dipanggil. Meluncurkan PowerShell dengan `-NoProfile` akan melewatinya; keberadaan berkas saja tidak membuktikan eksekusi profil atau konten berbahaya.

### Pembajakan Screensaver {#screensaver}

**Check ID:** `screensaver` · **ATT&CK dalam katalog:** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**Yang diperiksa Juntei:** Membaca nilai Desktop `SCRNSAVE.EXE` milik setiap pengguna, untuk mengidentifikasi image screensaver yang dikonfigurasi.

**Bukti yang diperiksa:**

- `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE` di NTUSER.DAT.

**Cara menginvestigasi:** Bandingkan dengan screensaver yang disetujui dan periksa targetnya. Pengaturan desktop lainnya, status sesi, dan timeout menentukan apakah screensaver dapat terpicu; path yang tersimpan tidak membuktikan bahwa screensaver telah dimulai.

### Perintah Telemetry Controller {#telemetry_controller}

**Check ID:** `telemetry_controller` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Membaca `Binary`, `Command`, dan `MaintenanceCommand` dari TelemetryController dan registrasi anak langsungnya.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` dan anak langsungnya: `Binary`, `Command`, `MaintenanceCommand`.

**Cara menginvestigasi:** Bandingkan executable dan argumen yang dipilih dengan build Windows dan perangkat lunak pemeliharaan yang sama. Perintah registry tidak menunjukkan bahwa pemicu pemeliharaan telah aktif.

### Profil Kustom Windows Terminal {#windows_terminal}

**Check ID:** `windows_terminal` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Mengurai pengaturan JSON/JSONC Windows Terminal versi packaged, preview, dan unpackaged, serta melaporkan pengaturan `commandline` profil/default yang tidak kosong.

**Bukti yang diperiksa:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` dan padanannya untuk WindowsTerminalPreview.
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: array profil lawas, `profiles.list`, dan field commandline `profiles.defaults`.

**Cara menginvestigasi:** Shell biasa tentu muncul di sini. Tentukan profil yang dipilih/default dan periksa perintah atau argumen peluncuran yang tidak biasa. Sekadar menyimpan profil tidak meluncurkannya.

### Langganan Peristiwa WMI {#wmi_event_subscription}

**Check ID:** `wmi_event_subscription` · **ATT&CK dalam katalog:** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**Yang diperiksa Juntei:** Melakukan carving terhadap instance event-consumer yang dikenali dan referensi binding filter-to-consumer dari repositori WMI OBJECTS.DATA yang diambil. Class yang dikenali mencakup consumer command-line, active-script, log-file, event-log, dan SMTP.

**Bukti yang diperiksa:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` dan `Repository\FS\OBJECTS.DATA`.

**Cara menginvestigasi:** Hit mentah di repositori mungkin usang, tidak ter-binding, atau historis; keluaran mempertahankan kualifikasi tersebut. Carving tidak merekonstruksi graf langganan aktif yang lengkap atau membuktikan bahwa pemicunya telah aktif. Waktu tulis berkas repositori bukanlah waktu pembuatan/eksekusi langganan.

## Explorer, COM, dan handler berkas atau URL { #explorer-com-and-file-or-url-handlers }

Component Object Model (COM) memungkinkan aplikasi mengaktifkan komponen perangkat lunak yang terdaftar. Shell Windows menggunakan handler terdaftar untuk membuka berkas, URL, folder, dan objek khusus. Pengenal COM (CLSID) mengidentifikasi komponen; ProgID memberikan nama yang mudah dibaca untuk sebagian komponen. Telusuri setiap registrasi hingga ke perintah atau DLL yang dipilihnya.

### Pembajakan App Paths {#app_paths}

**Check ID:** `app_paths` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Membaca nilai default di bawah registrasi `App Paths` tingkat mesin, yang dapat mengalihkan pencarian aplikasi untuk peluncuran berbasis shell.

**Bukti yang diperiksa:**

- Nilai default `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>`.

**Cara menginvestigasi:** Registrasi App Paths umum untuk aplikasi yang terinstal dan bukan penulisan ulang umum atas setiap peluncuran proses. Bandingkan nama dan image yang terdaftar dengan perangkat lunak yang diharapkan, lalu periksa bukti shell/proses yang relevan.

### Pembajakan Objek COM {#com_treat_as}

**Check ID:** `com_treat_as` · **ATT&CK dalam katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Yang diperiksa Juntei:** Menemukan pengalihan COM `TreatAs` dan registrasi InprocServer32/LocalServer32 per pengguna yang membayangi CLSID mesin, di seluruh penyimpanan class native/32-bit yang didukung. Target TreatAs diresolusikan menggunakan class milik pengguna yang sama dan class mesin.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` dan tampilan `Classes\Wow6432Node` mesin.
- `Software\Classes\CLSID` per pengguna di NTUSER.DAT dan `CLSID` di UsrClass.dat, termasuk tampilan `Wow6432Node` yang diimplementasikan: nilai default `TreatAs`, `InprocServer32`, dan `LocalServer32`.

**Cara menginvestigasi:** Override per pengguna dapat tidak memerlukan hak administrator, tetapi perangkat lunak yang sah juga dapat mendaftarkannya. Telusuri class dan DLL/executable yang dipilih, evaluasi arsitektur dan konteks aktivasinya, dan koroborasikan aktivasi COM yang sebenarnya secara independen.

### Pembajakan Handler Disk Cleanup {#disk_cleanup_handler}

**Check ID:** `disk_cleanup_handler` · **ATT&CK dalam katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Yang diperiksa Juntei:** Membaca CLSID handler VolumeCaches Disk Cleanup dan menelusuri registrasi COM InprocServer32 mesin yang dapat diresolusikan hingga ke pustakanya.

**Bukti yang diperiksa:**

- CLSID default `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` dan `Classes\CLSID\<CLSID>\InprocServer32`.

**Cara menginvestigasi:** Handler harus dipanggil oleh alur kerja pembersihan yang relevan agar dapat berjalan. Handler yang tidak teresolusi tidak dikeluarkan oleh pemeriksaan khusus ini, jadi gunakan parser registry untuk memeriksa registrasi mentahnya jika hasil pengambilan tidak lengkap.

### Override AppKey Explorer {#explorer_app_key}

**Check ID:** `explorer_app_key` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca nilai `ShellExecute` dan `Association` di bawah anak langsung `AppKey` Explorer, yang mencakup perintah/aksi yang dipilih oleh kunci aplikasi.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` dan `Association`.

**Cara menginvestigasi:** Utilitas keyboard dan alat OEM dapat mendefinisikan nilai-nilai ini. Nama asosiasi mungkin memerlukan pivot registry lain alih-alih berupa path berkas. Periksa tindakan pengguna yang akan memicunya.

### Browser Helper Objects {#explorer_bho}

**Check ID:** `explorer_bho` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Membaca registrasi CLSID Browser Helper Object dan meresolusikan pustaka InprocServer32 mesin bila tersedia; class ID yang tidak teresolusi tetap ditampilkan.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` dan nilai default `Classes\CLSID\<CLSID>\InprocServer32` mesin.

**Cara menginvestigasi:** Ini terkait dengan perilaku browser/Explorer yang kompatibel dan terutama relevan untuk lingkungan lawas. Ekstensi browser/keamanan yang disetujui dapat muncul. BHO yang terdaftar bukanlah bukti bahwa browser saat ini memuatnya.

### Pembajakan CLSID Desktop Explorer {#explorer_clsid_hijack}

**Check ID:** `explorer_clsid_hijack` · **ATT&CK dalam katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Yang diperiksa Juntei:** Membaca perintah shell-open yang dikonfigurasi pada tiga CLSID namespace Explorer khusus yang tercantum dari penyimpanan class mesin/pengguna.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**Cara menginvestigasi:** Bandingkan handler perintah untuk Desktop, My Computer, dan My Documents dengan baseline yang sesuai. Pemeriksaan ini memeriksa CLSID dan path tertentu, bukan setiap kemungkinan pembajakan namespace Explorer.

### Handler Menu Konteks Explorer {#explorer_context_menu}

**Check ID:** `explorer_context_menu` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Mengenumerasi handler menu konteks untuk Directory, Directory Background, Folder, Drive, AllFilesystemObjects, dan `*`, dengan menelusuri class ID ke registrasi DLL yang dikumpulkan bila memungkinkan.

**Bukti yang diperiksa:**

- Penyimpanan class mesin `HKLM\SOFTWARE\Classes` dan UsrClass.dat pengguna: nilai default `<subject>\shellex\ContextMenuHandlers\<handler>` dan registrasi pustaka COM yang bersesuaian.

**Cara menginvestigasi:** Ekstensi klik kanan dari alat arsip, produk keamanan, dan klien cloud adalah hal umum. Periksa handler yang sebenarnya dan interaksi pemicunya. GUID yang tidak teresolusi tetap merupakan petunjuk yang berguna, bukan bukti bahwa DLL tersebut tidak ada.

### Pembajakan Asosiasi Berkas {#file_association_hijack}

**Check ID:** `file_association_hijack` · **ATT&CK dalam katalog:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Yang diperiksa Juntei:** Memeriksa registrasi `shell\open\command` ekstensi langsung dan ProgID yang telah diresolusikan untuk `.txt`, `.pdf`, `.doc`, `.docx`, `.html`, `.htm`, `.js`, `.vbs`, `.hta`, `.exe`, `.bat`, `.cmd`, dan `.ps1`.

**Bukti yang diperiksa:**

- Penyimpanan class mesin `HKLM\SOFTWARE\Classes` dan UsrClass.dat pengguna: nilai default `<extension>` yang dipilih dan nilai default `<extension-or-ProgID>\shell\open\command`.

**Cara menginvestigasi:** Bandingkan dengan aplikasi yang diharapkan dan periksa argumen perintahnya. Ini adalah himpunan ekstensi yang ditentukan, bukan semua tipe berkas atau resolusi UserChoice efektif yang lengkap. Perintah yang tersimpan tidak membuktikan bahwa sebuah dokumen telah dibuka.

### Pembajakan Protocol Handler {#protocol_handler_hijack}

**Check ID:** `protocol_handler_hijack` · **ATT&CK dalam katalog:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Yang diperiksa Juntei:** Membaca perintah open URL untuk http, https, mailto, ms-msdt, dan ms-officecmd, ditambah registrasi `URL Protocol` yang ditemukan dan override pengguna yang berlaku.

**Bukti yang diperiksa:**

- Penyimpanan class mesin `HKLM\SOFTWARE\Classes` dan UsrClass.dat pengguna: penanda `URL Protocol` dan nilai default `<scheme>\shell\open\command`.

**Cara menginvestigasi:** Browser dan installer aplikasi secara sah mendaftarkan skema. Telusuri perintah dan argumennya, evaluasi tujuan yang diharapkan dari skema tersebut, dan korelasikan dengan bukti pembukaan URL/proses.

### Handler Ekstensi COM Recycle Bin {#recycle_bin_com_extension}

**Check ID:** `recycle_bin_com_extension` · **ATT&CK dalam katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Yang diperiksa Juntei:** Membaca override perintah open/empty/explore CLSID Recycle Bin yang tercantum serta registrasi handler context-menu/drag-drop langsung.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**Cara menginvestigasi:** Bandingkan dengan ekstensi shell yang disetujui. Sebagian nilai yang dilaporkan adalah pengenal COM, bukan pustaka yang teresolusi secara langsung; investigasi class yang direferensikan dan tindakan pengguna sebelum menyatakan adanya eksekusi kode.

### Pembajakan Search Protocol Handler {#search_protocol_handler}

**Check ID:** `search_protocol_handler` · **ATT&CK dalam katalog:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Yang diperiksa Juntei:** Membaca handler default `search-ms\shell\open\command` mesin/pengguna.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**Cara menginvestigasi:** Perubahan handler dapat mengubah apa yang terjadi saat URL search-ms dibuka, tetapi keberadaannya bukanlah bukti pengiriman exploit. Bandingkan dengan default Windows dan investigasi aktivitas URL/proses yang akan menggunakannya.

### SharedTaskScheduler {#shared_task_scheduler}

**Check ID:** `shared_task_scheduler` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Mengenumerasi nilai `SharedTaskScheduler` Explorer yang tidak kosong sebagai komponen shell yang terdaftar.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Nilai dapat mereferensikan class COM alih-alih path executable. Telusuri registrasi server-nya dan periksa perilaku sistem operasi, asal-usul ekstensi, dan bukti pemuatan shell yang relevan.

### ShellExecuteHooks {#shell_execute_hooks}

**Check ID:** `shell_execute_hooks` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Mengenumerasi nilai `ShellExecuteHooks` Explorer yang tidak kosong, yang mencakup registrasi hook shell-execution lawas.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Resolusikan CLSID apa pun yang direferensikan dan verifikasi apakah build dan kebijakan Windows mengizinkan hook tersebut. Registrasi lawas bisa saja tidak digunakan; nilai itu sendiri bukanlah eksekusi shell yang teramati.

### Persistensi COM Library dan Folder CLSID Start Menu {#start_menu_com}

**Check ID:** `start_menu_com` · **ATT&CK dalam katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Yang diperiksa Juntei:** Menemukan berkas XML library `.library-ms` yang URL `simpleLocation`-nya mereferensikan
CLSID COM, dan direktori bernama `<name>.{CLSID}`, di bawah
`Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu`. Interaksi Explorer dengan objek-objek
ini dapat mengaktifkan kode COM. Pemeriksaan ini mencakup direktori kosong yang diambil; pemeriksaan ini tidak mengikuti
symbolic link. Berkas library biasa di luar Start Menu dan GUID di dalam komentar XML tidak
dilaporkan oleh pemeriksaan ini.

**Bukti yang diperiksa:**

- Berkas `.library-ms` Start Menu per pengguna dan entri direktori yang dinamai dengan CLSID.
- Registrasi class yang diambil milik pengguna yang cocok dan SOFTWARE mesin, secara terpisah per arsitektur.
- Konfigurasi CodeBase/AppID/layanan terkelola melalui resolver yang sama dengan scheduled task.

**Cara menginvestigasi:** `Path` dan `SourceFile` mengidentifikasi artefak pemicu aktivasi. `Value`
berisi perintah server kandidat, atau CLSID jika resolusi tidak tersedia. `Description`
menautkan pemicu ke CLSID dan sumber registry yang diambil. `ResolvedPath`/`Sha256` hanya mendeskripsikan
payload yang teresolusi dan dikumpulkan. Asosiasi AppID yang hanya berupa nama executable tetap tidak teresolusi.
Pemicu bisa saja sah; keberadaannya tidak membuktikan adanya sign-in, aktivasi, eksekusi, atau serangan.
Periksa URL library, registrasi class, payload, konteks pengguna, dan log eksekusi independen.

**Peringatan pengumpulan:** Koleksi yang hanya berisi berkas dapat menghilangkan direktori kosong. Path Start Menu pengguna
standar diperiksa; Start Menu yang dialihkan dan lokasi namespace Explorer lainnya tidak dicakup
oleh pemeriksaan ini. Hive yang hilang atau registrasi yang tidak teresolusi menghasilkan coverage parsial sambil tetap mempertahankan
kandidat. Arsitektur task, elevasi, dan kebijakan dapat mengubah perilaku pencarian COM yang efektif.

**Detail lebih lanjut:** [Proposal asli dan kredit peneliti](https://github.com/last-byte/PersistenceSniper/pull/32),
[Tampilan gabungan COM per pengguna/mesin](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root),
[MITRE COM hijacking](https://attack.mitre.org/techniques/T1546/015/).

### Pembajakan COM TypeLib {#typelib_hijack}

**Check ID:** `typelib_hijack` · **ATT&CK dalam katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Yang diperiksa Juntei:** Membaca registrasi TypeLib GUID/versi/locale win32/win64 UsrClass.dat per pengguna, dengan mempertahankan path dan nilai script-moniker. Indeks resource numerik di bagian akhir hanya dihapus untuk keperluan resolusi berkas target.

**Bukti yang diperiksa:**

- Nilai default `TypeLib\<GUID>\<version>\<locale>\win32` dan `win64` di setiap UsrClass.dat yang diambil.

**Cara menginvestigasi:** Type library dapat menunjuk ke DLL yang sah, berkas alternatif, atau script moniker. Pemeriksaan ini menginventarisasi registrasi tersebut; evaluasi aplikasi yang terdampak dan koroborasikan pemuatan type library alih-alih berasumsi bahwa setiap path non-default adalah serangan.

## Aplikasi dan add-in Office { #office-applications-and-add-ins }

Office mendukung add-in, template, dan ekstensi khusus aplikasi yang sah. Registrasi yang mencurigakan dapat bergantung pada versi Office, aplikasi, pengaturan pemuatan, atau interaksi pengguna tertentu. Konfigurasi saja tidak membuktikan eksekusi makro.

### Registrasi Add-in Office {#office_addins}

**Check ID:** `office_addins` · **ATT&CK dalam katalog:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Yang diperiksa Juntei:** Mengenumerasi add-in Word, Excel, PowerPoint, Outlook, dan Access di root Office tanpa versi dan 14.0/15.0/16.0, termasuk penyimpanan Click-to-Run mesin dan root pengguna yang diimplementasikan. Pemeriksaan ini mempertahankan nilai Manifest/FileName/Path yang eksplisit atau meresolusikan entri berbasis COM yang memenuhi syarat, beserta konteks LoadBehavior.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` dan padanan Click-to-Run mesin yang diimplementasikan.
- `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` di NTUSER.DAT; class COM mesin/pengguna yang cocok dari SOFTWARE, NTUSER.DAT, dan UsrClass.dat.

**Cara menginvestigasi:** Add-in adalah perangkat lunak bisnis yang umum. Tinjau LoadBehavior, aplikasi/versi, manifest, payload yang sebenarnya, dan vendor yang diharapkan. Pemeriksaan ini tidak menjamin bahwa setiap format add-in atau tampilan registrasi yang terinstal telah dikumpulkan atau dimuat.

### Pembajakan Add-in AI Office {#office_ai_hijack}

**Check ID:** `office_ai_hijack` · **ATT&CK dalam katalog:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Yang diperiksa Juntei:** Mengenumerasi nilai-nilai di lokasi registrasi Common AI Click-to-Run Office 16.0.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: semua nilai.

**Cara menginvestigasi:** Bandingkan nilai dan path yang tidak biasa dengan build Office yang terinstal. Keberadaan sebuah nilai tidak menunjukkan fitur Office mana yang menggunakannya atau membuktikan bahwa nilai tersebut memilih kode yang dapat dieksekusi.

### Override DLL Office {#office_dll_override}

**Check ID:** `office_dll_override` · **ATT&CK dalam katalog:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Yang diperiksa Juntei:** Membaca `WwlibtDll` dan `PPCoreTDLL` di bawah root Word/PowerPoint untuk pengaturan tanpa versi dan 14.0/15.0/16.0, termasuk root Click-to-Run dan root pengguna yang diimplementasikan.

**Bukti yang diperiksa:**

- Pengaturan `Microsoft\Office\[<version>\]Word` dan `PowerPoint` mesin/pengguna, ditambah tree Office Click-to-Run mesin yang diimplementasikan: `WwlibtDll` dan `PPCoreTDLL`.

**Cara menginvestigasi:** Tentukan aplikasi/build mana yang mengenali override tersebut, periksa pustaka dan asal-usulnya, dan korelasikan dengan peluncuran Office atau image-load. Override yang dikonfigurasi saja bukanlah bukti eksekusi.

### Template Default Office {#office_templates}

**Check ID:** `office_templates` · **ATT&CK dalam katalog:** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**Yang diperiksa Juntei:** Menginventarisasi berkas Normal.dotm/NormalEmail.dotm yang dikumpulkan serta berkas yang langsung berada di folder Word Startup dan Excel XLSTART yang dikenali, termasuk folder startup instalasi Office.

**Bukti yang diperiksa:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` dan `NormalEmail.dotm`.
- Berkas langsung di `Microsoft\Word\STARTUP` dan `Microsoft\Excel\XLSTART` pengguna di bawah Roaming AppData.
- Berkas langsung di folder Office `<version>\STARTUP` / `XLSTART` dan `root\<version>\STARTUP` / `XLSTART` yang dikenali di bawah Program Files dan Program Files (x86).

**Cara menginvestigasi:** Template bisa saja sah atau tidak berisi konten yang dapat dieksekusi. Juntei tidak mengekstrak atau mengeksekusi makro dalam pemeriksaan ini. Periksa struktur internal dokumen, kebijakan makro, dan aktivitas aplikasi; path instalasi mesin tidak menyiratkan token proses SYSTEM.

### DLL Office Test {#office_test_dll}

**Check ID:** `office_test_dll` · **ATT&CK dalam katalog:** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**Yang diperiksa Juntei:** Mengenumerasi nilai-nilai di kunci Office Test Special Perf dari tampilan mesin/pengguna dan mesin 32-bit yang tercantum.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | Semua nilai yang tidak kosong |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | Semua nilai yang tidak kosong |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Perlakukan override yang tidak terduga sebagai petunjuk yang memerlukan validasi versi Office dan payload. Pemeriksaan ini melaporkan konfigurasi, bukan menguji apakah instalasi Office tertentu memuat nilai tersebut.

### Serangan Home Page Outlook {#outlook_home_page}

**Check ID:** `outlook_home_page` · **ATT&CK dalam katalog:** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**Yang diperiksa Juntei:** Membaca `URL` di bawah anak WebView Outlook pengguna untuk Office versi 14.0, 15.0, dan 16.0.

**Bukti yang diperiksa:**

- `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL` di NTUSER.DAT.

**Cara menginvestigasi:** Home page folder bisa saja sah dan perilakunya bervariasi menurut versi Outlook dan pembaruan keamanan. Periksa URL/konten dan kebijakan yang efektif, lalu korelasikan dengan bukti Outlook/web/proses.

### Pembajakan DLL VBA Monitor {#vba_monitors}

**Check ID:** `vba_monitors` · **ATT&CK dalam katalog:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Yang diperiksa Juntei:** Membaca nilai InprocServer32 untuk dua CLSID terkait VBA monitor yang diimplementasikan di penyimpanan class mesin dan per pengguna.

**Bukti yang diperiksa:**

- `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` dan `{0002E157-0000-0000-C000-000000000046}` di SOFTWARE mesin, ditambah registrasi UsrClass.dat pengguna yang setara.

**Cara menginvestigasi:** Bandingkan pustaka yang dipilih dengan lingkungan Office/VBA yang terinstal. Ini adalah pemeriksaan CLSID yang tertarget, bukan inventaris setiap ekstensi VBA atau bukti bahwa makro telah berjalan.

## Debugger, crash handler, dan compatibility shim { #debuggers-crash-handlers-and-compatibility-shims }

Debugger atau error handler dapat berjalan saat aplikasi dimulai, crash, hang, atau keluar. Basis data kompatibilitas aplikasi juga dapat memengaruhi startup. Investigasi pengaturan prasyarat dan program pemicunya bersama dengan perintah yang dilaporkan.

### Pembajakan Debugger AeDebug {#ae_debug}

**Check ID:** `ae_debug` · **ATT&CK dalam katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Yang diperiksa Juntei:** Membaca perintah `Debugger` AeDebug native dan 32-bit yang digunakan oleh konfigurasi debugging crash just-in-time.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**Cara menginvestigasi:** Perangkat lunak pengembangan/debugging dapat mengatur ini secara sah. Periksa perintah debugger lengkap dan prasyarat pemicu crash, lalu korelasikan dengan peristiwa application-error dan proses.

### Debugger Protected Process AeDebug {#ae_debug_protected}

**Check ID:** `ae_debug_protected` · **ATT&CK dalam katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Yang diperiksa Juntei:** Membaca perintah `Debugger` AeDebugProtected native dan 32-bit untuk permukaan debugging protected-process.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**Cara menginvestigasi:** Tentukan apakah build sistem operasi dan kebijakan proteksi dapat menggunakan perintah yang dikonfigurasi. Debugger yang tersimpan bukanlah bukti bahwa protected process mengalami crash atau bahwa perintah tersebut dijalankan.

### Debugger Image File Execution Options {#ifeo_debugger}

**Check ID:** `ifeo_debugger` · **ATT&CK dalam katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Yang diperiksa Juntei:** Mencari nilai `Debugger` di registrasi Image File Execution Options native/32-bit, termasuk kunci filter bersarang.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`, termasuk kunci filter bersarang dan tampilan `Wow6432Node` mesin.

**Cara menginvestigasi:** Debugger dapat dipilih ketika image yang disebutkan diluncurkan. Periksa cakupan image/filter dan argumen debugger; tooling pengembang juga dapat menggunakan IFEO. Telusuri peluncuran target untuk membuktikan eksekusi.

### NTDLL Terdelegasi IFEO {#ifeo_delegated_ntdll}

**Check ID:** `ifeo_delegated_ntdll` · **ATT&CK dalam katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Yang diperiksa Juntei:** Mencari override `VerifierDlls` di registrasi Image File Execution Options native/32-bit, termasuk kunci bersarang.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`, termasuk kunci bersarang dan tampilan `Wow6432Node` mesin.

**Cara menginvestigasi:** Predikat pemeriksaan ini adalah nilai VerifierDlls yang dikonfigurasi; pemeriksaan ini tidak membuktikan bahwa semua prasyarat verifier/delegasi telah diaktifkan. Tinjau flag yang berlaku, versi Windows, dan payload sebelum menyimpulkan adanya jalur injeksi yang berfungsi.

### Silent Process Exit Monitor {#ifeo_silent_process_exit}

**Check ID:** `ifeo_silent_process_exit` · **ATT&CK dalam katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Yang diperiksa Juntei:** Mencari `MonitorProcess` di registrasi SilentProcessExit native/32-bit, termasuk kunci bersarang.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`, termasuk kunci bersarang dan tampilan `Wow6432Node` mesin.

**Cara menginvestigasi:** Pemantauan silent-exit juga memerlukan konfigurasi pelaporan/IFEO yang relevan. Periksa pengaturan tersebut dan peristiwa keluarnya image yang dipantau; tooling diagnostik yang sah dapat menjelaskan monitor tersebut.

### Application Shimming {#installed_sdb}

**Check ID:** `installed_sdb` · **ATT&CK dalam katalog:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Yang diperiksa Juntei:** Membaca `DatabasePath` dari registrasi InstalledSDB langsung untuk basis data kompatibilitas aplikasi yang terinstal.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**Cara menginvestigasi:** Perbaikan kompatibilitas bisa saja sah. Periksa isi SDB, executable yang ditargetkan, dan riwayat instalasi. Registrasi basis data tidak mengidentifikasi shim-nya atau membuktikan bahwa aplikasi yang cocok telah dijalankan.

### Pembajakan Debugger LSM {#lsm_debugger}

**Check ID:** `lsm_debugger` · **ATT&CK dalam katalog:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Yang diperiksa Juntei:** Membaca `MonitorProcess` secara khusus di bawah SilentProcessExit untuk `lsm.exe` pada tampilan mesin native dan 32-bit.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**Cara menginvestigasi:** Pemeriksaan ini tumpang tindih dengan inventaris SilentProcessExit yang lebih luas. Konfirmasikan flag prasyarat dan keluarnya LSM yang relevan sebelum memperlakukan konfigurasi tersebut sebagai persistensi yang telah dieksekusi; hitung entri yang sama satu kali saat menilai temuan yang berbeda.

### DLL Auxiliary MiniDump {#minidump_auxiliary_dlls}

**Check ID:** `minidump_auxiliary_dlls` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Mengenumerasi nilai bernama yang tidak kosong di MiniDumpAuxiliaryDlls, dengan mempertahankan nama nilai sebagai konteks modul pemicu dan data nilai sebagai target yang dipilih.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: nama nilai modul pemicu dan data target.

**Cara menginvestigasi:** Periksa modul pemicu maupun DLL auxiliary. Tooling crash/minidump dapat menjelaskan sebuah entri, dan registrasi tidak menunjukkan bahwa operasi dump telah memuat pustaka tersebut.

### Application Shimming {#shim_custom}

**Check ID:** `shim_custom` · **ATT&CK dalam katalog:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Yang diperiksa Juntei:** Membaca nilai bernama `.sdb` di bawah kunci aplikasi Custom AppCompatFlags langsung, dengan mempertahankan asosiasi aplikasi-ke-basis data.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: nama nilai yang berakhiran `.sdb`.

**Cara menginvestigasi:** Pasangkan asosiasi ini dengan InstalledSDB dan basis data kompatibilitas yang sebenarnya. Asosiasi nama berkas saja tidak mengungkapkan perilaku basis data atau menunjukkan peluncuran aplikasi yang cocok.

### Pembajakan Debugger WER {#wer_debugger}

**Check ID:** `wer_debugger` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Membaca nilai `Debugger` Windows Error Reporting native/32-bit.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**Cara menginvestigasi:** Perangkat lunak diagnostik yang sah dapat mengonfigurasi debugger. Investigasi perintah lengkap dan peristiwa crash yang relevan; registrasi bukanlah bukti bahwa Windows Error Reporting menjalankannya.

### Pembajakan Debugger WER Hangs {#wer_hangs}

**Check ID:** `wer_hangs` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Membaca nilai `Debugger` Windows Error Reporting Hangs native/32-bit.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**Cara menginvestigasi:** Periksa apakah alur kerja penanganan hang akan memanggil pengaturan ini dan periksa program targetnya. Korelasikan dengan catatan hang aplikasi dan proses alih-alih memperlakukan penulisan registry sebagai eksekusi debugger.

### Pembajakan ReflectDebugger WER {#wer_reflect_debugger}

**Check ID:** `wer_reflect_debugger` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Membaca nilai `ReflectDebugger` Windows Error Reporting native/32-bit.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**Cara menginvestigasi:** Bandingkan dengan konfigurasi diagnostik yang disetujui dan tetapkan keberlakuan versi Windows/pemicunya. Periksa argumen perintah dan cari koroborasi crash/proses sebelum mengatribusikan eksekusi.

### Modul Runtime Exception WER {#wer_runtime_exception}

**Check ID:** `wer_runtime_exception` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Mengenumerasi nama nilai di bawah RuntimeExceptionHelperModules native/32-bit; di sini path pustaka dibawa dalam nama nilai, belum tentu dalam datanya.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` dan tampilan `Wow6432Node` mesin: nama nilai.

**Cara menginvestigasi:** Periksa modul yang disebutkan dan konteks aplikasi/runtime. Nilai numerik kecil di dalam data bukanlah path payload, dan registrasi saja tidak membuktikan bahwa helper tersebut telah dimuat.

## Runtime .NET dan pengaturan profiler { #net-runtime-and-profiler-settings }

Runtime terkelola dapat memuat profiler, assembly startup, atau pustaka debugging sebelum kode aplikasi. Pengaturan ini juga digunakan oleh pengembang dan produk pemantauan. Periksa runtime, arsitektur proses, flag pengaktifan, dan apakah proses yang baru dimulai dapat mewarisi pengaturan tersebut.

### Injeksi AppDomainManager {#appdomain_manager}

**Check ID:** `appdomain_manager` · **ATT&CK dalam katalog:** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**Yang diperiksa Juntei:** Membaca variabel lingkungan `APPDOMAIN_MANAGER_ASM` dan `APPDOMAIN_MANAGER_TYPE` mesin/pengguna, yang mencakup pemilihan assembly dan tipe untuk AppDomain manager .NET kustom.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**Cara menginvestigasi:** Aplikasi dapat dengan sengaja menggunakan manager kustom. Resolusikan assembly dan tipe secara bersamaan serta periksa kompatibilitas runtime dan pewarisan proses; baik satu variabel saja maupun timestamp kuncinya tidak membuktikan bahwa aplikasi terkelola telah memuatnya.

### Pembajakan Profiler .NET CLR {#cor_profiler}

**Check ID:** `cor_profiler` · **ATT&CK dalam katalog:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Yang diperiksa Juntei:** Membaca pengaturan lingkungan profiling COR mesin/pengguna dan entri bernama yang cocok di multi-string Environment masing-masing layanan. Pemeriksaan ini mempertahankan flag pengaktifan, pengenal profiler, dan path spesifik arsitektur.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` dan `Environment` di setiap NTUSER.DAT pengguna yang diambil.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: pengaturan COR bernama di dalam multi-string.

**Cara menginvestigasi:** Profiler mendukung pemantauan yang sah. Flag pengaktifan, CLSID, dan path DLL adalah bagian-bagian konfigurasi yang berbeda; konfirmasikan pengaktifan yang efektif dan proses .NET yang relevan alih-alih memperlakukan setiap baris sebagai profiler yang telah dieksekusi.

### Pembajakan Profiler .NET Core CLR {#coreclr_profiler}

**Check ID:** `coreclr_profiler` · **ATT&CK dalam katalog:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Yang diperiksa Juntei:** Membaca pengaturan lingkungan profiling CORECLR mesin/pengguna yang bersesuaian dan entri Environment layanan yang cocok, termasuk path spesifik arsitektur.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` dan `Environment` di setiap NTUSER.DAT pengguna yang diambil.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: pengaturan CORECLR bernama di dalam multi-string.

**Cara menginvestigasi:** Validasi runtime .NET, arsitektur, flag pengaktifan, dan pustaka. Produk observabilitas yang disetujui dapat mengonfigurasi pengaturan ini, dan proses yang sudah ada mungkin tidak mewarisi lingkungan yang baru diubah.

### Pembajakan .NET DbgManagedDebugger {#dotnet_dbg_managed_debugger}

**Check ID:** `dotnet_dbg_managed_debugger` · **ATT&CK dalam katalog:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Yang diperiksa Juntei:** Membaca pengaturan `DbgManagedDebugger` `.NETFramework` mesin/pengguna yang tercantum, termasuk path 32-bit mesin yang eksplisit.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**Cara menginvestigasi:** Tentukan apakah runtime .NET/kebijakan debugging yang berlaku menggunakan perintah ini. Tooling pengembang dapat menjelaskannya; korelasikan dengan crash terkelola dan pembuatan proses untuk membuktikan eksekusi.

### Kunci Registry COR_PROFILER .NET Framework {#dotnet_framework_profiler}

**Check ID:** `dotnet_framework_profiler` · **ATT&CK dalam katalog:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Yang diperiksa Juntei:** Membaca `COR_PROFILER`, `COR_PROFILER_PATH`, dan `COR_ENABLE_PROFILING` dari tampilan registry `.NETFramework` yang tercantum. Pemeriksaan ini melengkapi pemeriksaan profiler berbasis lingkungan.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**Cara menginvestigasi:** Evaluasi nilai-nilai tersebut secara bersamaan dan tentukan apakah runtime menghormati lokasi registry tersebut. Alat profiling yang dikonfigurasi bisa saja sah; jangan menginterpretasikan flag pengaktifan yang berdiri sendiri sebagai pemuatan DLL yang teridentifikasi.

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**Check ID:** `dotnet_startup_hooks` · **ATT&CK dalam katalog:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Yang diperiksa Juntei:** Membaca nilai lingkungan `DOTNET_STARTUP_HOOKS` mesin/pengguna, yang dapat memilih assembly startup-hook untuk runtime .NET yang didukung.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` dan `Environment\DOTNET_STARTUP_HOOKS` di setiap NTUSER.DAT pengguna.

**Cara menginvestigasi:** Periksa seluruh nilai untuk menemukan beberapa assembly dan verifikasi dukungan runtime. Pemeriksaan ini tidak mengenumerasi lingkungan startup-hook spesifik layanan atau membuktikan bahwa ada proses yang mewarisi dan menggunakan nilai tersebut.

### Known Managed Debugging DLLs {#known_managed_debugging_dlls}

**Check ID:** `known_managed_debugging_dlls` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca registrasi `KnownManagedDebuggingDlls` `.NETFramework` sebagai kemungkinan override pustaka managed-debugger.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**Cara menginvestigasi:** Bandingkan dengan tooling pengembangan/runtime yang terinstal dan tentukan debugger pemicunya. Entri registry tidak membuktikan adanya sesi debugger atau pemuatan pustaka.

## Override pencarian DLL dan executable { #dll-and-executable-lookup-overrides }

Pemeriksaan-pemeriksaan ini memeriksa lokasi pustaka eksplisit atau pengaturan yang memengaruhi cara sebuah komponen menemukan kode. Override path atau kebijakan pencarian yang dilemahkan merupakan petunjuk investigasi; payload dan komponen pemicu yang kompatibel tetap diperlukan untuk eksekusi.

### AppCert DLLs {#appcert_dlls}

**Check ID:** `appcert_dlls` · **ATT&CK dalam katalog:** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**Yang diperiksa Juntei:** Mengenumerasi registrasi AppCertDlls Session Manager yang tidak kosong, sebuah permukaan ekstensi pustaka yang terkait dengan API pembuatan proses.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | Semua nilai yang tidak kosong |

**Cara menginvestigasi:** Periksa setiap pustaka dan asal-usul instalasinya. Tentukan keberlakuan dan pemuatan saat runtime secara terpisah; pembuatan proses normal dalam timeline, dengan sendirinya, bukanlah bukti bahwa DLL yang dikonfigurasi telah dimuat.

### AppInit DLLs {#appinit_dlls}

**Check ID:** `appinit_dlls` · **ATT&CK dalam katalog:** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**Yang diperiksa Juntei:** Membaca `AppInit_DLLs` native/32-bit, memecah daftar pustakanya, dan menambahkan konteks `LoadAppInit_DLLs` dan `RequireSignedAppInit_DLLs` yang tersedia ke setiap kandidat.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs`, `LoadAppInit_DLLs`, dan `RequireSignedAppInit_DLLs`; juga tampilan `Wow6432Node` mesin.

**Cara menginvestigasi:** Flag LoadAppInit_DLLs bernilai nol secara eksplisit ditandai tidak aktif. Versi Windows, Secure Boot, dan persyaratan penandatanganan juga memengaruhi pemuatan. Tinjau pengaturan yang efektif sebelum menyatakan bahwa DLL AppInit telah dieksekusi.

**Detail lebih lanjut:** [Dokumentasi Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls).

### Override AutodialDLL {#autodial_dll}

**Check ID:** `autodial_dll` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca override `AutodialDLL` di Parameters WinSock2.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**Cara menginvestigasi:** Periksa fitur jaringan dan versi Windows sumber yang menggunakan pengaturan ini. Tinjau DLL yang sebenarnya dan aktivitas jaringan/proses; override saja tidak membuktikan adanya pemuatan yang aktif atau berbahaya.

### DLL Helper CHM {#chm_helper_dll}

**Check ID:** `chm_helper_dll` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca `Location` di bawah HtmlHelp Author, sebuah permukaan override pustaka/lokasi terkait bantuan (help).

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**Cara menginvestigasi:** Tentukan apakah komponen bantuan yang relevan menggunakan nilai tersebut pada sistem ini dan periksa berkas yang dipilih. Override registry bukanlah bukti bahwa dokumen CHM telah dibuka.

### Override DLL NL6 ContentIndex {#content_index_dll}

**Check ID:** `content_index_dll` · **ATT&CK dalam katalog:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Yang diperiksa Juntei:** Membaca `DLLOverridePath` dari registrasi ContentIndex Language langsung di control set aktif.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`.

**Cara menginvestigasi:** Komponen bahasa/pencarian dapat berbeda menurut build Windows. Periksa pustaka dan komponen terinstal yang cocok; telusuri proses pengindeksan yang sebenarnya untuk mengoroborasi pemuatan.

### DLL Crypto ExpoOffload {#crypto_expo_offload}

**Check ID:** `crypto_expo_offload` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca nilai `ExpoOffload` Cryptography Offload sebagai kandidat override pustaka kriptografi.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**Cara menginvestigasi:** Konfirmasikan keberlakuan fitur/build dan investigasi asal pustaka yang dipilih. Jangan perlakukan keberadaan nilai yang tidak umum sebagai bukti pencurian kredensial atau kode yang telah dieksekusi.

### DLL Software Rasterizer Direct3D {#direct3d_dll}

**Check ID:** `direct3d_dll` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca override pustaka `SoftwareRasterizer` Direct3D Drivers.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**Cara menginvestigasi:** Perangkat lunak grafis dan kompatibilitas dapat memengaruhi konfigurasi rasterizer. Periksa DLL dan aktivitas aplikasi Direct3D yang kompatibel; menyimpan path tidak membuktikan bahwa pustaka tersebut dimuat.

### Pelemahan Urutan Pencarian DLL {#dll_search_mode}

**Check ID:** `dll_search_mode` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Melaporkan `SafeDllSearchMode` atau `CWDIllegalInDllSearch` hanya ketika nilai Session Manager yang aktif terurai menjadi nol. Ini adalah kondisi kebijakan pencarian, bukan payload yang dipilih.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` dan `CWDIllegalInDllSearch`.

**Cara menginvestigasi:** Kebijakan pencarian yang dilemahkan dapat mempermudah serangan urutan pencarian yang terpisah. Hal ini tidak mengidentifikasi DLL yang ditanam atau membuktikan eksekusi. Nilai bukan nol atau yang tidak ada tidak dikeluarkan oleh pemeriksaan ini; evaluasi default dan kebijakan yang efektif secara terpisah.

**Detail lebih lanjut:** [Dokumentasi Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security).

### Pengecualian KnownDLLs {#exclude_from_known_dlls}

**Check ID:** `exclude_from_known_dlls` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca entri `ExcludeFromKnownDlls` yang tidak kosong dari Session Manager sebagai perubahan pada perilaku pencarian DLL yang dilindungi/dikenal.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`.

**Cara menginvestigasi:** Pengecualian mengidentifikasi kondisi pencarian, bukan pustaka pengganti itu sendiri. Temukan aplikasi yang terdampak dan lokasi pencarian DLL yang potensial, lalu koroborasikan adanya image-load yang tidak standar.

**Detail lebih lanjut:** [Dokumentasi Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order).

### Override DLL hhctrl.ocx {#hhctrl_ocx_dll}

**Check ID:** `hhctrl_ocx_dll` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca override InprocServer32 mesin native/32-bit dan penyimpanan class pengguna yang didukung untuk CLSID kontrol HTML Help yang diimplementasikan.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**Cara menginvestigasi:** Tentukan aplikasi dan arsitektur mana yang mengaktifkan kontrol tersebut. Periksa DLL yang terdaftar dan aktivitas terkait bantuan; registrasi COM tidak membuktikan bahwa HTML Help telah merender konten.

### Known DLLs {#known_dlls}

**Check ID:** `known_dlls` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Mengenumerasi nilai bernama di kunci KnownDLLs Session Manager yang aktif.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: nilai bernama.

**Cara menginvestigasi:** Banyak entri merupakan pustaka Windows yang diharapkan, dan sebagian nilai merupakan konfigurasi pendukung, bukan path payload. Bandingkan kunci lengkap dengan build tepercaya yang sesuai; pemeriksaan ini tidak memindai setiap tree KnownDLLs alternatif yang spesifik arsitektur.

### DLL Ekstensi LSA {#lsa_extensions}

**Check ID:** `lsa_extensions` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca `Extensions` dari konfigurasi LsaSrv LsaExtensionConfig.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**Cara menginvestigasi:** Perangkat lunak keamanan dapat memperluas komponen autentikasi. Periksa pustaka yang sebenarnya dan kompatibilitasnya dengan versi Windows sumber, lalu cari bukti image-load LSASS atau bukti penggunaan independen lainnya.

### Override Path DLL MAPI32 {#mapi32_dll_path}

**Check ID:** `mapi32_dll_path` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca `DLLPathEx` dan `DLLPath` di bawah registrasi klien Mail langsung pada tampilan mesin native dan 32-bit.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**Cara menginvestigasi:** Klien email mendaftarkan provider MAPI yang sah. Tentukan klien/provider mana yang dipilih dan periksa pustaka yang sebenarnya; provider yang terinstal tidak membuktikan bahwa sebuah aplikasi menggunakan MAPI selama insiden.

### DLL XA MSDTC {#msdtc_xa_dll}

**Check ID:** `msdtc_xa_dll` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca `OracleXaLib` dan `OracleOciLib` dari konfigurasi MTxOCI MSDTC.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**Cara menginvestigasi:** Integrasi basis data dapat secara sah memerlukan pustaka-pustaka ini. Verifikasi konfigurasi transaction coordinator dan perangkat lunak basis data yang terinstal, dan korelasikan dengan penggunaan layanan/modul alih-alih berasumsi bahwa path yang dikonfigurasi telah dijalankan.

### Override DLL NLDP {#nldp_dll}

**Check ID:** `nldp_dll` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca nilai konfigurasi `DllOverridePath` NlsData.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**Cara menginvestigasi:** Tinjau pustaka target, build Windows, dan komponen bahasa alami yang dapat menggunakan override tersebut. Nilai ini tidak menunjukkan bahwa pengindeksan atau pemrosesan bahasa telah terjadi.

### DLL TestDVCPlugin RDP {#rdp_test_dvc_plugin}

**Check ID:** `rdp_test_dvc_plugin` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca pengaturan pustaka `TestDVCPlugin` Terminal Server Client tingkat mesin.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**Cara menginvestigasi:** Investigasi plugin uji yang tidak dikenal dan perilaku spesifik versinya. Sesi klien yang kompatibel dan pemuatan pustaka diperlukan untuk membuktikan eksekusi; nilai registry saja tidak menunjukkan keduanya.

### Override DLL Search Indexer {#search_indexer_dll}

**Check ID:** `search_indexer_dll` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca override `DllPath` Windows Search dari SOFTWARE.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**Cara menginvestigasi:** Tentukan komponen/versi Search yang sesuai dan periksa berkas targetnya. Layanan Windows Search yang tidak aktif atau pengaturan yang tidak digunakan berbeda dengan pemuatan pustaka berbahaya yang teramati.

### DLL AutoProxy Winsock {#winsock_auto_proxy}

**Check ID:** `winsock_auto_proxy` · **ATT&CK dalam katalog:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Yang diperiksa Juntei:** Membaca `LibraryPath` di bawah registrasi Catalog_Entries dan Catalog_Entries64 NameSpace_Catalog5 langsung.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**Cara menginvestigasi:** Provider jaringan bisa saja sah. Tinjau kedua arsitektur, asal-usul provider, dan bukti aplikasi jaringan/image-load yang relevan. Pencarian namespace provider yang tertarget ini tidak mencakup setiap kemungkinan ekstensi Winsock.

## Akun dan visibilitas sign-in { #accounts-and-sign-in-visibility }

Catatan akun lokal dan pengaturan visibilitas sign-in dapat mengungkap persistensi berbasis identitas. Pemeriksaan-pemeriksaan ini memeriksa hive SAM dan SOFTWARE yang diambil; pemeriksaan ini bukan inventaris akun Active Directory, keanggotaan grup domain, atau logon di masa lalu.

### Keanggotaan Grup Administrators {#admin_group_membership}

**Check ID:** `admin_group_membership` · **ATT&CK dalam katalog:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Yang diperiksa Juntei:** Mendekode daftar anggota alias Administrators lokal dari SAM. Pemeriksaan ini melaporkan SID mesin lokal yang menunjuk akun tersembunyi atau tidak menunjuk akun/grup apa pun di SAM yang diambil.

**Bukti yang diperiksa:**

- `SAM\Domains\Builtin\Aliases\00000220\C` serta SID mesin dan catatan pengguna/grup di bawah `SAM\Domains\Account` di hive SAM.
- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList` untuk korelasi nama yang disembunyikan.

**Cara menginvestigasi:** Ini adalah pemeriksaan anomali yang tertarget, bukan daftar semua administrator. SID eksternal/domain tidak dievaluasi sebagai pengguna lokal yang hilang. Akun yang dihapus, waktu pengambilan, atau hive yang tidak lengkap memerlukan investigasi sebelum menyimpulkan keanggotaan yang berbahaya.

### Akun Lokal Tersembunyi {#hidden_account}

**Check ID:** `hidden_account` · **ATT&CK dalam katalog:** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**Yang diperiksa Juntei:** Melaporkan nilai numerik nol di UserList SpecialAccounts Winlogon, yang mengidentifikasi nama-nama yang disembunyikan dari layar sign-in biasa.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: nilai nama akun.

**Cara menginvestigasi:** Akun layanan atau pemeliharaan dapat disembunyikan dengan sengaja, dan nilai ini tidak membuktikan bahwa akun yang disebutkan ada atau merupakan administrator. Gabungkan dengan catatan akun SAM dan aktivitas logon.

### Pembajakan RID {#rid_hijacking}

**Check ID:** `rid_hijacking` · **ATT&CK dalam katalog:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Yang diperiksa Juntei:** Membandingkan RID heksadesimal subkunci akun SAM dengan RID yang disimpan pada offset 0x30 di nilai F-nya dan melaporkan ketidakcocokan.

**Bukti yang diperiksa:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` di hive SAM.

**Cara menginvestigasi:** Ketidakcocokan merupakan anomali catatan akun yang serius tetapi juga dapat memerlukan analisis kerusakan/pengambilan. Periksa byte sumber, status log transaksi, dan catatan akun/logon; hal ini tidak membuktikan aktor mana yang memodifikasi catatan tersebut.

### RID Suborner (Akun Admin Tersembunyi) {#rid_suborner}

**Check ID:** `rid_suborner` · **ATT&CK dalam katalog:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Yang diperiksa Juntei:** Melaporkan akun SAM yang RID subkuncinya bukan 500 tetapi RID pada nilai F-nya adalah 500, yaitu pola substitusi identitas administrator yang lebih spesifik.

**Bukti yang diperiksa:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` di hive SAM.

**Cara menginvestigasi:** Pemeriksaan ini tumpang tindih dengan pemeriksaan ketidakcocokan RID umum. Meskipun judul katalognya demikian, pemeriksaan ini tidak secara independen mensyaratkan pengaturan sign-in tersembunyi; konfirmasikan visibilitas akun, hak akses, dan penggunaan akun yang sebenarnya secara terpisah.

## Komponen autentikasi dan proteksi kredensial { #authentication-components-and-credential-protection }

Local Security Authority (LSA) menangani fungsi-fungsi keamanan melalui LSASS dan komponen terkait. Authentication package, password filter, dan network provider dapat berupa perangkat lunak keamanan yang sah atau jalur menuju akses kredensial yang persisten. Dua pemeriksaan menginventarisasi pengaturan proteksi, bukan autostart executable. Interpretasikan keduanya menggunakan peran host, versi Windows, dan kebijakan yang efektif.

### DLL Provider AMSI {#amsi_providers}

**Check ID:** `amsi_providers` · **ATT&CK dalam katalog:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Yang diperiksa Juntei:** Mengenumerasi CLSID provider AMSI mesin dan meresolusikan pustaka InprocServer32 native/32-bit-nya bila tersedia. Registrasi yang tidak teresolusi mempertahankan class ID-nya.

**Bukti yang diperiksa:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` dan registrasi InprocServer32 `Classes\CLSID` / `Classes\Wow6432Node\CLSID` yang dapat diresolusikan.

**Cara menginvestigasi:** Antivirus dan produk keamanan secara sah mendaftarkan provider AMSI. Tinjau asal-usul provider/pustaka dan pemuatan yang sebenarnya; registrasi tidak membuktikan bahwa AMSI telah di-bypass atau bahwa provider tersebut berbahaya.

### Authentication Packages {#authentication_packages}

**Check ID:** `authentication_packages` · **ATT&CK dalam katalog:** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**Yang diperiksa Juntei:** Membaca setiap entri `Authentication Packages` LSA yang tidak kosong dari control set aktif.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**Cara menginvestigasi:** Bandingkan dengan build Windows sumber dan perangkat lunak autentikasi yang terinstal. Package non-default bisa saja sah. Resolusikan DLL-nya dan koroborasikan aktivitas pemuatan/kredensial sebelum menyimpulkan adanya intersepsi.

### Perilaku Logon Admin DSRM {#dsrm_backdoor}

**Check ID:** `dsrm_backdoor` · **ATT&CK dalam katalog:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Yang diperiksa Juntei:** Melaporkan `DsrmAdminLogonBehavior=2` numerik di lokasi ControlSet001, ControlSet002, atau CurrentControlSet yang diimplementasikan. Pengaturan ini dapat mengizinkan penggunaan akun DSRM di luar konteks pemulihannya yang biasa.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior`, ditambah ControlSet002 dan CurrentControlSet bila ada.

**Cara menginvestigasi:** Terutama relevan untuk domain controller. Tentukan control set mana yang aktif menggunakan Select di SYSTEM; kecocokan pada control set historis belum tentu efektif. Nilai ini tidak membuktikan bahwa penyerang mengetahui kata sandi DSRM atau telah melakukan logon.

### Konfigurasi Credential Guard {#lsa_cfg_flags}

**Check ID:** `lsa_cfg_flags` · **ATT&CK dalam katalog:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Yang diperiksa Juntei:** Membaca nilai `LsaCfgFlags` LSA yang dikonfigurasi, termasuk nilai aktif biasa maupun nol. Pemeriksaan ini menginventarisasi konfigurasi Credential Guard, bukan autostart executable.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**Cara menginvestigasi:** Interpretasikan nilai ini bersama versi Windows, prasyarat firmware/keamanan, dan kebijakan yang efektif. Nilai nol yang tersimpan atau nilai yang tidak ada, dengan sendirinya, tidak membuktikan adanya perubahan oleh penyerang, status proteksi yang efektif, atau pencurian kredensial.

### Password Filter LSA {#lsa_password_filter}

**Check ID:** `lsa_password_filter` · **ATT&CK dalam katalog:** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**Yang diperiksa Juntei:** Membaca `Notification Packages` LSA. Nama pustaka tanpa path diresolusikan terhadap direktori Windows System32 yang diambil, dengan menambahkan `.dll` bila sesuai.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**Cara menginvestigasi:** Produk kebijakan kata sandi umumnya menginstal filter. Periksa pustaka dan bukti pemuatannya, serta bedakan filter yang dikonfigurasi dari bukti bahwa perubahan kata sandi atau intersepsi kredensial telah terjadi.

### Status Proteksi PPL LSASS {#lsa_run_as_ppl}

**Check ID:** `lsa_run_as_ppl` · **ATT&CK dalam katalog:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Yang diperiksa Juntei:** Membaca nilai `RunAsPPL` LSA yang dikonfigurasi, termasuk nilai bukan nol. Ini adalah inventaris pengaturan proteksi, bukan pengujian bahwa LSASS berjalan tanpa proteksi.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**Cara menginvestigasi:** Proteksi LSA yang efektif bergantung pada versi Windows, kebijakan, penguncian firmware, dan status restart. Tidak adanya baris untuk nilai yang tidak ada bukanlah hasil proteksi yang terukur. Gunakan bukti sistem/runtime yang independen untuk menilai proteksi yang efektif.

### DLL Network Provider {#network_provider_dll}

**Check ID:** `network_provider_dll` · **ATT&CK dalam katalog:** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**Yang diperiksa Juntei:** Membaca `NetworkProvider\ProviderPath` di bawah layanan-layanan di control set SYSTEM yang aktif.

**Bukti yang diperiksa:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`.

**Cara menginvestigasi:** Provider logon jaringan bisa saja sah. Periksa urutan/keberlakuan provider, perangkat lunak jaringan/keamanan yang terinstal, dan pemuatan pustaka yang sebenarnya; nilai ProviderPath saja tidak membuktikan adanya intersepsi kredensial.

### Security Packages {#security_packages}

**Check ID:** `security_packages` · **ATT&CK dalam katalog:** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**Yang diperiksa Juntei:** Membaca setiap entri `Security Packages` LSA yang tidak kosong, yang mencakup security support provider yang dikonfigurasi.

**Bukti yang diperiksa:**

| Kunci registry | Nilai yang diperiksa |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**Cara menginvestigasi:** Bandingkan dengan baseline Windows/produk keamanan yang diharapkan dan periksa setiap pustaka. Konfigurasi ini tidak membuktikan bahwa sebuah provider telah dimuat, menangkap kredensial, atau ditambahkan selama insiden.

## Dari kandidat menuju kesimpulan { #work-from-a-candidate-to-a-conclusion }

1. Buka sumber asli yang diidentifikasi oleh `SourceFile`. Konfirmasikan nilai lengkap, aksi XML,
   skrip, atau catatan basis data yang dipulihkan, serta pengguna/host tepat pemiliknya.
2. Tentukan apakah mekanisme tersebut dapat beroperasi: periksa flag pengaktifan, versi runtime atau Windows,
   status layanan/task, pemicu yang diperlukan, pengguna, arsitektur, dan control set aktif.
3. Periksa program, DLL, skrip, atau assembly yang dipilih. Hitung hash byte yang dikumpulkan dan tinjau argumen
   serta kontennya. Launcher Windows atau string penerbit yang dikenali tidak menentukan keabsahannya.
4. Tetapkan apa yang sebenarnya terjadi menggunakan bukti independen. Contohnya meliputi pembuatan proses,
   image load Sysmon, catatan runtime Task Scheduler, aktivitas layanan, Prefetch, dan berkas terkait.
   Peristiwa perubahan konfigurasi dapat membuktikan bahwa pengaturan telah berubah tanpa membuktikan bahwa payload-nya telah berjalan.
5. Bangun timeline menggunakan timestamp dari sumber asli dan asal-usul korelasi yang tercatat.
   Perlakukan penulisan kunci, perubahan instalasi, penulisan berkas/kontainer, dan eksekusi sebagai fakta-fakta yang terpisah.
6. Catat keterbatasan sekaligus temuan: hive yang hilang, berkas target yang hilang, principal task yang tidak diketahui,
   metadata penanda tangan yang belum diverifikasi, catatan historis hasil carving, atau pemicu yang tidak dapat dikoroborasi.

Untuk sintaks perintah, profil, setiap field keluaran, dan status coverage, kembali ke
[panduan perintah Persistence Hunter](../commands/persistence-hunter.md). Halaman
[panduan artefak](../artifacts/index.md) menjelaskan sumber-sumber yang mendasarinya, dan halaman
[ucapan terima kasih](acknowledgements.md) mengidentifikasi alat persistensi upstream.
