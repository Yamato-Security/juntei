# सामान्य आउटपुट विधियाँ { #common-output-patterns }

## एक आर्टिफ़ैक्ट या एक संग्रह { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d` उस command के लिए उपयुक्त फ़ाइलों को recursively खोजता है। Source-path columns संयुक्त आउटपुट में
hosts और आर्टिफ़ैक्ट में अंतर करते हैं।

## CSV सहेजें { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

कुछ parsers को directory की आवश्यकता होती है क्योंकि वे कई CSV tables बनाते हैं:

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## JSONL stream करें { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

Messages और progress standard error पर जाते हैं; standard output parse करने योग्य बना रहता है।

## जानबूझकर overwrite करें { #overwrite-intentionally }

Juntei default रूप से मौजूदा आउटपुट को overwrite करने से इनकार करता है। Target की पुष्टि करें, कोई भी पहले का
महत्वपूर्ण परिणाम सुरक्षित रखें, फिर `-C` / `--clobber` जोड़ें:

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## Quiet और no-color modes { #quiet-and-no-color-modes }

Automation के लिए `-q` का और जब terminal color codes अनुपयुक्त हों तब `-K` का उपयोग करें:

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## Case और paths { #case-and-paths }

Spaces, `$`, shell metacharacters या backslashes वाले Windows paths को अपने shell की आवश्यकता के अनुसार
quote करें। Bash-जैसे shells पर, single quotes `$MFT` और `$J` को variables बनने से रोकते हैं:

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
