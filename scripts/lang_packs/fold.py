"""Match src/parse/normalize.rs fold_latin so compiled tokens survive tokenize."""


def fold_latin(text: str) -> str:
    out: list[str] = []
    for char in text:
        mapped = _FOLD.get(char)
        if mapped is not None:
            out.append(mapped)
        else:
            out.append(char.lower())
    return "".join(out)


_FOLD = {
    "ä": "ae",
    "Ä": "ae",
    "ö": "oe",
    "Ö": "oe",
    "ü": "ue",
    "Ü": "ue",
    "ß": "ss",
    "à": "a",
    "á": "a",
    "â": "a",
    "ã": "a",
    "å": "a",
    "ā": "a",
    "ă": "a",
    "ç": "c",
    "č": "c",
    "ć": "c",
    "è": "e",
    "é": "e",
    "ê": "e",
    "ë": "e",
    "ē": "e",
    "ė": "e",
    "ę": "e",
    "ì": "i",
    "í": "i",
    "î": "i",
    "ï": "i",
    "ī": "i",
    "į": "i",
    "ñ": "n",
    "ń": "n",
    "ň": "n",
    "ò": "o",
    "ó": "o",
    "ô": "o",
    "õ": "o",
    "ø": "o",
    "ō": "o",
    "ő": "o",
    "ù": "u",
    "ú": "u",
    "û": "u",
    "ū": "u",
    "ű": "u",
    "ý": "y",
    "ÿ": "y",
    "ž": "z",
    "ź": "z",
    "ż": "z",
    "š": "s",
    "ś": "s",
    "ł": "l",
    "đ": "d",
    "æ": "ae",
    "œ": "oe",
    "ı": "i",
    "ş": "s",
    "Ş": "s",
    "ğ": "g",
    "Ğ": "g",
    "ț": "t",
    "Ț": "t",
    "ș": "s",
    "Ș": "s",
}
