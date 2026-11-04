# Remerciements de Persistence Hunter { #persistence-hunter-acknowledgements }

La commande `persistence-hunter` de Juntei s'appuie sur deux générations de travaux communautaires de recherche de la persistance.
Merci aux auteurs et contributeurs qui ont mis leurs techniques, leurs implémentations et leurs recherches
à la disposition des enquêteurs.

## PyrsistenceSniper : le projet Python { #pyrsistencesniper-the-python-project }

[**PyrsistenceSniper**](https://github.com/hexastrike/PyrsistenceSniper), par
[Hexastrike](https://github.com/Hexastrike), est la référence directe de cette fonctionnalité. Juntei porte
son catalogue de 130 vérifications en Rust natif, ainsi que la liste d'autorisation par défaut, l'instantané LOLBAS et le
comportement de classification. La version de référence est figée au
[commit `b99c117`](https://github.com/hexastrike/PyrsistenceSniper/tree/b99c117a3cd16c33992a56dd698aedaec024460f).

Le projet Python est sous licence MIT. Juntei conserve sa mention de copyright et de licence dans
[`THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt`](https://github.com/Yamato-Security/juntei/blob/main/THIRD_PARTY_NOTICES/PyrsistenceSniper-LICENSE.txt).
L'implémentation de Juntei fonctionne hors ligne, sans interpréteur Python. Sa gestion des collectes,
son rapport de couverture et son interprétation prudente des preuves incomplètes sont documentés dans le
[guide de la commande](../commands/persistence-hunter.md).

## PersistenceSniper : l'outil PowerShell d'origine { #persistencesniper-the-original-powershell-tool }

[**PersistenceSniper**](https://github.com/last-byte/PersistenceSniper) est le projet PowerShell d'origine
de recherche de la persistance qui a inspiré PyrsistenceSniper. PyrsistenceSniper crédite
Federico Lagrasta ; le dépôt d'origine crédite également Riccardo Ancarani et d'autres
contributeurs. Nous remercions le projet d'origine et ses contributeurs d'avoir rendu les mécanismes de persistance
de Windows plus faciles à investiguer.

La référence directe du portage de Juntei est le projet Python. L'outil PowerShell d'origine est crédité
au titre de cette filiation. Juntei implémente également de manière indépendante les mécanismes proposés par
[hongson11698](https://github.com/hongson11698) dans la
[PR #31 de résolution COM des tâches planifiées](https://github.com/last-byte/PersistenceSniper/pull/31)
et la [PR #32 des bibliothèques/dossiers CLSID du menu Démarrer](https://github.com/last-byte/PersistenceSniper/pull/32) du projet d'origine.
Ceux-ci complètent le catalogue de référence Python ; cela n'implique pas que chaque fonctionnalité pour système actif de l'un ou l'autre des projets amont
soit implémentée par la commande hors ligne de Juntei.

## Recherches et outils associés { #supporting-research-and-tools }

- [MITRE ATT&CK](https://attack.mitre.org/) fournit les identifiants de techniques présents dans le
  catalogue. La correspondance avec une technique décrit une utilisation possible d'un mécanisme ; une valeur de Registre
  ou un fichier correspondant ne constitue pas, à lui seul, une attaque confirmée au sens d'ATT&CK.
- [Microsoft Sysinternals Autoruns](https://learn.microsoft.com/en-us/sysinternals/downloads/autoruns)
  est également reconnu par PyrsistenceSniper comme source d'inspiration pour l'examen des démarrages automatiques de Windows.
- [LOLBAS](https://github.com/LOLBAS-Project/LOLBAS) documente des binaires Windows légitimes qui peuvent
  être détournés. Reconnaître l'un de ces binaires aide à interpréter une commande, mais ne rend pas cette
  commande automatiquement malveillante.

Pour les fondations des parseurs du Registre, de NTFS, des Shell Links, des journaux d'événements et autres utilisées dans l'ensemble de Juntei,
consultez les [remerciements à l'échelle du projet](../acknowledgements.md).

Revenir au [guide des mécanismes de persistance](index.md).
