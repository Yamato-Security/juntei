# Layanan Windows { #windows-services }

## Apa yang dicatat oleh konfigurasi layanan { #what-service-configuration-records }

Layanan Windows dan driver kernel dikonfigurasi di bawah setiap control set `SYSTEM`. Catatannya menyebutkan
executable/driver atau DLL layanan, mode start, tipe layanan, akun, grup urutan pemuatan, start
tertunda, perilaku saat terjadi error, dan tindakan yang dilakukan Windows setelah kegagalan.

Penyerang membuat atau memodifikasi layanan untuk persistensi, eskalasi hak akses, pemuatan driver, dan
pergerakan lateral. Perangkat lunak yang sah menggunakan mekanisme yang persis sama, sehingga path, akun, penandatanganan,
konteks pembuatan, dan bukti saat runtime menjadi penting.

<span class="evidence-strength">Kesimpulan terbaik:</span> control set yang diurai mengonfigurasi
layanan/driver seperti yang ditampilkan. Konfigurasi saja tidak membuktikan bahwa layanan berhasil dijalankan.

## Lokasi { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

Kumpulkan `SYSTEM` beserta log transaksinya. Event ID seperti System 7045 dan Security 4697 dapat
mencatat instalasi; peristiwa service-control dan proses dapat mendukung adanya start/eksekusi yang sebenarnya.

## Peringatan bagi analis { #analyst-cautions }

- `ImagePath` dapat berisi variabel lingkungan, tanda kutip, argumen, path perangkat, atau path tanpa tanda kutip
  yang mengandung spasi. Pertahankan nilainya persis apa adanya sebelum dinormalisasi.
- `ServiceDll` sangat penting untuk layanan yang di-host oleh `svchost.exe`; `ImagePath` saja mungkin hanya menyebutkan host-nya.
- `StartType=automatic` tidak membuktikan bahwa layanan dijalankan atau tetap aktif.
- `ObjectName` adalah akun yang dikonfigurasi, belum tentu pengguna yang menginstal layanan tersebut.
- Driver dan layanan mode pengguna menggunakan flag `ServiceType` dan konvensi path yang berbeda.
- Tindakan kegagalan dapat mengeksekusi perintah sebagai akun layanan setelah terjadi kegagalan; ini adalah permukaan
  persistensi yang mudah terlewat.
- Control set lain mungkin menyimpan konfigurasi sebelumnya; gunakan `IsCurrentControlSet`.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath` | Identitas sistem dan hive masukan. Pertahankan di seluruh koleksi. |
| `ControlSet`, `IsCurrentControlSet` | Control set sumber dan apakah `Select\Current` menunjuknya. Utamakan yang current untuk klaim tentang status aktif. |
| `ServiceName` | Nama kunci registry/nama internal layanan. Pivot yang kuat ke peristiwa dan perintah Service Control Manager. |
| `DisplayName`, `Description` | Teks yang ditampilkan kepada pengguna; dapat berupa referensi resource atau dikendalikan penyerang. |
| `ImagePath` | Perintah executable/driver sebagaimana dikonfigurasi. Urai tanda kutip, argumen, variabel lingkungan, dan path perangkat. |
| `ServiceDLL` | DLL yang dimuat oleh layanan host. Sering kali merupakan payload sebenarnya untuk layanan shared-process. |
| `StartType`, `StartValue` | Mode start dalam bentuk mudah dibaca/mentah: boot (0), system (1), automatic (2), manual (3), disabled (4). |
| `DelayedAutostart` | Layanan otomatis dijalankan setelah fase boot awal; tetap dapat digunakan untuk persistensi. |
| `ServiceType`, `TypeValue` | Flag yang didekode/mentah: kernel driver, file-system driver, own/share process, user service, dll. |
| `ErrorControl` | Tingkat penanganan error saat boot/start. Terutama sebagai konteks operasional. |
| `ObjectName` | Akun layanan yang dikonfigurasi atau nama objek driver. Bukan identitas penginstal. |
| `Group` | Grup urutan pemuatan, terutama penting untuk driver boot/system. |
| `FailureResetPeriod` | Detik sebelum hitungan kegagalan direset; `4294967295` berarti tidak pernah. |
| `FailureCommand` | Perintah yang dikonfigurasi untuk tindakan kegagalan `run_command`. Field persistensi bernilai tinggi. |
| `FailureActions` | Daftar tindakan lengkap yang telah didekode (`restart`, `reboot`, `run_command`, `none`) beserta jedanya. |
| `FailureActionType`, `FailureActionDelayMs` | Tindakan pertama yang diratakan untuk pemfilteran; periksa juga daftar lengkapnya. |
| `FailureActionsRaw` | Byte asli bila dekode tidak lengkap/rusak. |
| `KeyLastWrittenUTC` | Perubahan terakhir pada kunci layanan, belum tentu waktu instalasi/start. |
| `TransactionLogStatus`, `LogsApplied` | Konteks replay registry. |
| `Problems` | Ketidakcocokan tipe, struktur kegagalan yang cacat, atau field yang hilang/tidak dapat dibaca. |

## Strategi investigasi { #investigation-strategy }

1. Tinjau path yang tidak standar, direktori yang dapat ditulisi, akun yang tidak biasa, waktu kunci yang baru, dan perintah kegagalan.
2. Ekspansikan variabel lingkungan menggunakan konfigurasi host yang terdampak, bukan mesin analis.
3. Periksa hash/tanda tangan executable atau DLL yang sebenarnya jika berhasil dikumpulkan.
4. Korelasikan nama/path layanan dengan Event 7045/4697, pembuatan proses, Prefetch, `$MFT`, dan USN.
5. Bandingkan control set current dan non-current untuk menemukan persistensi yang diubah/dihapus.

## Detail lebih lanjut { #further-details }

- [Microsoft: Pohon registry untuk layanan perangkat dan driver](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / tipe dan status layanan](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: Tindakan kegagalan ChangeServiceConfig2](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
