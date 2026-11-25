"""Load-models / retry / API-key-required chrome for every Assist locale."""

from __future__ import annotations

KEYS = (
    "llmLoadModels",
    "llmApiKeyRequired",
    "llmRetry",
)


def actions(*, load: str, key_required: str, retry: str) -> dict[str, str]:
    return {
        "llmLoadModels": load,
        "llmApiKeyRequired": key_required,
        "llmRetry": retry,
    }


# Hand translations for the 65 generated locales (de/en live in TypeScript).
PACKS: dict[str, dict[str, str]] = {
    "af": actions(
        load="Laai modelle",
        key_required="’n API-sleutel word vir hierdie verskaffer vereis.",
        retry="Probeer weer",
    ),
    "ar": actions(
        load="تحميل النماذج",
        key_required="مطلوب مفتاح API لهذا الموفر.",
        retry="إعادة المحاولة",
    ),
    "bg": actions(
        load="Зареди модели",
        key_required="За този доставчик е необходим API ключ.",
        retry="Опитай отново",
    ),
    "bn": actions(
        load="মডেল লোড করুন",
        key_required="এই প্রদানকারীর জন্য একটি API কী প্রয়োজন।",
        retry="আবার চেষ্টা",
    ),
    "ca": actions(
        load="Carrega models",
        key_required="Cal una clau API per a aquest proveïdor.",
        retry="Torna-ho a provar",
    ),
    "cs": actions(
        load="Načíst modely",
        key_required="Pro tohoto poskytovatele je vyžadován API klíč.",
        retry="Zkusit znovu",
    ),
    "cy": actions(
        load="Llwytho modelau",
        key_required="Mae angen allwedd API ar gyfer y darparwr hwn.",
        retry="Ceisio eto",
    ),
    "da": actions(
        load="Indlæs modeller",
        key_required="En API-nøgle er påkrævet for denne udbyder.",
        retry="Prøv igen",
    ),
    "de-AT": actions(
        load="Modelle laden",
        key_required="Für diesen Anbieter ist ein API-Key erforderlich.",
        retry="Erneut versuchen",
    ),
    "de-CH": actions(
        load="Modelle laden",
        key_required="Für diesen Anbieter ist ein API-Key erforderlich.",
        retry="Erneut versuchen",
    ),
    "el": actions(
        load="Φόρτωση μοντέλων",
        key_required="Απαιτείται κλειδί API για αυτόν τον πάροχο.",
        retry="Επανάληψη",
    ),
    "en-GB": actions(
        load="Load models",
        key_required="An API key is required for this provider.",
        retry="Retry",
    ),
    "es": actions(
        load="Cargar modelos",
        key_required="Se requiere una clave API para este proveedor.",
        retry="Reintentar",
    ),
    "et": actions(
        load="Laadi mudelid",
        key_required="Selle teenusepakkuja jaoks on vaja API võtit.",
        retry="Proovi uuesti",
    ),
    "eu": actions(
        load="Kargatu modeloak",
        key_required="Hornitzaile honetarako API gakoa behar da.",
        retry="Saiatu berriro",
    ),
    "fa": actions(
        load="بارگذاری مدل‌ها",
        key_required="برای این ارائه‌دهنده کلید API لازم است.",
        retry="تلاش دوباره",
    ),
    "fi": actions(
        load="Lataa mallit",
        key_required="Tämä palveluntarjoaja vaatii API-avaimen.",
        retry="Yritä uudelleen",
    ),
    "fr": actions(
        load="Charger les modèles",
        key_required="Une clé API est requise pour ce fournisseur.",
        retry="Réessayer",
    ),
    "ga": actions(
        load="Luchtaigh samhlacha",
        key_required="Tá eochair API ag teastáil don soláthraí seo.",
        retry="Bain triail eile as",
    ),
    "gl": actions(
        load="Cargar modelos",
        key_required="Requírese unha clave API para este provedor.",
        retry="Tentar de novo",
    ),
    "gu": actions(
        load="મોડેલ લોડ કરો",
        key_required="આ પ્રદાતા માટે API કી જરૂરી છે.",
        retry="ફરી પ્રયાસ",
    ),
    "he": actions(
        load="טען מודלים",
        key_required="נדרש מפתח API עבור ספק זה.",
        retry="נסה שוב",
    ),
    "hi": actions(
        load="मॉडल लोड करें",
        key_required="इस प्रदाता के लिए API कुंजी आवश्यक है।",
        retry="फिर से कोशिश करें",
    ),
    "hr": actions(
        load="Učitaj modele",
        key_required="Za ovog pružatelja potreban je API ključ.",
        retry="Pokušaj ponovno",
    ),
    "hu": actions(
        load="Modellek betöltése",
        key_required="Ehhez a szolgáltatóhoz API-kulcs szükséges.",
        retry="Újra",
    ),
    "hy": actions(
        load="Բեռնել մոդելները",
        key_required="Այս մատակարարի համար պահանջվում է API բանալի։",
        retry="Կրկին փորձել",
    ),
    "id": actions(
        load="Muat model",
        key_required="Kunci API diperlukan untuk penyedia ini.",
        retry="Coba lagi",
    ),
    "is": actions(
        load="Hlaða líkönum",
        key_required="API-lykill er nauðsynlegur fyrir þennan veitanda.",
        retry="Reyna aftur",
    ),
    "it": actions(
        load="Carica modelli",
        key_required="Per questo provider è richiesta una chiave API.",
        retry="Riprova",
    ),
    "ja": actions(
        load="モデルを読み込む",
        key_required="このプロバイダーには API キーが必要です。",
        retry="再試行",
    ),
    "ka": actions(
        load="მოდელების ჩატვირთვა",
        key_required="ამ პროვაიდერისთვის საჭიროა API გასაღები.",
        retry="ხელახლა ცდა",
    ),
    "kn": actions(
        load="ಮಾದರಿಗಳನ್ನು ಲೋಡ್ ಮಾಡಿ",
        key_required="ಈ ಪೂರೈಕೆದಾರರಿಗೆ API ಕೀ ಅಗತ್ಯವಿದೆ.",
        retry="ಮತ್ತೆ ಪ್ರಯತ್ನಿಸಿ",
    ),
    "ko": actions(
        load="모델 불러오기",
        key_required="이 공급자에는 API 키가 필요합니다.",
        retry="다시 시도",
    ),
    "kw": actions(
        load="Karga modellys",
        key_required="Yma edhom a alhwedh API rag an porthyer ma.",
        retry="Assaya arta",
    ),
    "lb": actions(
        load="Modeller lueden",
        key_required="Fir dëse Provider brauchs du en API-Schlëssel.",
        retry="Nach eng Kéier",
    ),
    "lt": actions(
        load="Įkelti modelius",
        key_required="Šiam teikėjui reikalingas API raktas.",
        retry="Bandyti dar kartą",
    ),
    "lv": actions(
        load="Ielādēt modeļus",
        key_required="Šim pakalpojumu sniedzējam nepieciešama API atslēga.",
        retry="Mēģināt vēlreiz",
    ),
    "ml": actions(
        load="മോഡലുകൾ ലോഡ് ചെയ്യുക",
        key_required="ഈ ദാതാവിന് API കീ ആവശ്യമാണ്.",
        retry="വീണ്ടും ശ്രമിക്കുക",
    ),
    "mn": actions(
        load="Модел ачаалах",
        key_required="Энэ нийлүүлэгчид API түлхүүр шаардлагатай.",
        retry="Дахин оролдох",
    ),
    "mr": actions(
        load="मॉडेल लोड करा",
        key_required="या प्रदात्यासाठी API की आवश्यक आहे.",
        retry="पुन्हा प्रयत्न",
    ),
    "ms": actions(
        load="Muat model",
        key_required="Kunci API diperlukan untuk pembekal ini.",
        retry="Cuba lagi",
    ),
    "nb": actions(
        load="Last inn modeller",
        key_required="En API-nøkkel kreves for denne leverandøren.",
        retry="Prøv igjen",
    ),
    "ne": actions(
        load="मोडेल लोड गर्नुहोस्",
        key_required="यो प्रदायकको लागि API कुञ्जी आवश्यक छ।",
        retry="फेरि प्रयास",
    ),
    "nl": actions(
        load="Modellen laden",
        key_required="Voor deze provider is een API-sleutel vereist.",
        retry="Opnieuw proberen",
    ),
    "pa": actions(
        load="ਮਾਡਲ ਲੋਡ ਕਰੋ",
        key_required="ਇਸ ਪ੍ਰਦਾਤਾ ਲਈ API ਕੁੰਜੀ ਲੋੜੀਂਦੀ ਹੈ।",
        retry="ਦੁਬਾਰਾ ਕੋਸ਼ਿਸ਼",
    ),
    "pl": actions(
        load="Wczytaj modele",
        key_required="Dla tego dostawcy wymagany jest klucz API.",
        retry="Spróbuj ponownie",
    ),
    "pt": actions(
        load="Carregar modelos",
        key_required="É necessária uma chave API para este fornecedor.",
        retry="Tentar novamente",
    ),
    "pt-BR": actions(
        load="Carregar modelos",
        key_required="Uma chave de API é necessária para este provedor.",
        retry="Tentar de novo",
    ),
    "ro": actions(
        load="Încarcă modelele",
        key_required="Pentru acest furnizor este necesară o cheie API.",
        retry="Reîncearcă",
    ),
    "sk": actions(
        load="Načítať modely",
        key_required="Pre tohto poskytovateľa je potrebný API kľúč.",
        retry="Skúsiť znova",
    ),
    "sl": actions(
        load="Naloži modele",
        key_required="Za tega ponudnika je potreben API ključ.",
        retry="Poskusi znova",
    ),
    "sr": actions(
        load="Учитај моделе",
        key_required="За овог пружаоца је потребан API кључ.",
        retry="Покушај поново",
    ),
    "sr-Latn": actions(
        load="Učitaj modele",
        key_required="Za ovog pružaoca je potreban API ključ.",
        retry="Pokušaj ponovo",
    ),
    "sv": actions(
        load="Ladda modeller",
        key_required="En API-nyckel krävs för den här leverantören.",
        retry="Försök igen",
    ),
    "sw": actions(
        load="Pakia modeli",
        key_required="Ufunguo wa API unahitajika kwa mtoa huduma huyu.",
        retry="Jaribu tena",
    ),
    "ta": actions(
        load="மாடல்களை ஏற்று",
        key_required="இந்த வழங்குநருக்கு API விசை தேவை.",
        retry="மீண்டும் முயற்சி",
    ),
    "te": actions(
        load="మోడల్‌లను లోడ్ చేయి",
        key_required="ఈ ప్రొవైడర్‌కు API కీ అవసరం.",
        retry="మళ్లీ ప్రయత్నించు",
    ),
    "th": actions(
        load="โหลดโมเดล",
        key_required="ผู้ให้บริการนี้ต้องมีคีย์ API",
        retry="ลองอีกครั้ง",
    ),
    "tr": actions(
        load="Modelleri yükle",
        key_required="Bu sağlayıcı için bir API anahtarı gerekir.",
        retry="Yeniden dene",
    ),
    "uk": actions(
        load="Завантажити моделі",
        key_required="Для цього постачальника потрібен API-ключ.",
        retry="Спробувати знову",
    ),
    "ur": actions(
        load="ماڈلز لوڈ کریں",
        key_required="اس فراہم کنندہ کے لیے API کلید درکار ہے۔",
        retry="دوبارہ کوشش",
    ),
    "vi": actions(
        load="Tải mô hình",
        key_required="Nhà cung cấp này yêu cầu khóa API.",
        retry="Thử lại",
    ),
    "zh-CN": actions(
        load="加载模型",
        key_required="此提供商需要 API 密钥。",
        retry="重试",
    ),
    "zh-HK": actions(
        load="載入模型",
        key_required="此供應商需要 API 金鑰。",
        retry="重試",
    ),
    "zh-TW": actions(
        load="載入模型",
        key_required="此供應商需要 API 金鑰。",
        retry="重試",
    ),
}


def apply_llm_actions_copy(packs: dict[str, dict[str, str]]) -> None:
    missing = sorted(set(packs) - set(PACKS))
    if missing:
        raise SystemExit(f"llm actions chrome missing locales: {missing}")
    extra = sorted(set(PACKS) - set(packs))
    if extra:
        raise SystemExit(f"llm actions chrome extra locales: {extra}")
    for code, fields in packs.items():
        row = PACKS[code]
        absent = [key for key in KEYS if key not in row]
        if absent:
            raise SystemExit(f"{code}: llm actions missing keys {absent}")
        fields.update(row)
