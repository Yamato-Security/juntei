---
hide:
  - navigation
  - toc
---

<div class="jt-hero" markdown>

![Juntei, डिजिटल फ़ॉरेंसिक की देवी](assets/juntei-goddess.png){ .jt-hero-art width="1448" height="1086" }

<p class="tagline">
<strong>Juntei</strong> Yamato Security का cross-platform parser है, जो उन Windows forensic artifacts के लिए है
जो event logs <em>नहीं</em> हैं। यह एकत्र किए गए साक्ष्य को सुसंगत CSV, JSON और JSONL
records में बदलता है, और वह संदर्भ सुरक्षित रखता है जिसकी जाँचकर्ता को ग़लत निष्कर्षों से बचने के लिए आवश्यकता होती है।
</p>

<div class="jt-cta" markdown>
[यहाँ से शुरू करें :material-rocket-launch:](getting-started/index.md){ .md-button .md-button--primary }
[आर्टिफ़ैक्ट समझें :material-archive-search:](artifacts/index.md){ .md-button }
[कमांड संदर्भ :material-console:](commands/index.md){ .md-button }
[GitHub :fontawesome-brands-github:](https://github.com/Yamato-Security/juntei){ .md-button }
</div>

</div>

## कच्चे artifact से बचाव-योग्य निष्कर्ष तक { #from-raw-artifact-to-defensible-conclusion }

Juntei आपको बता सकता है कि कोई artifact क्या दर्ज करता है; यह site समझाती है कि वह record **क्या सिद्ध करता है और क्या
सिद्ध नहीं करता**। भले ही आपने पहले कभी Windows DFIR न किया हो, किसी artifact page से शुरुआत करें।
हर मार्गदर्शिका उसका उद्देश्य, disk पर स्थान, retention और collection से जुड़ी सावधानियाँ, parsing command,
output fields, व्याख्या, correlations और आगे पढ़ने की सामग्री को कवर करती है।

<div class="grid cards" markdown>

-   :material-file-tree:{ .lg .middle } __File-system इतिहास__

    ---

    फ़ाइल के अस्तित्व, नामों, स्थानांतरण, deletion, metadata बदलावों और alternate data streams का
    पुनर्निर्माण करने के लिए `$MFT` और USN Journal का उपयोग करें।

-   :material-play-circle-outline:{ .lg .middle } __Program साक्ष्य__

    ---

    Prefetch निष्पादन का मज़बूती से समर्थन कर सकता है। Amcache और ShimCache inventory, path, hash,
    compatibility और ऐतिहासिक संदर्भ जोड़ते हैं—लेकिन सावधानीपूर्वक व्याख्या की माँग करते हैं।

-   :material-account-clock:{ .lg .middle } __उपयोगकर्ता गतिविधि__

    ---

    LNK फ़ाइलें, Jump Lists, RecentDocs और SRUM खोले गए items, application गतिविधि,
    removable या network स्थान, resource उपयोग और ऐतिहासिक उपयोगकर्ता संदर्भ उजागर करते हैं।

-   :material-shield-bug:{ .lg .middle } __सुरक्षा साक्ष्य__

    ---

    Defender क्वारंटीन threat labels, मूल paths, download origins, hashes,
    owners सुरक्षित रख सकता है, और isolated विश्लेषण के लिए quarantine किए गए bytes को सुरक्षित रूप से recover भी कर सकता है।

-   :material-database-search:{ .lg .middle } __Registry स्थिति__

    ---

    Allocated और recover किए जा सकने वाले registry data पढ़ें, transaction logs replay करें, और services,
    scheduled tasks, local accounts और recent-document स्थिति की जाँच करें।

-   :material-link-variant:{ .lg .middle } __Correlate करें, अनुमान न लगाएँ__

    ---

    कोई भी अकेला artifact पूरी कहानी नहीं बताता। मार्गदर्शिकाएँ दिखाती हैं कि स्वतंत्र स्रोतों को
    साक्ष्य-आधारित timeline में कैसे जोड़ा जाए।

</div>

!!! warning "Forensic output साक्ष्य है, निर्णय नहीं"
    संदिग्ध दिखने वाला path एक सुराग है। Timestamp निष्पादन के बजाय फ़ाइल के metadata का वर्णन कर सकता है।
    लापता record यह सिद्ध नहीं करता कि कोई action नहीं हुई। स्रोत साक्ष्य सुरक्षित रखें,
    tool versions और commands दर्ज करें, और महत्वपूर्ण निष्कर्षों की पुष्टि करें।
