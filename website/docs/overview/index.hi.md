# Juntei के बारे में { #about-juntei }

Juntei memory-safe Rust में लिखा गया एक Windows फ़ॉरेंसिक आर्टिफ़ैक्ट parser है। यह
[Hayabusa](https://github.com/Yamato-Security/hayabusa) का पूरक है: Hayabusa Windows event logs का विश्लेषण करता है,
जबकि Juntei स्थायी file-system, registry, निष्पादन, उपयोगकर्ता-गतिविधि, resource-उपयोग
और security-product आर्टिफ़ैक्ट को पार्स करता है।

वही binary Windows, macOS और Linux पर चलता है। यह तब उपयोगी है जब किसी विश्लेषक को KAPE,
Velociraptor, disk-image या कोई अन्य offline collection मिलता है और वह हर आर्टिफ़ैक्ट के लिए अलग प्रोग्राम
के बजाय एक सुसंगत interface चाहता है।

## डिज़ाइन लक्ष्य { #design-goals }

- **बचाव योग्य parsing:** कच्ची values सुरक्षित रखें और चुपचाप data गढ़ने के बजाय क्षति की रिपोर्ट करें।
- **सभी के लिए उपयोगी:** डिफ़ॉल्ट उपयोग शुरुआती लोगों के लिए आसान होना चाहिए, साथ ही अनुभवी
  उपयोगकर्ताओं के लिए customizable और पूर्ण-सुविधा वाला भी।
- **Standalone binary:** चलाने के लिए किसी runtime library की आवश्यकता नहीं। यह बिना किसी तैयारी के
  सीधे काम करता है।
- **अत्यंत तेज़:** सबसे तेज़ parser बनने का लक्ष्य, साथ ही बहुत अधिक memory की
  आवश्यकता नहीं।
- **Cross-platform उपयोग:** Windows workstation के बिना Windows साक्ष्य का विश्लेषण करें।
- **Automation:** Spreadsheets और timeline tools के लिए CSV; `jq`, scripts और data lakes के लिए JSON/JSONL।
- **सुरक्षित विफलता:** Malformed records की रिपोर्ट की जाती है और जहाँ संभव हो उन्हें छोड़ दिया जाता है।
- **पारदर्शी वंशावली:** Juntei हर implementation के पीछे के open-source parsers और reference tools की पहचान करता है।

## Juntei क्या नहीं करता { #what-juntei-does-not-do }

Juntei live disk acquire नहीं करता और न ही साक्ष्य संरक्षण का स्थान लेता है, और वह केवल VHDX disk images को ही
mount करता है, read-only रूप में। यह
हर आर्टिफ़ैक्ट को स्वचालित रूप से एक ही incident कथा में नहीं जोड़ता। यह आपके द्वारा दिए गए
साक्ष्य को पार्स करता है और जाँच तथा correlation के लिए उपयुक्त records बनाता है।

Juntei आपके लिए विश्लेषण भी नहीं करेगा। साक्ष्य का विश्लेषण करने के लिए एक DFIR विशेषज्ञ आवश्यक है।

## लाइसेंस { #license }

Juntei
[Reciprocal Public License 1.5](https://github.com/Yamato-Security/juntei/blob/main/LICENSE)
(RPL-1.5) के अंतर्गत जारी किया गया है और इसका copyright Shirofune Security Co., Ltd. के पास है। Version 1.0.0 से पहले इसका source code
AGPLv3 के अंतर्गत प्रकाशित किया गया था। इसके निर्माण में प्रयुक्त crates और अन्य third-party code तथा data के
licenses
[`THIRD_PARTY_NOTICES`](https://github.com/Yamato-Security/juntei/tree/main/THIRD_PARTY_NOTICES) में हैं।
Juntei जिन projects पर आधारित है और जिनके विरुद्ध इसकी जाँच की गई, वे
[आभार](../acknowledgements.md) में सूचीबद्ध हैं।
