# Screenshots

Explore Juntei's command line, automatic parsing, and organized investigation results.
Click a screenshot to view it at full size.

## 1. Startup and commands

<figure markdown>

[![Juntei startup screen showing the CODE BLUE release and available commands](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>Juntei's startup screen lists automatic parsing, source-specific parsers, and Persistence Hunter.</figcaption>

</figure>

## 2. Automatic parsing in under 20 seconds

On a **MacBook Pro M4**, the run shown below parsed **8.7 GiB of collected Windows data**
from **six systems** and saved **4.4 GiB of output in 19.685 seconds**—less than 20 seconds.
The [auto-parse](commands/auto-parse.md) command ran 240 parser tasks using 16 parallel
processes and produced 482 result files. There were **210 successful tasks, 30 skipped
because the artifacts were not collected, and zero failures**.

<figure markdown>

[![Juntei auto-parse completing six systems, 8.7 GiB of input and 4.4 GiB of output in 19.685 seconds](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>Automatic host discovery, parallel parsing, and a completion summary showing the input size, output size, task results, and elapsed time.</figcaption>

</figure>

## 3. Results organized for investigation

<figure markdown>

[![AZUCHI output folders grouped into DFIR categories, with account CSVs and their parser logs](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>Each computer's results are grouped by DFIR category. Artifact CSVs and their matching <code>-logs.txt</code> files stay together, making it easy to find both the records and parsing details.</figcaption>

</figure>
