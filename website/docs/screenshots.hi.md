# स्क्रीनशॉट { #screenshots }

Juntei की command line, स्वचालित parsing, और व्यवस्थित जाँच परिणामों को देखें।
किसी screenshot को पूरे आकार में देखने के लिए उस पर click करें।

## 1. Startup और commands { #1-startup-and-commands }

<figure markdown>

[![CODE BLUE release और उपलब्ध commands दिखाती Juntei की startup screen](assets/screenshots/01-Startup.png)](assets/screenshots/01-Startup.png)

<figcaption>Juntei की startup screen स्वचालित parsing, स्रोत-विशिष्ट parsers और Persistence Hunter को सूचीबद्ध करती है।</figcaption>

</figure>

## 2. 20 सेकंड से कम में स्वचालित parsing { #2-automatic-parsing-in-under-20-seconds }

एक **MacBook Pro M4** पर, नीचे दिखाए गए run ने **छह सिस्टमों** से **8.7 GiB एकत्र किया गया Windows data**
पार्स किया और **20 सेकंड से कम में 4.4 GiB output** सहेजा।
[auto-parse](commands/auto-parse.md) command ने 16 समानांतर processes का उपयोग करके 240 parser tasks चलाए
और 482 result फ़ाइलें बनाईं। **210 tasks सफल रहे, 30 छोड़ दिए गए
क्योंकि artifacts एकत्र नहीं किए गए थे, और कोई विफलता नहीं हुई**।

<figure markdown>

[![Juntei auto-parse छह सिस्टम, 8.7 GiB input और 4.4 GiB output 20 सेकंड से कम में पूरा करता हुआ](assets/screenshots/02-AutoParse.png)](assets/screenshots/02-AutoParse.png)

<figcaption>स्वचालित host discovery, समानांतर parsing, और input आकार, output आकार, task परिणाम और बीता हुआ समय दिखाने वाला completion summary।</figcaption>

</figure>

## 3. जाँच के लिए व्यवस्थित परिणाम { #3-results-organized-for-investigation }

<figure markdown>

[![एक सिस्टम के output folders DFIR श्रेणियों में समूहित, account CSVs और उनके parser logs के साथ](assets/screenshots/03-OutputFolders.png)](assets/screenshots/03-OutputFolders.png)

<figcaption>हर कंप्यूटर के परिणाम DFIR श्रेणी के अनुसार समूहित होते हैं। Artifact CSVs और उनकी मेल खाती <code>-logs.txt</code> फ़ाइलें साथ रहती हैं, जिससे records और parsing विवरण दोनों खोजना आसान हो जाता है।</figcaption>

</figure>
