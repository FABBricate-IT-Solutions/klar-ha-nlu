"""Generate YAML voice tests for Klar — German and English, same families."""

from .lib import HOME, ROOT, dump
from .wohnung import de_area, de_assist, de_devices, de_spoken, en


def main() -> None:
    de = ROOT / "wohnung_mittel"
    en_root = ROOT / "wohnung_en"
    dump(de / "home_config.yaml", HOME)
    dump(en_root / "home_config.yaml", HOME.replace("language: de", "language: en"))
    de_area.write(de)
    de_devices.write(de)
    de_spoken.write(de)
    de_assist.write(de)
    en.write(en_root)
    print(f"wrote {de} and {en_root}")


if __name__ == "__main__":
    main()
