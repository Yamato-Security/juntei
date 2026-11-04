# Cache Penjadwal Tugas { #task-scheduler-cache }

## Apa itu TaskCache { #what-taskcache-is }

Windows Task Scheduler menyimpan cache registry di bawah `SOFTWARE` yang menautkan path tugas dengan GUID,
trigger/daftar, principal, tindakan (action), metadata runtime, dan security descriptor. Juntei mendekode
tindakan eksekusi dan COM handler beserta tindakan email/pesan lawas, timestamp tugas, dan tanda-tanda
tugas yang disembunyikan dari antarmuka Task Scheduler biasa.

Tugas terjadwal adalah mekanisme persistensi dan eksekusi jarak jauh yang umum. Cache registry juga bernilai
ketika penyerang menghapus atau merusak berkas XML tugas.

<span class="evidence-strength">Kesimpulan terbaik:</span> TaskCache mengonfigurasi atau mempertahankan
tugas/tindakan tersebut. Field runtime dan log independen diperlukan untuk membuktikan eksekusi tertentu.

## Lokasi { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

Kumpulkan `SOFTWARE` beserta log transaksinya dan seluruh pohon `System32\Tasks`. Log peristiwa Task Scheduler
Operational dan Security menambahkan bukti pembuatan/pembaruan/runtime.

## Peringatan bagi analis { #analyst-cautions }

- Cache registry dan XML tugas dapat saling bertentangan setelah perusakan, penghapusan, registrasi yang tidak lengkap, atau kerusakan.
- `Hidden` milik Juntei mengidentifikasi penyembunyian struktural (keterkaitan Tree/security descriptor yang hilang dalam
  kondisi yang relevan), bukan sekadar tugas yang dikonfigurasi dengan flag UI “Hidden” biasa.
- Windows 7 menyimpan tindakan tugas di XML, bukan di nilai `Actions` milik cache; tindakan yang kosong
  tidak otomatis mencurigakan di sana.
- `LastRun` dan `LastResult` berguna tetapi bukan riwayat eksekusi yang lengkap dan tahan manipulasi.
- `Context` menyebutkan principal/konteks yang dikonfigurasi, bukan pengguna yang mendaftarkan tugas tersebut.
- CLSID COM handler memerlukan resolusi registry/berkas secara terpisah.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath` | Konteks host dan hive `SOFTWARE` sumber. |
| `TaskName` | Path scheduler lengkap. Pivot utama ke XML dan log peristiwa. |
| `TaskGUID` | GUID yang menggabungkan entri cache Tree dan Tasks. Kunci korelasi yang kuat. |
| `TaskIndex` | Metadata Tree/indeks yang digunakan oleh scheduler; bersifat kontekstual, bukan hitungan eksekusi. |
| `TaskLists` | Daftar cache (`Boot`, `Logon`, `Plain`, `Maintenance`) yang mengategorikan pemicuan/penanganan. |
| `Hidden` | Indikator tugas tersembunyi secara struktural. Tinjau `InTree`, `InTasks`, dan status descriptor sebelum menyimpulkan adanya perusakan. |
| `InTree`, `InTasks` | Apakah kedua sisi cache ada. Keterkaitan yang hilang dapat mengindikasikan penghapusan/perusakan/registrasi yang tidak lengkap. |
| `HasSecurityDescriptor` | Apakah entri Tree memiliki `SD`; ketiadaannya dapat berkontribusi pada perilaku tugas tersembunyi pada versi yang relevan. |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | Perubahan tingkat kunci untuk setiap sisi cache; bukan waktu pembuatan/eksekusi yang persis. |
| `ActionIndex` | Tindakan di dalam tugas, berbasis nol. Satu tugas dapat menghasilkan beberapa baris. |
| `ActionType` | `exec`, `com_handler`, `send_email`, atau `show_message`. |
| `Executable`, `Args`, `WorkingDirectory` | Tindakan eksekusi efektif. Interpretasikan tanda kutip, variabel, path relatif, dan LOLBin. |
| `COM-ClassID`, `ActionData` | CLSID dan data COM handler, atau payload email/pesan lawas. Resolusikan CLSID secara terpisah. |
| `Context` | Konteks principal yang dienkode bersama tindakan (`LocalSystem`, `Author`, dll.); identitas yang dikonfigurasi, bukan pendaftar. |
| `Author`, `URI`, `Description` | Metadata tugas. Mudah dikendalikan penyerang dan bukan bukti kepercayaan. |
| `LastRegisteredUTC` | Waktu registrasi/pembaruan DynamicInfo bila ada. |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | Metadata runtime/hasil yang di-cache. Korelasikan dengan log operasional dan bukti proses. |
| `ActionsRaw` | Byte tindakan asli jika dekode tidak lengkap atau validasi diperlukan. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemulihan registry. |
| `Problems` | Ketidakcocokan path/GUID, tipe nilai yang salah, struktur tindakan yang terpotong, atau bagian yang hilang. |

## Strategi investigasi { #investigation-strategy }

1. Tinjau tugas yang tersembunyi/tidak konsisten dan path non-Microsoft terlebih dahulu.
2. Rekonstruksi perintah efektif, termasuk argumen, direktori kerja, ekspansi variabel lingkungan, dan resolusi COM.
3. Bandingkan catatan registry dengan XML tugas dan lakukan hash/periksa timestamp executable yang dirujuk.
4. Korelasikan `TaskName`/GUID/waktu dengan peristiwa Task Scheduler, Security 4698/4702, log proses, Prefetch, dan MFT/USN.
5. Telusuri data registry yang terhapus/dipulihkan bila penyerang menghapus tugas tersebut.

## Detail lebih lanjut { #further-details }

- [Microsoft: Task Scheduler schema](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: Task Scheduler event log](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
