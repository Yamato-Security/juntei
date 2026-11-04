---
hide:
  - navigation
  - toc
---

<div class="jt-hero" markdown>

![Juntei, déesse de la criminalistique numérique](assets/juntei-goddess.png){ .jt-hero-art width="1448" height="1086" }

<p class="tagline">
<strong>Juntei</strong> est le parseur multiplateforme de Yamato Security pour les artefacts forensiques Windows
qui ne sont <em>pas</em> des journaux d'événements. Il convertit les preuves collectées en enregistrements CSV, JSON et JSONL
cohérents, tout en préservant le contexte dont un enquêteur a besoin pour éviter les conclusions erronées.
</p>

<div class="jt-cta" markdown>
[Commencer ici :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[Découvrir les artefacts :material-archive-search:](artifacts/index.md){ .md-button }
[Référence des commandes :material-console:](commands/index.md){ .md-button }
[GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/juntei){ .md-button }
</div>

</div>

## De l'artefact brut à une conclusion défendable { #from-raw-artifact-to-defensible-conclusion }

Juntei peut vous indiquer ce qu'enregistre un artefact ; ce site explique ce que cet enregistrement **prouve et ne
prouve pas**. Commencez par une page d'artefact, même si vous n'avez jamais pratiqué le DFIR Windows.
Chaque guide en présente la finalité, l'emplacement sur disque, les réserves liées à la rétention et à la collecte, la commande d'analyse,
les champs de sortie, l'interprétation, les corrélations et des lectures approfondies.

<div class="grid cards" markdown>

-   :material-file-tree:{ .lg .middle } __Historique du système de fichiers__

    ---

    Utilisez la `$MFT` et le journal USN pour reconstituer l'existence des fichiers, leurs noms, leurs déplacements, leur suppression,
    les modifications de métadonnées et les flux de données alternatifs.

-   :material-play-circle-outline:{ .lg .middle } __Preuves relatives aux programmes__

    ---

    Prefetch peut fortement étayer une exécution. Amcache et ShimCache apportent un inventaire, des chemins, des empreintes,
    des informations de compatibilité et un contexte historique, mais exigent une interprétation prudente.

-   :material-account-clock:{ .lg .middle } __Activité des utilisateurs__

    ---

    Les fichiers LNK, les Jump Lists, RecentDocs et SRUM révèlent les éléments ouverts, l'activité des applications,
    les emplacements amovibles ou réseau, l'utilisation des ressources et le contexte historique de l'utilisateur.

-   :material-shield-bug:{ .lg .middle } __Preuves liées à la sécurité__

    ---

    La quarantaine Defender peut conserver les libellés de menaces, les chemins d'origine, les origines des téléchargements, les empreintes,
    les propriétaires, et même permettre de récupérer en toute sécurité les octets mis en quarantaine pour une analyse isolée.

-   :material-database-search:{ .lg .middle } __État du Registre__

    ---

    Lisez les données de Registre allouées et récupérables, rejouez les journaux de transactions, et examinez les services,
    les tâches planifiées, les comptes locaux et l'état des documents récents.

-   :material-link-variant:{ .lg .middle } __Corréler, ne pas deviner__

    ---

    Aucun artefact ne raconte à lui seul toute l'histoire. Les guides montrent comment combiner des sources indépendantes
    en une chronologie étayée par des preuves.

</div>

!!! warning "Une sortie forensique est une preuve, pas un verdict"
    Un chemin qui paraît suspect est une piste. Un horodatage peut décrire les métadonnées d'un fichier plutôt que
    son exécution. L'absence d'un enregistrement ne prouve pas qu'une action n'a pas eu lieu. Préservez les preuves sources,
    consignez les versions des outils et les commandes, et corroborez les conclusions importantes.
