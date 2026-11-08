# Outlook test files

Openly licensed PST, OST, `.msg` and `.oft` files from other projects' test data for
`parse-file outlook`, the same ones [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)'s
tests read. The `.gz` files are the original files compressed with gzip, and nothing else is
changed; `tests/outlook_cli.rs` decompresses them into temporary directories. The SHA-256 is of
the original (decompressed) file.

| File | Source | What it is | Rows (`parse-file outlook`) |
| --- | --- | --- | --- |
| `libyal/outlook.pst.gz` | [libyal/testdata](https://github.com/libyal/testdata) `pst/outlook.pst` (`f018eb7`) | Unicode PST (permute encoding): 22 folders, 2 messages with recipients and 22 attachments | 22 folders, 2 items, 2 recipients, 22 attachments |
| `java-libpst/dist-list.pst.gz` | [rjohnsondev/java-libpst](https://github.com/rjohnsondev/java-libpst) `src/test/resources/dist-list.pst` (`f158a64`) | Unicode PST (permute encoding): 4 messages (a contact, a distribution list, a free/busy item and a recurring appointment whose two exceptions are embedded messages) and 3 hidden items | 24 folders, 9 items, 2 attachments, 2 contacts, 1 calendar item |
| `java-libpst/example-2013.ost.gz` | The same: `src/test/resources/example-2013.ost` | Outlook 2013 OST in the 4 KiB layout (no encoding, compressed blocks): 3 messages and 14 hidden items | 46 folders, 17 items, 3 recipients |
| `msgreader-hiraoka/msgInMsgInMsg.msg` | [HiraokaHyperTools/msgreader](https://github.com/HiraokaHyperTools/msgreader) `test/msgInMsgInMsg.msg` (`8f9b2a3`) | A message in a message in a message, with Japanese text in the innermost | 3 items, 1 recipient, 4 attachments |
| `msgreader-hiraoka/contactAnsi.msg` | The same: `test/contactAnsi.msg` | An ANSI contact in code page 932 (Japanese) | 1 item, 1 contact |
| `apache-poi/msgClassTask.msg` | [apache/poi](https://github.com/apache/poi) `test-data/hsmf/msgClassTask.msg` (`519e0a3`) | A task | 1 item, 1 task |
| `apache-poi/msgClassAppointment.msg` | The same: `test-data/hsmf/msgClassAppointment.msg` | An appointment with a recipient | 1 item, 1 recipient, 1 calendar item |
| `aspose-email-java/mapiToOft.oft` | [aspose-email/Aspose.Email-for-Java](https://github.com/aspose-email/Aspose.Email-for-Java) `Examples/src/main/resources/outlook/mapiToOft.oft` (`cf4567e`) | An Outlook template | 1 item, 1 recipient |

Every file also has its row in the Files table. With `--properties`, the eight files give 3,496
property rows.

## Licences

| Directory | Licence |
| --- | --- |
| `libyal/` | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) (libyal test data, Joachim Metz): see `libyal/NOTICE` |
| `java-libpst/` | Apache-2.0 (the project is dual-licensed Apache-2.0 or LGPL; used under Apache-2.0): `java-libpst/LICENSE` |
| `msgreader-hiraoka/` | Apache-2.0: `msgreader-hiraoka/LICENSE` |
| `apache-poi/` | Apache-2.0: `apache-poi/LICENSE`, with Apache POI's `apache-poi/NOTICE` |
| `aspose-email-java/` | MIT (Aspose Pty Ltd): `aspose-email-java/LICENSE` |

## SHA-256

```text
df01707f76d0e24ab913cf1ffeffa6eaf9c1d590e02102c7ed26bb3ac51d4e24  libyal/outlook.pst
c86841da106036b5abe5a2141dc7644cbb2bf8b504873515eb35a2efeb8c28ac  java-libpst/dist-list.pst
b18ff390d0c22273029ee580ab6025976af69a4f84e276d1b1c01a988650853d  java-libpst/example-2013.ost
9c77fc1ff1b9132c546959a0ed6c98e3678cd369bf6b0b6abb777abc9e6720d6  msgreader-hiraoka/msgInMsgInMsg.msg
79b82e21d032cca26d25cacb2092b36f25e0f463d3c78b7067cc2a10cca9dc9d  msgreader-hiraoka/contactAnsi.msg
3cd593624de75119a0b7d30ad40c62aa19e3b5444c59fece0155dd43a414e31c  apache-poi/msgClassTask.msg
e0a8fb568bc837c4f835a9e85e70280beb1b604b6223a3fa547b22c4860f3eb9  apache-poi/msgClassAppointment.msg
7bb13973976b7fea099e0becd1e152849f258f470855c1a154452cdbf4bfdbd9  aspose-email-java/mapiToOft.oft
```
