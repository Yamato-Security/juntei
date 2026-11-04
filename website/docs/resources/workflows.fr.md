# Investigations croisées entre artefacts { #cross-artifact-workflows }

Aucun artefact ne devrait porter seul une conclusion importante. Ces déroulements montrent comment transformer une piste
en une chaîne de preuves défendable.

## Exécution d'un exécutable suspect { #suspicious-executable-execution }

1. **Prefetch :** établissez l'heure d'exécution conservée, le contexte de chemin/d'empreinte, le nombre d'exécutions et les fichiers chargés.
2. **`$MFT` :** trouvez le chemin, la taille, les horodatages, les ADS, l'état de suppression et la référence MFT.
3. **Journal USN :** reconstituez la création, les écritures, le renommage et la suppression.
4. **Amcache :** ajoutez le SHA-1, les métadonnées PE, la version et le chemin historique.
5. **ShimCache :** ajoutez le chemin/l'ordre dans le cache de compatibilité et l'indicateur d'exécution, lorsqu'il est pris en charge.
6. **SRUM :** recherchez l'utilisation des ressources/du réseau par le même exécutable/utilisateur.
7. **Defender :** vérifiez l'heure de mise en quarantaine, le chemin d'origine, l'empreinte et l'origine du téléchargement.
8. **Journaux d'événements/EDR :** identifiez le processus parent, la ligne de commande, le compte, le réseau et le contexte de sécurité.

La conclusion doit distinguer la présence du fichier, l'exécution et l'attribution à une personne ou à une intrusion.

## Activité de fichiers liée à un rançongiciel { #ransomware-file-activity }

1. Identifiez les extensions chiffrées et les chemins des victimes dans `$MFT`.
2. Utilisez les raisons et l'ordre USN pour reconstituer les écritures/renommages/suppressions massifs.
3. Utilisez Prefetch pour identifier les chiffreurs candidats et leurs heures d'exécution.
4. Utilisez SRUM pour repérer les pics de CPU/disque/réseau du processus candidat.
5. Vérifiez les services et TaskCache pour le déploiement/la persistance.
6. Vérifiez la quarantaine Defender et les journaux d'événements pour la détection/la remédiation.

MFT/USN peuvent prouver l'impact sur le système de fichiers même lorsqu'aucun artefact d'exécution ne subsiste. Ils
n'identifient généralement pas le processus responsable sans télémétrie des processus ni corrélation.

## Accès initial par un document / un raccourci malveillant { #malicious-document-shortcut-initial-access }

1. RecentDocs et les Jump Lists identifient l'interaction utilisateur/application et le contexte MRU.
2. Analysez le LNK correspondant du dossier Recent pour la cible, le volume/partage, la référence MFT et les arguments.
3. Utilisez le Zone.Identifier provenant de l'ADS résident du `$MFT` ou du contenu Defender pour les URL/zones d'origine.
4. Utilisez Prefetch/les journaux d'événements pour l'application documentaire et les processus enfants.
5. Utilisez USN/MFT pour suivre la création et la suppression de la charge utile extraite.

## Persistance par service { #service-persistence }

1. Décodez le nom du service, `ImagePath`, `ServiceDll`, le compte, le type de démarrage et les actions en cas d'échec.
2. Résolvez les variables d'environnement et le comportement des chemins avec/sans guillemets.
3. Localisez l'exécutable/la DLL dans le `$MFT` et l'historique USN.
4. Vérifiez Prefetch et les journaux d'événements pour le démarrage effectif du service/processus.
5. Comparez les jeux de contrôles non actuels et les enregistrements du Registre récupérés pour retrouver une configuration antérieure.

## Persistance par tâche planifiée { #scheduled-task-persistence }

1. Décodez les actions TaskCache, le principal/contexte, les métadonnées d'enregistrement/d'exécution et l'état caché.
2. Comparez avec le XML de `C:\Windows\System32\Tasks`.
3. Corrélez le nom/GUID de la tâche avec les événements Security et Task Scheduler Operational.
4. Localisez l'exécutable/le script et calculez son empreinte ; examinez Prefetch et l'historique du système de fichiers.
5. Récupérez les enregistrements TaskCache supprimés si la tâche a été supprimée.

## Exfiltration présumée { #suspected-exfiltration }

1. Utilisez SRUM pour identifier des volumes d'octets envoyés inhabituels par application/utilisateur/profil, ainsi que la période de mesure.
2. Confirmez l'identité de l'exécutable avec Prefetch, Amcache et `$MFT`.
3. Identifiez les fichiers consultés/ouverts au moyen des Jump Lists, des LNK, de RecentDocs et des données d'événements/EDR.
4. Utilisez la télémétrie du proxy, du pare-feu, DNS, des paquets ou de l'EDR pour la destination/le protocole.

Le volume SRUM étaye une hypothèse d'exfiltration, mais n'identifie pas le contenu transféré et ne prouve pas
à lui seul une intention malveillante.

## Clés de corrélation à préserver { #correlation-keys-worth-preserving }

| Clé | Artefacts |
|---|---|
| Chemin complet / nom de fichier | Presque tous les artefacts |
| SHA-1 / SHA-256 | Amcache, Defender, fichiers récupérés |
| Entrée + séquence MFT | `$MFT`, Prefetch, LNK, Jump Lists, USN |
| USN | Amcache, `$MFT`, journal USN |
| Numéro de série de volume / droids | Prefetch, LNK, Jump Lists, identifiants d'objet MFT |
| SID / nom d'utilisateur | SRUM, SAM, Registre, journaux d'événements |
| GUID de tâche / nom de tâche | TaskCache, XML des tâches, journaux d'événements |
| Nom de service | Services, journaux d'événements, données de processus/pilotes |
| EntryId / ScanId Defender | Quarantaine, historique Defender, journaux d'événements |
