# Captures d'écran { #screenshots }

Découvrez la ligne de commande de Juntei, l'analyse automatique et les résultats d'investigation organisés.
Cliquez sur une capture d'écran pour l'afficher en taille réelle.

## 1. Démarrage et commandes { #1-startup-and-commands }

<figure markdown>

[![Écran de démarrage de Juntei affichant la version CODE BLUE et les commandes disponibles](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>L'écran de démarrage de Juntei liste l'analyse automatique, les parseurs propres à chaque source et Persistence Hunter.</figcaption>

</figure>

## 2. Analyse automatique en moins de 20 secondes { #2-automatic-parsing-in-under-20-seconds }

Sur un **MacBook Pro M4**, l'exécution présentée ci-dessous a analysé **8,7 Gio de données Windows collectées**
provenant de **six systèmes** et a enregistré **4,4 Gio de sortie en moins de 20 secondes**.
La commande [auto-parse](commands/auto-parse.md) a exécuté 240 tâches d'analyse à l'aide de 16 processus
parallèles et a produit 482 fichiers de résultats. Il y a eu **210 tâches réussies, 30 ignorées
parce que les artefacts n'avaient pas été collectés, et aucun échec**.

<figure markdown>

[![auto-parse de Juntei traitant six systèmes, 8,7 Gio en entrée et 4,4 Gio en sortie en moins de 20 secondes](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>Découverte automatique des hôtes, analyse en parallèle et synthèse finale indiquant la taille d'entrée, la taille de sortie, les résultats des tâches et le temps écoulé.</figcaption>

</figure>

## 3. Des résultats organisés pour l'investigation { #3-results-organized-for-investigation }

<figure markdown>

[![Dossiers de sortie d'un système regroupés par catégorie DFIR, avec les CSV de comptes et leurs journaux d'analyse](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>Les résultats de chaque ordinateur sont regroupés par catégorie DFIR. Les CSV d'artefacts et leurs fichiers <code>-logs.txt</code> correspondants restent ensemble, ce qui permet de retrouver facilement à la fois les enregistrements et les détails de l'analyse.</figcaption>

</figure>
