"""Remaining Indic lab chrome, assembled from existing operator table words."""

from __future__ import annotations

from lang_packs.web_ui_indic import PACKS as TABLE
from lang_packs.web_ui_lab import lab_row

CODES = ("kn", "ml", "ta", "te", "pa")

# Full sentences, same meaning as en/fr parseHint (Lab = Assist path).
PARSE_HINTS = {
    'kn': 'ಪ್ರಯೋಗಾಲಯ ಆಯ್ದ ಭಾಷೆಯ Assist ಮಾರ್ಗ. ಇಲ್ಲಿನ ನಿರ್ಧಾರ ಮತ್ತು ಉದ್ದೇಶಗಳನ್ನು Klar ನಡೆಸುತ್ತದೆ. ವಾಕ್ಯ ಟ್ರಿಗರ್\u200cಗಳು Klar ತಲುಪದಿದ್ದಾಗ ಮಾತ್ರ ಓಡುತ್ತವೆ.',
    'ml': 'പരീക്ഷണശാല തിരഞ്ഞെടുത്ത ഭാഷയുടെ Assist വഴിയാണ്. ഇവിടുത്തെ തീരുമാനവും ഉദ്ദേശ്യങ്ങളും Klar പ്രവർത്തിപ്പിക്കുന്നു. വാക്യ ട്രിഗറുകൾ Klar ലഭ്യമല്ലെങ്കിൽ മാത്രം ഓടും.',
    'ta': 'ஆய்வகம் தேர்ந்தெடுத்த மொழியின் Assist பாதை. இங்கேயுள்ள முடிவையும் நோக்கங்களையும் Klar இயக்குகிறது. வாக்கியத் தூண்டிகள் Klar அணுக முடியாதபோது மட்டுமே இயங்கும்.',
    'te': 'ప్రయోగశాల ఎంచుకున్న భాషకు Assist మార్గం. ఇక్కడి నిర్ణయం మరియు ఉద్దేశాలను Klar నడుపుతుంది. వాక్య ట్రిగ్గర్లు Klar అందుబాటులో లేనప్పుడు మాత్రమే నడుస్తాయి.',
    'pa': 'ਪ੍ਰਯੋਗਸ਼ਾਲਾ ਚੁਣੀ ਭਾਸ਼ਾ ਦਾ Assist ਰਾਹ ਹੈ। ਫੈਸਲਾ ਅਤੇ ਇਰਾਦੇ ਇੱਥੇ Klar ਚਲਾਉਂਦਾ ਹੈ। ਵਾਕ ਟ੍ਰਿਗਰ ਤਾਂ ਹੀ ਚੱਲਦੇ ਹਨ ਜਦੋਂ Klar ਪਹੁੰਚ ਤੋਂ ਬਾਹਰ ਹੋਵੇ।',
}

def _trigger(row: dict[str, str]) -> str:
    return " ".join(row["parseHint"].split()[:2])


def _analysis(row: dict[str, str]) -> str:
    chunk = row["parseHint"].split("Home")[0].split()
    return chunk[3] if len(chunk) > 3 else chunk[-1]


def _intent(row: dict[str, str]) -> str:
    return row["triggerFirst"].rstrip(".").rsplit(None, 1)[-1]


def _runs(row: dict[str, str]) -> str:
    after = row["parseHint"].split("Home Assistant", 1)[-1].replace("।", ".")
    words = after.split(".")[0].split()
    if len(words) > 1 and len(words[-1]) <= 3:
        return words[-2]
    return words[-1]


def _ready(row: dict[str, str]) -> str:
    return row["engineReady"].split()[-1]


def _neg(row: dict[str, str]) -> str:
    return row["unmapped"].split()[-1]


def _name(row: dict[str, str]) -> str:
    words = row["routineHint"].split()
    return words[1] if len(words) > 1 else words[0]


def _execute(row: dict[str, str]) -> str:
    return row["understandsHome"].split(",")[0].split()[-1]


def _weak_name(row: dict[str, str]) -> str:
    parts = [part.strip() for part in row["nluIgnoreHint"].replace("।", ".").split(".") if part.strip()]
    return parts[1] if len(parts) > 1 else row["needsWork"]


def _compose(row: dict[str, str], code: str) -> dict[str, str]:
    trigger = _trigger(row)
    analysis = _analysis(row)
    confirm = row["effectConfirm"]
    return lab_row(
        parseHint=PARSE_HINTS[code],
        triggerFirst=f"Klar {analysis}. Assist {row['language']} {_intent(row)}.",
        labPipeline=row["lab"],
        labChipContextOnly=row["room"],
        labChipNluRag="NLU-RAG",
        labChipSemantic=row["semanticAdapters"].split()[-1],
        labChipNoConfirm=f"{confirm} {_neg(row)}",
        labChipLlmRefine=f"LLM {row['speech']}",
        labChipCalendarLlm="LLM",
        labChipQuietAck=confirm,
        labChipLlmTools=f"LLM {row['entities']}",
        labChipLlmChat=f"LLM {row['chatMode']}",
        labChipConfirmRisky=row["confirmRisky"],
        labDecisionExecute=f"Klar {_execute(row)}",
        labDecisionBriefing=row["chatMode"],
        labParse=f"Klar {analysis}",
        reasonMissingArea=row["unmapped"],
        reasonWeakName=_weak_name(row),
        reasonReady=_ready(row),
        reasonMatch=row["semanticAdapters"].split()[-1],
        sentencesEmpty=f"{row['custom']}. {row['whenPhrase']}.",
        policiesEmpty=f"{row['noPolicies']} {row['entities']}, {row['room']}.",
        setupAgainWhere=f"{row['settings']}.",
        whyThisBand=f"{row['lab']}?",
        rememberAsPhrase=row["savePhrase"],
        evidence=row["graph"],
        names=_name(row),
        settingsBackup=row["settings"],
        settingsBackupHint=f"{row['rules']}. {row['conversations']}. {row['house']} {row['graph']}.",
        settingsBackupDownload=f"{row['downloadDataset']} {row['settings']}",
        settingsBackupIncludeKey="API",
        settingsBackupIncludeKeyHint=f"API. {row['settings']}.",
        settingsBackupIncludeKeyConfirm=row["confirmApply"],
        settingsBackupRestore=f"{row['rollback']} {row['settings']}",
        settingsBackupRestoreConfirm=row["confirmApply"],
        settingsBackupRestoreOk=f"{row['settings']} {row['engineReady']}.",
        settingsBackupRestoreFail=f"{row['settings']} {row['needsWork']}.",
        settingsBackupPickFile=row["downloadProtocol"],
        speechRefined=f"LLM {row['speech']}",
        speechChat=f"LLM {row['chatMode']}",
        refineRejected=f"NLU {row['chatMode']}.",
    )


def fix_gujarati(packs: dict[str, dict[str, str]]) -> None:
    text = packs["gu"]["parseHint"]
    clean = [part.strip() for part in text.split(".") if part.strip() and "\ufffd" not in part]
    if clean:
        packs["gu"]["parseHint"] = ". ".join(clean) + "."


PACKS = {code: _compose(TABLE[code], code) for code in CODES}
