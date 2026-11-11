# Akun lokal SAM { #sam-local-accounts }

## Apa isi SAM { #what-the-sam-contains }

Hive Security Account Manager (`SAM`) menyimpan record akun lokal. Juntei mendekode nama
akun, RID, field profil/komentar, flag kontrol akun, timestamp logon terakhir/kata sandi, penghitung,
petunjuk kata sandi, keterkaitan akun Microsoft, path profil, dan apakah Windows menyembunyikan akun
dari UI sign-in.

Hal ini berguna untuk menemukan administrator lokal yang baru dibuat, akun dorman yang diaktifkan, perubahan
kata sandi, akun tersembunyi, dan akun persiapan untuk pergerakan lateral.

<span class="evidence-strength">Kesimpulan terbaik:</span> SAM lokal merepresentasikan akun dengan
kondisi yang tercatat. Ini bukan riwayat autentikasi yang lengkap dan tidak mendeskripsikan akun domain.

## Lokasi dan pengumpulan { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

Kumpulkan hive dan log-nya dengan hak akses/akses raw yang memadai. Identitas host dapat diperkaya dari
hive `SYSTEM` yang bersesuaian dalam sebuah koleksi.

## Peringatan bagi analis { #analyst-cautions }

- Perintah ini mengurai akun **lokal**, bukan pengguna domain Active Directory.
- Juntei dengan sengaja tidak mengekstrak hash kata sandi.
- RID 500 adalah akun Administrator bawaan meskipun telah di-rename; nama saja tidak cukup.
- `LastLogon` dan penghitung merupakan kondisi registry, bukan jejak audit per sesi yang lengkap.
- `HiddenFromLogonUI` menyembunyikan akun dari layar sign-in; ini tidak menonaktifkan akun.
- `DefaultAccount` dan `WDAGUtilityAccount` bawaan dapat disembunyikan secara sah.
- Flag akun null dari nilai `F` yang rusak/tidak ada tidak boleh diinterpretasikan sebagai aktif.
- `NamesKeyLastWrittenUTC` memperkirakan konteks pendaftaran nama akun tetapi bukan waktu pembuatan yang terjamin.

## Mengurai dengan Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## Field keluaran { #output-fields }

| Field | Interpretasi |
|---|---|
| `Host`, `SourcePath` | Konteks host dan hive SAM yang tepat. |
| `RID` | Relative Identifier. Titik pivot identitas akun yang kuat; 500 adalah Administrator bawaan. |
| `User` | Nama akun lokal saat ini. Dapat di-rename; korelasikan berdasarkan RID. |
| `FullName`, `Comment`, `UserComment` | Teks deskriptif; dapat dikendalikan penyerang dan berguna untuk meninjau penyamaran (masquerading). |
| `AccountFlags` | Kondisi yang didekode seperti disabled, locked out, password not required, normal account, password never expires. |
| `AccountFlagsValue` | Flag numerik mentah untuk validasi dan bit yang tidak didukung. |
| `CountryCode`, `CodePage` | Metadata locale/code page akun; biasanya konteks, bukan atribusi. |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | Kondisi logon terakhir yang sama dengan presisi penuh dan dalam detik utuh untuk kompatibilitas. Bukan riwayat yang lengkap. |
| `PasswordLastSetUTC` | Waktu perubahan kata sandi yang tercatat. Selidiki nilai yang secara tak terduga baru atau lama. |
| `AccountExpirationUTC` | Masa kedaluwarsa yang dikonfigurasi jika terbatas. Null dapat berarti tidak pernah/tidak diatur. |
| `LastFailedLogonUTC` | Logon lokal gagal terakhir yang tercatat dalam kondisi SAM; perkuat dengan peristiwa Security. |
| `FailedLogonCount`, `LogonCount` | Penghitung kumulatif/kondisi dengan perilaku pembaruan khas Windows; gunakan sebagai konteks, bukan jumlah peristiwa yang tepat. |
| `PasswordHint` | Petunjuk sensitif yang diberikan pengguna; dapat mengungkap pola kata sandi. Tangani dengan semestinya. |
| `InternetUserName` | Identitas akun Microsoft yang terkait, jika ada. |
| `HiddenFromLogonUI` | Kondisi penyembunyian dari UI. Tinjau RID tinggi/akun kustom yang tidak terduga; tidak berarti nonaktif. |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | Konfigurasi lingkungan/logon akun lokal; skrip/path UNC yang tidak biasa dapat mengindikasikan persistensi. |
| `KeyLastWrittenUTC` | Waktu perubahan key akun RID. Tingkat key, bukan spesifik per field. |
| `NamesKeyLastWrittenUTC` | Waktu perubahan key pemetaan nama pengguna; perkiraan siklus hidup akun yang berguna. |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | Nilai FILETIME 64-bit mentah yang dipertahankan untuk validasi dan penanganan nilai sentinel khusus. |
| `TransactionLogStatus`, `LogsApplied` | Konteks pemutaran ulang registry. |
| `Problems` | Ketidaksesuaian antara F/V/Names, entri yang hilang, offset yang tidak valid, atau kerusakan dekode. |

## Strategi investigasi { #investigation-strategy }

1. Bandingkan RID, nama, flag, path profil, dan `NamesKeyLastWrittenUTC` dengan baseline host.
2. Tinjau akun RID 1000+ yang aktif, akun dengan password-not-required, tanpa masa kedaluwarsa, tersembunyi, atau yang baru diubah.
3. Korelasikan dengan peristiwa Security 4720/4722/4724/4732, peristiwa logon, pembuatan profil, dan aktivitas jarak jauh.
4. Telusuri riwayat pemulihan registry jika sebuah akun telah dihapus sebelum pengumpulan.
5. Bedakan akun provisioning/default yang sah dari akun interaktif yang dibuat penyerang.

## Detail lebih lanjut { #further-details }

- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [Plugin SAM dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
