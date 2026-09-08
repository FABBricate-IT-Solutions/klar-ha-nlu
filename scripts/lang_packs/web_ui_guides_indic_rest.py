"""Page-guide chrome for remaining Indic Assist locales."""

from __future__ import annotations

from lang_packs.web_ui_guides import guides

PACKS: dict[str, dict[str, str]] = {
    "gu": guides(
        empty="કોઈ મોડલ નથી. id.",
        fail="મોડલ. સાફ કરો. id.",
        loading="મોડલ…",
        thinking="બંધ no-thinking (Lemonade/llama.cpp: chat_template_kwargs, OpenAI o-series: reasoning_effort none, Anthropic: omit thinking, Google: thinking_budget 0).",
        convos="Assist — વાતચીત.",
        evaluator="ઘર. ક્રમ (પહેલો મળતો વપરાશકર્તા નિયમ જીતે)",
        routines="બોલાયેલું નામ Home Assistant સ્ક્રિપ્ટ ચલાવે છે.",
        sentences="જાણીતા ઇરાદા પર વાક્ય બાંધો.",
        policies="નીતિઓ. ઘર નિયમો. ક્રમ (પહેલો મળતો વપરાશકર્તા નિયમ જીતે)",
        lab="વાક્ય ઉમેરો, વિશ્લેષણ / Enter.",
    ),
    "kn": guides(
        empty="ತೆರೆದ ಮಾದರಿ ಇಲ್ಲ. id.",
        fail="ಮಾದರಿ. ಅಳಿಸು. id.",
        loading="ಮಾದರಿ…",
        thinking="ಆಫ್ no-thinking (Lemonade/llama.cpp: chat_template_kwargs, OpenAI o-series: reasoning_effort none, Anthropic: omit thinking, Google: thinking_budget 0).",
        convos="Assist — ಸಂಭಾಷಣೆಗಳು.",
        evaluator="ಮನೆ. ಕ್ರಮ (ಮೊದಲು ಹೊಂದುವ ಬಳಕೆದಾರ ನಿಯಮ ಗೆಲ್ಲುತ್ತದೆ)",
        routines="ಹೇಳಿದ ಹೆಸರು Home Assistant ಲಿಪಿಯನ್ನು ಚಲಾಯಿಸುತ್ತದೆ.",
        sentences="ತಿಳಿದ ಉದ್ದೇಶಕ್ಕೆ ವಾಕ್ಯ ಕಟ್ಟಿ.",
        policies="ನೀತಿಗಳು. ಮನೆ ನಿಯಮಗಳು. ಕ್ರಮ (ಮೊದಲು ಹೊಂದುವ ಬಳಕೆದಾರ ನಿಯಮ ಗೆಲ್ಲುತ್ತದೆ)",
        lab="ವಾಕ್ಯ ಸೇರಿಸು, ವಿಶ್ಲೇಷಿಸು / Enter.",
    ),
    "ml": guides(
        empty="ഈ എൻഡ്‌പോയിന്റിൽ നിന്ന് മോഡലുകളില്ല. id ടൈപ്പ് ചെയ്യുക.",
        fail="മോഡലുകൾ ലിസ്റ്റ് ചെയ്യാനായില്ല. id ടൈപ്പ് ചെയ്യുക.",
        loading="മോഡലുകൾ ലോഡ് ചെയ്യുന്നു…",
        thinking="ഓഫ് ഓരോ ദാതാവിന്റെയും no-thinking കൊടി ഉപയോഗിക്കുന്നു (Lemonade/llama.cpp: chat_template_kwargs, OpenAI o പരമ്പര: reasoning_effort none, Anthropic: thinking ഒഴിവാക്കുക, Google: thinking_budget 0).",
        convos="Assist വഴി എന്തെങ്കിലും പറയുക. അത് ഇവിടെ കാണും.",
        evaluator="വീട്ടിലെന്നപോലെ ടൈപ്പ് ചെയ്യുക. ഫ്രെയിം ചെയ്ത ഘട്ടമാണ് പ്രവർത്തിച്ചത്.",
        routines="പറഞ്ഞ പേര് Home Assistant സ്ക്രിപ്റ്റ് തുടങ്ങുന്നു.",
        sentences="അറിയാവുന്ന ഉദ്ദേശത്തിലേക്ക് ഒരു വാക്യം. പിന്നെ പരീക്ഷിക്കുക.",
        policies="മൂന്ന് പാതകൾ. നിങ്ങളുടെ വീട്ടുനിയമം ആദ്യം ജയിക്കും. താഴെ ഒരു വാക്യം പരീക്ഷിക്കുക.",
        lab="ഒരു വാക്യം ടൈപ്പ് ചെയ്യുക, പിന്നെ വിശകലനം അല്ലെങ്കിൽ Enter.",
    ),
    "te": guides(
        empty="తెరిచిన మోడల్ లేదు. id.",
        fail="మోడల్. తుడుచు. id.",
        loading="మోడల్…",
        thinking="ఆఫ్ no-thinking (Lemonade/llama.cpp: chat_template_kwargs, OpenAI o-series: reasoning_effort none, Anthropic: omit thinking, Google: thinking_budget 0).",
        convos="Assist — సంభాషణలు.",
        evaluator="ఇల్లు. క్రమం (మొదట సరిపోయే వాడుకరి నియమం గెలుస్తుంది)",
        routines="చెప్పిన పేరు Home Assistant లిపిని నడుపుతుంది.",
        sentences="తెలిసిన ఉద్దేశానికి వాక్యం బంధించండి.",
        policies="విధానాలు. ఇల్లు నియమాలు. క్రమం (మొదట సరిపోయే వాడుకరి నియమం గెలుస్తుంది)",
        lab="వాక్యం జోడించు, విశ్లేషించు / Enter.",
    ),
    "pa": guides(
        empty="ਕੋਈ ਮਾਡਲ ਨਹੀਂ. id.",
        fail="ਮਾਡਲ. ਸਾਫ਼ ਕਰੋ. id.",
        loading="ਮਾਡਲ…",
        thinking="ਬੰਦ no-thinking (Lemonade/llama.cpp: chat_template_kwargs, OpenAI o-series: reasoning_effort none, Anthropic: omit thinking, Google: thinking_budget 0).",
        convos="Assist — ਗੱਲਬਾਤ.",
        evaluator="ਘਰ. ਕ੍ਰਮ (ਪਹਿਲਾ ਮਿਲਦਾ ਵਰਤੋਂਕਾਰ ਨਿਯਮ ਜਿੱਤਦਾ ਹੈ)",
        routines="ਬੋਲਿਆ ਨਾਂ Home Assistant ਲਿਪੀ ਚਲਾਉਂਦਾ ਹੈ।",
        sentences="ਜਾਣੇ ਇਰਾਦੇ ਉੱਤੇ ਵਾਕ ਬੰਨ੍ਹੋ।",
        policies="ਨੀਤੀਆਂ. ਘਰ ਨਿਯਮ. ਕ੍ਰਮ (ਪਹਿਲਾ ਮਿਲਦਾ ਵਰਤੋਂਕਾਰ ਨਿਯਮ ਜਿੱਤਦਾ ਹੈ)",
        lab="ਵਾਕ ਜੋੜੋ, ਵਿਸ਼ਲੇਸ਼ਣ / Enter.",
    ),
}
