# Glossaire DFIR { #dfir-glossary }

| Terme | Signification en langage courant |
|---|---|
| Alloué (allocated) | L'enregistrement/l'objet est actuellement utilisé selon l'artefact. |
| Flux de données alternatif (Alternate Data Stream, ADS) | Données NTFS nommées attachées à un fichier en plus de son contenu normal sans nom, par exemple `Zone.Identifier`. |
| Artefact | Données persistantes produites par un système d'exploitation/une application, capables de répondre à des questions forensiques. |
| Carving | Recherche d'enregistrements d'après leur structure/signature, sans s'appuyer sur leur conteneur/index habituel. |
| Jeu de contrôle (control set) | Une version de la configuration SYSTEM (`ControlSet001`, etc.) ; `Select\Current` désigne celle qui est active. |
| Corroboration | Preuves indépendantes étayant la même conclusion. |
| Base de données/ruche « dirty » | Le fichier principal n'était pas entièrement synchronisé au moment de la collecte ; les journaux de transactions peuvent contenir l'état requis. |
| Droid | Identifiant de suivi des liens distribués, semblable à un GUID, utilisé pour suivre les fichiers/volumes lors de leurs déplacements. |
| ESE / JET Blue | Format de base de données Extensible Storage Engine de Microsoft, utilisé par SRUM et d'autres données Windows. |
| FILETIME | Nombre d'intervalles de 100 nanosecondes depuis le 1601-01-01 UTC ; un nombre n'a de sens en tant qu'heure que si son schéma l'indique. |
| Référence de fichier | Numéro d'entrée NTFS accompagné d'un numéro de séquence, identifiant une génération d'un enregistrement MFT. |
| Ruche (hive) | Fichier binaire contenant une arborescence du Registre Windows. |
| Enregistrement INDX | Bloc (généralement de 4 Kio) d'un index de répertoire NTFS (`$I30`) répertoriant les fichiers du répertoire ; son slack peut conserver d'éventuelles preuves de noms de fichiers supprimés ou renommés. |
| LNK | Shell Link/raccourci Windows contenant une cible et de nombreuses métadonnées. |
| MRU | Ordre des éléments les plus récemment utilisés (Most Recently Used) ; généralement un ordre relatif plutôt qu'un horodatage pour chaque élément. |
| MFT | Master File Table de NTFS, contenant les enregistrements et les attributs des fichiers/répertoires. |
| Hachage Prefetch | Identifiant Windows utilisé dans le nommage des fichiers `.pf` ; il ne s'agit pas d'un hachage cryptographique du contenu. |
| Dernière écriture d'une clé du Registre | Heure à laquelle la clé a changé ; Windows n'indique généralement pas quelle valeur a changé. |
| Données résidentes | Octets d'un fichier/flux stockés directement dans un attribut MFT plutôt que dans des data runs externes. |
| RID | Identificateur relatif (Relative Identifier), la composante finale d'un compte/groupe sous une autorité/un domaine de SID Windows. |
| SDDL | Security Descriptor Definition Language : un descripteur de sécurité écrit sous forme de texte, par exemple `O:S-1-5-18D:(A;;0x001f01ff;;;S-1-5-32-544)`. |
| Descripteur de sécurité | Le propriétaire d'un fichier, son groupe principal, sa DACL (qui peut faire quoi) et sa SACL (audit et étiquette d'intégrité). NTFS conserve chaque descripteur distinct une seule fois dans `$Secure:$SDS`, sous un identifiant de sécurité. |
| SID | Identificateur de sécurité Windows (Security Identifier) d'un compte, d'un service, d'un groupe ou d'un principal de sécurité. |
| Slack | Octets inutilisés à l'intérieur ou à la suite d'une structure allouée, pouvant conserver des données plus anciennes. |
| SRUM | Base de données System Resource Usage Monitor de l'utilisation des ressources par les applications/utilisateurs. |
| Timestomping | Modification intentionnelle des horodatages visant à tromper l'investigation ; les heuristiques ne constituent pas une preuve. |
| Journal de transactions | Modifications en attente/validées utilisées pour ramener un Registre ou une base de données à un état cohérent. |
| USN | Update Sequence Number utilisé par les enregistrements du journal des modifications NTFS et les métadonnées associées. |
| UTC | Temps universel coordonné ; base temporelle commune à privilégier pour les chronologies couvrant plusieurs systèmes. |
| Numéro de série du volume | Numéro attribué par NTFS à un volume lors de son formatage (`$Boot`) ; ses 32 bits de poids faible sont ce que les fichiers LNK, Jump List et Prefetch enregistrent du volume d'un fichier. |
