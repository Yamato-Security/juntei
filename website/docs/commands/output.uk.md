# Спільні способи виводу { #common-output-patterns }

## Один артефакт або збірка { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d` рекурсивно шукає файли, що підходять для цієї команди. Стовпці вихідних шляхів розрізняють
хости та артефакти в об'єднаному виводі.

## Збереження CSV { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

Деяким парсерам потрібен каталог, оскільки вони створюють кілька таблиць CSV:

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## Потоковий вивід JSONL { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

Повідомлення та індикатор перебігу виводяться в стандартний потік помилок; стандартний вивід залишається придатним для розбору.

## Навмисне перезаписування { #overwrite-intentionally }

За замовчуванням Juntei відмовляється перезаписувати наявний вивід. Перевірте ціль, збережіть будь-який попередній
результат, який має значення, а потім додайте `-C` / `--clobber`:

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## Тихий режим і режим без кольорів { #quiet-and-no-color-modes }

Використовуйте `-q` для автоматизації та `-K`, коли коди кольорів термінала недоречні:

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## Регістр і шляхи { #case-and-paths }

Беріть у лапки шляхи Windows, що містять пробіли, `$`, метасимволи оболонки або зворотні скісні риски, як того вимагає ваша
оболонка. В оболонках на кшталт Bash одинарні лапки не дають `$MFT` і `$J` перетворитися на змінні:

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
