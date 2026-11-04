# Première investigation { #first-investigation }

Ce déroulement suppose que vous avez reçu une collecte de triage hors ligne dans `evidence/`.

## 1. Préserver la source { #1-preserve-the-source }

Travaillez à partir d'une copie vérifiée. Consignez les empreintes, l'heure de collecte, le fuseau horaire, l'hôte, le volume et l'outil
de collecte. Juntei lit les preuves, mais des options telles que l'extraction des flux résidents ou de la quarantaine créent
de nouveaux fichiers de sortie ; dirigez-les vers un répertoire d'analyse distinct.

## 2. Inventorier ce dont vous disposez { #2-inventory-what-you-have }

Recherchez les chemins indiqués dans [Collecte des preuves](collection.md). Il est normal que des artefacts manquent. Ne
supposez pas qu'une collecte contient des journaux de transactions simplement parce qu'elle contient une ruche du Registre.

## 3. Exécuter d'abord les résumés { #3-run-summaries-first }

De nombreuses commandes affichent un résumé concis lorsqu'aucun fichier de sortie n'est demandé :

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

Lisez les avertissements relatifs aux dommages, aux bases de données « sales » (dirty) et aux journaux non appliqués avant de vous fier aux enregistrements.

Pour analyser en une seule opération tous les artefacts pris en charge d'une collecte portant sur plusieurs ordinateurs, utilisez :

```bash
juntei auto-parse -d evidence/ -o analysis/
```

Consultez [Analyse automatique des collectes](../commands/auto-parse.md) pour la découverte des hôtes, le parallélisme,
l'organisation de la sortie et le JSONL consolidé destiné à l'ingestion dans un SIEM.

## 4. Enregistrer une sortie structurée { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

Préférez JSONL pour les scripts et les valeurs imbriquées, et CSV pour un tableur ou une visionneuse de chronologie.

## 5. Pivoter à partir d'un identifiant fiable { #5-pivot-from-a-reliable-identifier }

Commencez par un nom de fichier, un chemin complet, un SHA-1/SHA-256, un SID d'utilisateur, une entrée/séquence MFT, un GUID de tâche,
un nom de service ou une fenêtre temporelle. Ne normalisez la casse que pour les champs que Windows traite sans tenir compte de la casse.

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. Corréler { #6-correlate }

Utilisez les [Investigations croisées entre artefacts](../resources/workflows.md). Une séquence pratique est la suivante :

1. Prefetch pour l'exécution et les heures d'exécution récentes.
2. `$MFT` et journal USN pour la création, les chemins, les renommages et la suppression.
3. Amcache et ShimCache pour la présence historique et les métadonnées.
4. LNK, Jump Lists, RecentDocs et SRUM pour le contexte utilisateur/application.
5. Services du Registre et TaskCache pour la persistance.
6. Quarantaine Defender pour la détection, la suppression, l'origine et les empreintes.

## 7. Documenter la reproductibilité { #7-document-reproducibility }

Consignez le chemin source, l'empreinte de la source, la version de Juntei, la commande exacte, l'empreinte de la sortie, le fuseau horaire du système
et tout filtrage ultérieur. Ne modifiez jamais silencieusement une sortie de preuves générée.
