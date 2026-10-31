# Acknowledgements

Juntei exists because the DFIR and open-source communities published parsers, format research,
reference output, specifications, and test evidence. The projects below informed implementation,
validation, field naming, or expected analyst output. Being listed does not mean their code is
embedded in Juntei; consult each project for its license.

## Parser foundations maintained by Yamato Security

| Juntei area | Maintained parser | Upstream/foundation |
|---|---|---|
| Amcache | [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) | [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) |
| `$MFT` | [ys-mft-parser](https://github.com/Yamato-Security/ys-mft-parser) | [Omer Ben-Amram mft](https://github.com/omerbenamram/mft) |
| Prefetch | [ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) | [ForensicRS frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) |
| USN Journal | [ys-usn-journal-parser](https://github.com/Yamato-Security/ys-usn-journal-parser) | [Sootmark usn](https://github.com/Sootmark/usn) |
| NTFS `$LogFile` | [ys-logfile-parser](https://github.com/Yamato-Security/ys-logfile-parser) | [Sootmark ntfslog](https://github.com/Sootmark/ntfslog) |
| Directory indexes (`$I30`) | [ys-mft-parser](https://github.com/Yamato-Security/ys-mft-parser) (`i30` module) | Index-slack research credited below |
| SRUM / ESE | [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) | [Sootmark srum](https://github.com/Sootmark/srum) and [ese](https://github.com/Sootmark/ese) |
| LNK | [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) | [Sootmark shell](https://github.com/Sootmark/shell) |
| Jump Lists | [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) | Sootmark shell compound-file and Jump List work |
| ShimCache | [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) | [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) |
| Defender | [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) | [Sootmark defender](https://github.com/Sootmark/defender) |
| Registry | [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) | Registry-format and recovery research credited below |

## Reference tools and independent validation

- [**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper), by Hexastrike,
  supplied the persistence check catalogue, default allowlist, LOLBAS snapshot, and reference
  behavior for the native Rust `persistence-sniper` command. Its MIT license and copyright notice
  are retained in `THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`.

- Maxim Suhanov's **regf specification** and **yarp** informed registry structures, transaction-log
  replay, and deleted-record recovery; his **dfir_ntfs** was the reference for INDX records and
  index-slack carving.
- Harel Segev's **INDXRipper** is a further reference for carving file names from index slack.
- **dfir_ntfs**, Joakim Schicht's **LogFileParser**, and Stroz Friedberg's **ntfs-linker** informed
  `$LogFile` validation and interpretation; Velocidex's **go-ntfs** supplied public NTFS test evidence.
- Joachim Metz / libyal projects **libregf**, **libfsntfs**, **libscca**, **libfwsi**, and
  **libesedb** provide detailed format research and independent parsing references.
- Fox-IT's **dissect.esedb** and **dissect.target** provided independent SRUM, SAM, and Defender
  interpretations used during validation.
- Some of Eric Zimmerman's tools established field names and examiner expectations used for comparisons.
- **regipy**, **python-registry**, **Notatin**, and **artemis** provided registry implementations,
  tests, or sample interpretations.
- **LnkParse3**, AbdulRhmanAlfaifi's **lnk_parser** and **jumplist_parser**, and JLECmd provided
  independent LNK/Jump List behavior and public test cases.
- Security Ronin's **prefetch-forensic** and the **forensicnomicon** project informed Juntei's
  high-value Prefetch/Amcache/ShimCache observations.
- **defender-dump** and dissect.target provide separate implementations for understanding Defender
  quarantine structures and safe extraction behavior.
- **Plaso**, **Chainsaw**, and **Velociraptor** supplied parser comparisons, test evidence, or
  collection-oriented reference behavior across several artifacts.

## Specifications and research

We thank Microsoft for publishing Open Specifications for Shell Links, Compound Binary Files,
NTFS/SAM protocols, registry/service/task structures, and API documentation. We also thank the
authors of AppCompatCache, Amcache, SRUM, Prefetch, registry, and NTFS forensic research that made
these artifacts understandable to practitioners.

## Test evidence

Public evidence from Andrew Rathbun's
[DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum), NIST CFReDS, Plaso,
Regipy, Artemis, parser test repositories, and other properly licensed collections makes regression
testing against real Windows data possible. Juntei keeps attribution/license notes beside included
test files.

## Contributors

Thank you to every Juntei and Yamato Security parser contributor, reviewer, issue reporter, and
user who supplied edge cases. Current contributors are listed by GitHub on the
[Juntei contributors page](https://github.com/Yamato-Security/juntei/graphs/contributors).

## License

Juntei is released under the
[GNU Affero General Public License v3.0](https://github.com/Yamato-Security/juntei/blob/main/LICENSE).
