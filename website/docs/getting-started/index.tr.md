# Juntei Kurulumu { #install-juntei }

## Sürüm ikili dosyaları { #release-binaries }

Sisteminize uygun zip dosyasını
[Juntei sürümler sayfasından](https://github.com/Yamato-Security/juntei/releases) indirin. Henüz bir sürüm ikili dosyası
yayımlanmamışsa aşağıda açıklandığı gibi kaynaktan derleyin. Başka bir analistin çalıştırmayı yeniden üretebilmesi için
arşivi veya kaynak commit'ini ve sağlama toplamını vaka notlarınızla birlikte saklayın.

| Sistem | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` veya `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` veya `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

Her zip yalnızca sürüm ve platformla adlandırılmış ikili dosyayı içerir (örneğin
`juntei-1.0.0-win-x64.exe`): Juntei başka hiçbir dosyaya ihtiyaç duymaz ve Windows ikili dosyası
Visual C++ Redistributable gerektirmez. musl Linux ikili dosyaları statik olarak bağlanmıştır ve her
dağıtımda çalışır; gnu olanlar güncel bir glibc gerektirir. macOS'ta indirilen ikili dosyaya
Sistem Ayarları > Gizlilik ve Güvenlik bölümünden izin verin veya karantina özniteliğini
`xattr -d com.apple.quarantine juntei-<version>-mac-*` ile kaldırın.

```bash
juntei --version
juntei --help
```

Örneklerde ikili dosya için `juntei` kullanılır; dosyayı yeniden adlandırın veya tam adını yazın.

## Kaynaktan derleme { #compile-from-source }

Juntei şu anda `Cargo.toml` içinde belirtilen Rust sürümünü gerektirir (bu sayfa yazıldığı sırada
Rust 1.99).

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked`, `Cargo.lock` içinde kaydedilen bağımlılık revizyonlarının tam olarak aynısını kullanır.

## Temel komut biçimi { #basic-command-shape }

Her ayrıştırıcı tek bir dosya (`-f`) veya bir dizin koleksiyonu (`-d`) kabul eder:

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

Bir rapordaki eski bir komutu kopyalamak yerine güncel seçenekleri görüntüleyin:

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

Güvenli bir iş akışı için [İlk İnceleme](first-investigation.md) sayfasıyla devam edin.
