# Ucapan terima kasih Persistence Hunter { #persistence-hunter-acknowledgements }

Perintah `persistence-hunter` milik Juntei dibangun di atas dua generasi perburuan persistensi oleh komunitas.
Terima kasih kepada para penulis dan kontributor yang telah menyediakan teknik, implementasi, dan riset mereka
bagi para investigator.

## PyrsistenceSniper: proyek Python { #pyrsistencesniper-the-python-project }

[**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper), karya
[Hexastrike](https://github.com/Hexastrike), adalah referensi langsung untuk fitur ini. Juntei mem-port
katalog 130 pemeriksaannya ke Rust native, beserta allowlist default, snapshot LOLBAS, dan
perilaku klasifikasinya. Versi referensi dipatok pada
[commit `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f).

Proyek Python tersebut berlisensi MIT. Juntei mempertahankan pemberitahuan hak cipta dan lisensinya di
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt).
Implementasi Juntei berjalan secara offline tanpa interpreter Python. Penanganan koleksi,
pelaporan cakupan, dan interpretasi konservatif atas bukti yang tidak lengkap didokumentasikan di
[panduan perintah](../commands/persistence-hunter.md).

## PersistenceSniper: alat PowerShell asli { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) adalah proyek perburuan persistensi
PowerShell asli yang menginspirasi PyrsistenceSniper. PyrsistenceSniper memberikan kredit kepada
Federico Lagrasta; repositori aslinya juga memberikan kredit kepada Riccardo Ancarani dan kontributor
lainnya. Kami berterima kasih kepada proyek asli dan para kontributornya karena telah membuat mekanisme persistensi
Windows lebih mudah diselidiki.

Referensi porting langsung Juntei adalah proyek Python tersebut. Alat PowerShell asli diberi kredit
atas silsilah tersebut. Juntei juga secara independen mengimplementasikan mekanisme yang diusulkan oleh
[hongson11698](https://github.com/hongson11698) di proyek asli melalui
[PR #31 resolusi COM scheduled task](https://github.com/last-byte/PersistenceSniper/pull/31)
dan [PR #32 library/folder CLSID Start Menu](https://github.com/last-byte/PersistenceSniper/pull/32).
Keduanya menambah katalog referensi Python; ini tidak menyiratkan bahwa setiap fitur sistem live di salah satu proyek upstream
diimplementasikan oleh perintah offline Juntei.

## Riset dan alat pendukung { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/) menyediakan pengenal teknik yang dicantumkan di
  katalog. Pemetaan teknik mendeskripsikan kemungkinan penggunaan sebuah mekanisme; nilai registry
  atau berkas yang cocok bukan, dengan sendirinya, serangan yang dikonfirmasi oleh ATT&CK.
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  juga diakui oleh PyrsistenceSniper sebagai inspirasi untuk memeriksa autostart Windows.
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) mendokumentasikan binary Windows yang sah yang dapat
  disalahgunakan. Mengenali salah satu binary ini membantu menginterpretasikan sebuah perintah, tetapi tidak membuat
  perintah tersebut otomatis berbahaya.

Untuk fondasi parser registry, NTFS, Shell Link, log peristiwa, dan lainnya yang digunakan di seluruh Juntei,
lihat [ucapan terima kasih untuk seluruh proyek](../acknowledgements.md).

Kembali ke [panduan mekanisme persistensi](index.md).
