mod voice_suite_support;

use voice_suite_support::parity::run_parity;

macro_rules! parity {
    ($name:ident, $code:literal) => {
        #[test]
        fn $name() {
            let stats = run_parity($code);
            assert!(stats.ok + stats.fail > 0, "{} has no parity cases", $code);
            if std::env::var_os("KLAR_PARITY_REPORT").is_some() {
                if stats.fail > 0 {
                    println!(
                        "::warning title={code} parity::{fail} failures (path-scoped report; not a PR gate)",
                        code = $code,
                        fail = stats.fail
                    );
                }
                return;
            }
            assert_eq!(stats.fail, 0, "{} parity failures:\n{}", $code, stats.fails.join("\n"));
        }
    };
}

parity!(parity_de, "de");
parity!(parity_en, "en");
parity!(parity_fr, "fr");
parity!(parity_nl, "nl");
parity!(parity_es, "es");
parity!(parity_it, "it");
parity!(parity_pt, "pt");
parity!(parity_ca, "ca");
parity!(parity_ro, "ro");
parity!(parity_da, "da");
parity!(parity_nb, "nb");
parity!(parity_sv, "sv");
parity!(parity_fi, "fi");
parity!(parity_de_ch, "de-CH");
parity!(parity_de_at, "de-AT");
parity!(parity_en_gb, "en-GB");
parity!(parity_pt_br, "pt-BR");
parity!(parity_af, "af");
parity!(parity_cs, "cs");
parity!(parity_sk, "sk");
parity!(parity_pl, "pl");
parity!(parity_hu, "hu");
parity!(parity_hr, "hr");
parity!(parity_sl, "sl");
parity!(parity_bg, "bg");
parity!(parity_el, "el");
parity!(parity_sr, "sr");
parity!(parity_sr_latn, "sr-Latn");
parity!(parity_uk, "uk");
parity!(parity_zh_cn, "zh-CN");
parity!(parity_zh_tw, "zh-TW");
parity!(parity_zh_hk, "zh-HK");
parity!(parity_ar, "ar");
parity!(parity_he, "he");
parity!(parity_fa, "fa");
parity!(parity_ur, "ur");
parity!(parity_tr, "tr");
parity!(parity_th, "th");
parity!(parity_ko, "ko");
parity!(parity_ja, "ja");
parity!(parity_cy, "cy");
parity!(parity_et, "et");
parity!(parity_eu, "eu");
parity!(parity_ga, "ga");
parity!(parity_gl, "gl");
parity!(parity_is, "is");
parity!(parity_lb, "lb");
parity!(parity_kw, "kw");
parity!(parity_lt, "lt");
parity!(parity_lv, "lv");
parity!(parity_id, "id");
parity!(parity_ms, "ms");
parity!(parity_sw, "sw");
parity!(parity_vi, "vi");
parity!(parity_hi, "hi");
parity!(parity_bn, "bn");
parity!(parity_gu, "gu");
parity!(parity_kn, "kn");
parity!(parity_ml, "ml");
parity!(parity_mr, "mr");
parity!(parity_ta, "ta");
parity!(parity_te, "te");
parity!(parity_pa, "pa");
parity!(parity_ne, "ne");
parity!(parity_hy, "hy");
parity!(parity_ka, "ka");
parity!(parity_mn, "mn");
