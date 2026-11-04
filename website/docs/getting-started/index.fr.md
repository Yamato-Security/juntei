# Installer Juntei { #install-juntei }

## Binaires publiés { #release-binaries }

Téléchargez le fichier zip correspondant à votre système depuis la
[page des versions de Juntei](https://github.com/Yamato-Security/juntei/releases). Si aucun binaire n'a encore été
publié, compilez à partir des sources comme décrit ci-dessous. Conservez l'archive ou le commit source ainsi que
la somme de contrôle avec vos notes de dossier, afin qu'un autre analyste puisse reproduire l'exécution.

| Système | Zip |
| --- | --- |
| Windows x64 | `juntei-<version>-win-x64.zip` |
| Windows Arm64 | `juntei-<version>-win-aarch64.zip` |
| Linux x64 | `juntei-<version>-lin-x64-gnu.zip` ou `juntei-<version>-lin-x64-musl.zip` |
| Linux Arm64 | `juntei-<version>-lin-aarch64-gnu.zip` ou `juntei-<version>-lin-aarch64-musl.zip` |
| macOS (Apple silicon) | `juntei-<version>-mac-aarch64.zip` |
| macOS (Intel) | `juntei-<version>-mac-x64.zip` |

Chaque zip ne contient que le binaire, nommé avec la version et la plateforme (par exemple
`juntei-1.0.0-win-x64.exe`) : Juntei n'a besoin d'aucun autre fichier, et le binaire Windows ne nécessite pas le
Visual C++ Redistributable. Les binaires Linux musl sont liés statiquement et s'exécutent sur n'importe quelle
distribution ; les binaires gnu nécessitent une glibc récente. Sous macOS, autorisez le binaire téléchargé dans
Réglages Système > Confidentialité et sécurité, ou supprimez son attribut de quarantaine avec
`xattr -d com.apple.quarantine juntei-<version>-mac-*`.

```bash
juntei --version
juntei --help
```

Les exemples utilisent `juntei` pour désigner le binaire ; renommez-le, ou saisissez son nom complet.

## Compiler à partir des sources { #compile-from-source }

Juntei nécessite actuellement la version de Rust déclarée dans `Cargo.toml` (Rust 1.99 au moment de la rédaction
de cette page).

```bash
git clone https://github.com/Yamato-Security/juntei.git
cd juntei
cargo build --release --locked
./target/release/juntei --version
```

`--locked` utilise les révisions exactes des dépendances enregistrées dans `Cargo.lock`.

## Forme de base des commandes { #basic-command-shape }

Chaque parseur accepte soit un fichier unique (`-f`), soit une collecte sous forme de répertoire (`-d`) :

```bash
juntei <COMMAND> -f <ARTIFACT> [OPTIONS]
juntei <COMMAND> -d <COLLECTION> [OPTIONS]
```

Affichez les options actuelles plutôt que de copier une ancienne commande tirée d'un rapport :

```bash
juntei parse-file prefetch --help
juntei parse-registry read --help
```

Poursuivez avec [Première investigation](first-investigation.md) pour une méthode de travail sûre.
