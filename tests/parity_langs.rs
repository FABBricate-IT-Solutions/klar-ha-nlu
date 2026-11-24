//! Per-locale parity. `parity_*` is the full rubric (apartment, family home, m0, m2).
//! `parity_own_*` is Klar's own apartment suite alone and runs on every pull request.
//! `parity_conversation_*` is `#[ignore]`: generated locales are not at zero, each has a
//! recorded ceiling in `tests/fixtures/suite_bounds.yaml`.
mod voice_suite_support;

use voice_suite_support::bounds::assert_within_bound;
use voice_suite_support::parity::{run_parity, run_parity_conversation, run_parity_own};

macro_rules! parity {
    ($name:ident, $own:ident, $conversation:ident, $code:literal) => {
        #[test]
        fn $name() {
            let stats = run_parity($code);
            assert!(stats.ok + stats.fail > 0, "{} has no parity cases", $code);
            assert_eq!(stats.fail, 0, "{} parity failures:\n{}", $code, stats.fails.join("\n"));
        }
        #[test]
        fn $own() {
            let stats = run_parity_own($code);
            assert!(stats.ok + stats.fail > 0, "{} has no own parity cases", $code);
            assert_eq!(stats.fail, 0, "{} own parity failures:\n{}", $code, stats.fails.join("\n"));
        }
        #[test]
        #[ignore = "conversation overlay has a failure ceiling per locale; run with --run-ignored all (weekly language-parity.yml)"]
        fn $conversation() {
            let stats = run_parity_conversation($code);
            assert!(stats.ok + stats.fail > 0, "{} has no conversation cases", $code);
            assert_within_bound("parity_conversation", $code, &stats);
        }
    };
}

parity!(parity_de, parity_own_de, parity_conversation_de, "de");
parity!(parity_en, parity_own_en, parity_conversation_en, "en");
parity!(parity_fr, parity_own_fr, parity_conversation_fr, "fr");
parity!(parity_nl, parity_own_nl, parity_conversation_nl, "nl");
parity!(parity_es, parity_own_es, parity_conversation_es, "es");
parity!(parity_it, parity_own_it, parity_conversation_it, "it");
parity!(parity_pt, parity_own_pt, parity_conversation_pt, "pt");
parity!(parity_ca, parity_own_ca, parity_conversation_ca, "ca");
parity!(parity_ro, parity_own_ro, parity_conversation_ro, "ro");
parity!(parity_da, parity_own_da, parity_conversation_da, "da");
parity!(parity_nb, parity_own_nb, parity_conversation_nb, "nb");
parity!(parity_sv, parity_own_sv, parity_conversation_sv, "sv");
parity!(parity_fi, parity_own_fi, parity_conversation_fi, "fi");
parity!(parity_de_ch, parity_own_de_ch, parity_conversation_de_ch, "de-CH");
parity!(parity_de_at, parity_own_de_at, parity_conversation_de_at, "de-AT");
parity!(parity_en_gb, parity_own_en_gb, parity_conversation_en_gb, "en-GB");
parity!(parity_pt_br, parity_own_pt_br, parity_conversation_pt_br, "pt-BR");
parity!(parity_af, parity_own_af, parity_conversation_af, "af");
parity!(parity_cs, parity_own_cs, parity_conversation_cs, "cs");
parity!(parity_sk, parity_own_sk, parity_conversation_sk, "sk");
parity!(parity_pl, parity_own_pl, parity_conversation_pl, "pl");
parity!(parity_hu, parity_own_hu, parity_conversation_hu, "hu");
parity!(parity_hr, parity_own_hr, parity_conversation_hr, "hr");
parity!(parity_sl, parity_own_sl, parity_conversation_sl, "sl");
parity!(parity_bg, parity_own_bg, parity_conversation_bg, "bg");
parity!(parity_el, parity_own_el, parity_conversation_el, "el");
parity!(parity_sr, parity_own_sr, parity_conversation_sr, "sr");
parity!(parity_sr_latn, parity_own_sr_latn, parity_conversation_sr_latn, "sr-Latn");
parity!(parity_uk, parity_own_uk, parity_conversation_uk, "uk");
parity!(parity_zh_cn, parity_own_zh_cn, parity_conversation_zh_cn, "zh-CN");
parity!(parity_zh_tw, parity_own_zh_tw, parity_conversation_zh_tw, "zh-TW");
parity!(parity_zh_hk, parity_own_zh_hk, parity_conversation_zh_hk, "zh-HK");
parity!(parity_ar, parity_own_ar, parity_conversation_ar, "ar");
parity!(parity_he, parity_own_he, parity_conversation_he, "he");
parity!(parity_fa, parity_own_fa, parity_conversation_fa, "fa");
parity!(parity_ur, parity_own_ur, parity_conversation_ur, "ur");
parity!(parity_tr, parity_own_tr, parity_conversation_tr, "tr");
parity!(parity_th, parity_own_th, parity_conversation_th, "th");
parity!(parity_ko, parity_own_ko, parity_conversation_ko, "ko");
parity!(parity_ja, parity_own_ja, parity_conversation_ja, "ja");
parity!(parity_cy, parity_own_cy, parity_conversation_cy, "cy");
parity!(parity_et, parity_own_et, parity_conversation_et, "et");
parity!(parity_eu, parity_own_eu, parity_conversation_eu, "eu");
parity!(parity_ga, parity_own_ga, parity_conversation_ga, "ga");
parity!(parity_gl, parity_own_gl, parity_conversation_gl, "gl");
parity!(parity_is, parity_own_is, parity_conversation_is, "is");
parity!(parity_lb, parity_own_lb, parity_conversation_lb, "lb");
parity!(parity_kw, parity_own_kw, parity_conversation_kw, "kw");
parity!(parity_lt, parity_own_lt, parity_conversation_lt, "lt");
parity!(parity_lv, parity_own_lv, parity_conversation_lv, "lv");
parity!(parity_id, parity_own_id, parity_conversation_id, "id");
parity!(parity_ms, parity_own_ms, parity_conversation_ms, "ms");
parity!(parity_sw, parity_own_sw, parity_conversation_sw, "sw");
parity!(parity_vi, parity_own_vi, parity_conversation_vi, "vi");
parity!(parity_hi, parity_own_hi, parity_conversation_hi, "hi");
parity!(parity_bn, parity_own_bn, parity_conversation_bn, "bn");
parity!(parity_gu, parity_own_gu, parity_conversation_gu, "gu");
parity!(parity_kn, parity_own_kn, parity_conversation_kn, "kn");
parity!(parity_ml, parity_own_ml, parity_conversation_ml, "ml");
parity!(parity_mr, parity_own_mr, parity_conversation_mr, "mr");
parity!(parity_ta, parity_own_ta, parity_conversation_ta, "ta");
parity!(parity_te, parity_own_te, parity_conversation_te, "te");
parity!(parity_pa, parity_own_pa, parity_conversation_pa, "pa");
parity!(parity_ne, parity_own_ne, parity_conversation_ne, "ne");
parity!(parity_hy, parity_own_hy, parity_conversation_hy, "hy");
parity!(parity_ka, parity_own_ka, parity_conversation_ka, "ka");
parity!(parity_mn, parity_own_mn, parity_conversation_mn, "mn");
