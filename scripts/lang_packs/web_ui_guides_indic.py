"""Page-guide chrome for Indic Assist locales and Swahili."""

from __future__ import annotations

from lang_packs.web_ui_guides import guides
from lang_packs.web_ui_guides_indic_rest import PACKS as REST

PACKS: dict[str, dict[str, str]] = {
    "hi": guides(
        empty="इस एंडपॉइंट से कोई मॉडल नहीं। एक id लिखें।",
        fail="मॉडल सूची नहीं मिली। एक id लिखें।",
        loading="मॉडल लोड हो रहे हैं…",
        thinking="बंद प्रत्येक प्रदाता का no-thinking ध्वज इस्तेमाल करता है (Lemonade/llama.cpp: chat_template_kwargs, OpenAI o श्रृंखला: reasoning_effort none, Anthropic: thinking छोड़ें, Google: thinking_budget 0)।",
        convos="Assist से कुछ कहें। वह यहाँ दिखेगा।",
        evaluator="जैसा घर पर बोलते हैं वैसा लिखें। फ्रेम वाला चरण वही है जो चला।",
        routines="बोला गया नाम Home Assistant स्क्रिप्ट चलाता है।",
        sentences="एक वाक्य ज्ञात आशय पर। फिर आज़माएँ।",
        policies="तीन पथ। आपका घर का नियम पहले जीतता है। नीचे एक वाक्य आज़माएँ।",
        lab="एक वाक्य लिखें, फिर विश्लेषण या Enter।",
    ),
    "bn": guides(
        empty="এই এন্ডপয়েন্ট থেকে কোনো মডেল নেই। একটি id লিখুন।",
        fail="মডেল তালিকা করা যায়নি। একটি id লিখুন।",
        loading="মডেল লোড হচ্ছে…",
        thinking="বন্ধ প্রতিটি প্রদানকারীর no-thinking পতাকা ব্যবহার করে (Lemonade/llama.cpp: chat_template_kwargs, OpenAI o সিরিজ: reasoning_effort none, Anthropic: thinking বাদ, Google: thinking_budget 0)।",
        convos="Assist দিয়ে কিছু বলুন। এখানে দেখা যাবে।",
        evaluator="বাড়িতে যেমন বলেন তেমন লিখুন। ফ্রেম করা ধাপটিই চালু হয়েছে।",
        routines="বলা নাম Home Assistant স্ক্রিপ্ট চালায়।",
        sentences="একটি বাক্য পরিচিত অভিপ্রায়ে। তারপর চেষ্টা করুন।",
        policies="তিনটি পথ। আপনার ঘরের নিয়ম আগে জেতে। নিচে একটি বাক্য চেষ্টা করুন।",
        lab="একটি বাক্য লিখুন, তারপর বিশ্লেষণ বা Enter।",
    ),
    "mr": guides(
        empty="या एंडपॉइंटवरून मॉडेल नाहीत. id लिहा.",
        fail="मॉडेल यादी करता आली नाही. id लिहा.",
        loading="मॉडेल लोड होत आहेत…",
        thinking="बंद प्रत्येक पुरवठादाराचा no-thinking ध्वज वापरतो (Lemonade/llama.cpp: chat_template_kwargs, OpenAI o मालिका: reasoning_effort none, Anthropic: thinking वगळा, Google: thinking_budget 0).",
        convos="Assist मधून काहीतरी सांगा. ते इथे दिसेल.",
        evaluator="घरी बोलता तसे लिहा. चौकटीतील पायरी तीच जी चालली.",
        routines="बोललेले नाव Home Assistant स्क्रिप्ट सुरू करते.",
        sentences="ओळखीच्या हेतूवर एक वाक्य. मग वापरून पहा.",
        policies="तीन मार्ग. तुमचा घरगुती नियम आधी जिंकतो. खाली वाक्य वापरून पहा.",
        lab="एक वाक्य लिहा, मग विश्लेषण किंवा Enter.",
    ),
    "ta": guides(
        empty="இந்த முனையத்திலிருந்து மாதிரிகள் இல்லை. id தட்டச்சு செய்யவும்.",
        fail="மாதிரிகளைப் பட்டியலிட முடியவில்லை. id தட்டச்சு செய்யவும்.",
        loading="மாதிரிகள் ஏற்றப்படுகின்றன…",
        thinking="அணைப்பு ஒவ்வொரு வழங்குநரின் no-thinking கொடியைப் பயன்படுத்தும் (Lemonade/llama.cpp: chat_template_kwargs, OpenAI o தொடர்: reasoning_effort none, Anthropic: thinking விடு, Google: thinking_budget 0).",
        convos="Assist வழியாக ஏதேனும் சொல்லுங்கள். அது இங்கே தெரியும்.",
        evaluator="வீட்டில் சொல்வது போல் தட்டச்சு செய்யுங்கள். சட்டமிட்ட படிதான் இயங்கியது.",
        routines="சொல்லப்பட்ட பெயர் Home Assistant ஸ்கிரிப்டைத் தொடங்கும்.",
        sentences="தெரிந்த நோக்கத்துக்கு ஒரு வாக்கியம். பிறகு முயல்க.",
        policies="மூன்று பாதைகள். உங்கள் வீட்டு விதி முதலில் வெல்லும். கீழே ஒரு வாக்கியம் முயல்க.",
        lab="ஒரு வாக்கியம் தட்டச்சு செய்து, பகுப்பாய்வு அல்லது Enter.",
    ),
    "ne": guides(
        empty="यो एन्डपोइन्टबाट मोडेल छैनन्। id लेख्नुहोस्।",
        fail="मोडेल सूची बनाउन सकिएन। id लेख्नुहोस्।",
        loading="मोडेल लोड हुँदैछन्…",
        thinking="बन्द प्रत्येक प्रदायकको no-thinking झन्डा प्रयोग गर्छ (Lemonade/llama.cpp: chat_template_kwargs, OpenAI o श्रृंखला: reasoning_effort none, Anthropic: thinking छाड्नुहोस्, Google: thinking_budget 0)।",
        convos="Assist मार्फत केही भन्नुहोस्। त्यो यहाँ देखिन्छ।",
        evaluator="घरमा जस्तै लेख्नुहोस्। फ्रेम भएको चरण त्यही हो जुन चल्यो।",
        routines="बोलिएको नाम Home Assistant स्क्रिप्ट सुरु गर्छ।",
        sentences="चिनेको आशयमा एउटा वाक्य। अनि प्रयास गर्नुहोस्।",
        policies="तीन पथ। तपाईंको घरको नियम पहिले जित्छ। तल वाक्य प्रयास गर्नुहोस्।",
        lab="एउटा वाक्य लेख्नुहोस्, अनि विश्लेषण वा Enter।",
    ),
    "sw": guides(
        empty="Hakuna modeli kutoka endpoint hii. Andika id.",
        fail="Imeshindwa kuorodhesha modeli. Andika id.",
        loading="Inapakia modeli…",
        thinking="Zima hutumia bendera ya no-thinking ya mtoa huduma (Lemonade/llama.cpp: chat_template_kwargs, mfululizo wa o wa OpenAI: reasoning_effort none, Anthropic: ruka thinking, Google: thinking_budget 0).",
        convos="Sema kitu kupitia Assist. Kitaonekana hapa.",
        evaluator="Andika kama nyumbani. Hatua yenye fremu ndiyo iliyowaka.",
        routines="Jina lililosemwa huanza skripti ya Home Assistant.",
        sentences="Sentensi moja kwenye nia inayojulikana. Kisha jaribu.",
        policies="Njia tatu. Sheria yako ya nyumba inashinda kwanza. Jaribu sentensi chini.",
        lab="Andika sentensi, kisha Chambua au Enter.",
    ),
}
PACKS.update(REST)
