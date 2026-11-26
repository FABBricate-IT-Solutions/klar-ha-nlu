# Phrase in drei Schritten korrigieren

[Deutsch](fix-phrase.md) · [English](en/fix-phrase.md)

Generierte Assist-Locales liegen als **Datendateien** unter
`packs/generated/{code}.yaml` (lesen/diffen) und als kompiliertes Rust unter
`src/lang/packs/` (Engine). Generiertes Rust **nicht** von Hand ändern. Eine
Pack-Korrektur berührt **eine** Datei: `packs/assist/{code}.yaml`.

## Drei Schritte

1. **Reproduzieren** des falschen Parse:

   ```bash
   cargo run --quiet -- lang preview --language fr --text "mode cinema"
   ```

2. **Korrigieren** in der Assist-Datendatei (CLI legt sie bei Bedarf an):

   ```bash
   cargo run --quiet -- lang new-case \
     --language fr \
     --phrase "mode cinema" \
     --intent HassTurnOn \
     --slot entity_id=scene.cinema
   ```

   Oder `packs/assist/fr.yaml` von Hand editieren — Verb, Lexikon-Set oder Intent.
   Für den Fix muss nur diese eine Datei geändert werden.

3. **Prüfen** mit Preview und den Localesuiten:

   ```bash
   cargo run --quiet -- lang preview --language fr --text "mode cinema" --pack packs/assist/fr.yaml
   cargo nextest run --locked -E 'test(/assist_langs|language/)'
   ```

## Was wohin gehört

| Pfad | Rolle |
|------|------|
| `packs/assist/{code}.yaml` | Phrase-Korrekturen — **eine Datei je Locale**; der Generator überschreibt sie nie |
| `packs/generated/{code}.yaml` | Generiertes Pack als Daten (lesen/diffen); wird neu geschrieben; nicht von Hand ändern |
| `src/lang/packs/{mod}/` | Kompilierte Engine-Tabellen aus dem Generator — nicht von Hand ändern |
| `scripts/lang_packs/` | Lexikon-Quelle für eine **neue** Locale oder einen Bulk-Refresh |

Deutsch und Englisch bleiben handgeschrieben unter `src/lang/packs/de/` und `…/en/`.
