"""AI Task entity name for every Home Assistant UI locale."""

from __future__ import annotations

NAMES = {
    "de": "KI-Aufgabe",
    "de-CH": "KI-Ufgaab",
    "de-AT": "KI-Aufgabe",
    "en-GB": "AI Task",
    "fr": "Tâche IA",
    "nl": "AI-taak",
    "es": "Tarea de IA",
    "it": "Attività IA",
    "pt": "Tarefa de IA",
    "pt-BR": "Tarefa de IA",
    "da": "AI-opgave",
    "nb": "KI-oppgave",
    "sv": "AI-uppgift",
    "fi": "Tekoälytehtävä",
    "pl": "Zadanie SI",
    "cs": "Úloha AI",
    "ca": "Tasca d'IA",
    "ro": "Sarcină IA",
    "hu": "MI-feladat",
    "hr": "AI zadatak",
    "sl": "AI-naloga",
    "sk": "Úloha AI",
    "bg": "Задача с ИИ",
    "el": "Εργασία ΤΝ",
    "sr": "АИ задатак",
    "sr-Latn": "AI zadatak",
    "uk": "Завдання ШІ",
    "zh-CN": "AI 任务",
    "zh-TW": "AI 任務",
    "zh-HK": "AI 任務",
    "ja": "AIタスク",
    "ko": "AI 작업",
    "th": "งาน AI",
    "vi": "Tác vụ AI",
    "id": "Tugas AI",
    "ms": "Tugas AI",
    "hi": "AI कार्य",
    "bn": "AI কাজ",
    "gu": "AI કાર્ય",
    "kn": "AI ಕಾರ್ಯ",
    "ml": "AI ടാസ്‌ക്",
    "mr": "AI कार्य",
    "ta": "AI பணி",
    "te": "AI పని",
    "pa": "AI ਕਾਰਜ",
    "ne": "AI कार्य",
    "ar": "مهمة ذكاء اصطناعي",
    "he": "משימת בינה מלאכותית",
    "fa": "وظیفه هوش مصنوعی",
    "ur": "مصنوعی ذہانت کا کام",
    "tr": "YZ görevi",
    "hy": "ԱԲ առաջադրանք",
    "ka": "ხელოვნური ინტელექტის ამოცანა",
    "mn": "Хиймэл оюуны даалгавар",
    "af": "KI-taak",
    "sw": "Kazi ya AI",
    "cy": "Tasg AI",
    "et": "TI ülesanne",
    "eu": "IA ataza",
    "ga": "Tasc AI",
    "gl": "Tarefa de IA",
    "is": "Gervigreindarverkefni",
    "lb": "KI-Aufgab",
    "kw": "Towl AI",
    "lt": "DI užduotis",
    "lv": "MI uzdevums",
}


def apply_ai_task_names(packs: dict[str, dict[str, str]]) -> None:
    missing = sorted(set(packs) - set(NAMES))
    extra = sorted(set(NAMES) - set(packs))
    if missing or extra:
        raise SystemExit(f"ai_task names missing={missing} extra={extra}")
    for code, fields in packs.items():
        name = NAMES[code]
        if "\ufffd" in name:
            raise SystemExit(f"{code}: ai_task name is garbled")
        fields["ai_task_name"] = name
