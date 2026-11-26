# Beta gold sentences (package 4.3)

Hand-curated natural sentences from the plan evidence probe
(`klar-nlu-plan-evidence/lang/natural.py`). Not speaker-verified end-to-end;
full ~40 speaker-checked sentences per Beta locale remain a gap.

These fixtures are the non-tautological gate for Beta (Q-02): they are not
generated from pack lexicons by `scripts/parity/`.

Known gaps left out of the hard family gate (still useful as probes):

- hi: `रसोई की बत्ती बुझा दो` currently resolves as LightSet
- MediaPause cases may return `KlarNoMusicPlayer` on the default home
  (accepted as media family)
