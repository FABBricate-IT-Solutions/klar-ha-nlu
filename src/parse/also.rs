pub(crate) fn is_also_token(token: &str) -> bool {
    matches!(
        token,
        "auch" | "too" | "also" | "well" | "aussi" | "ook" | "tambien" | "anche" | "tambem" | "tambe"
            | "ogsaa" | "ocksa" | "myos" | "taky" | "tiez" | "tez" | "takodjer" | "tudi" | "takodje"
            | "също" | "επισης" | "такође" | "також" | "也" | "都" | "ايضا" | "גם" | "هم" | "بھی"
            | "dahi" | "deasemenea" | "duay" | "도" | "も" | "hefyd" | "ka" | "ere" | "freisin"
            | "tamén" | "lika" | "genausou" | "ynwedh" | "irgi" | "ari" | "juga" | "pia" | "nua"
            | "bhi" | "pan" | "kooda" | "koode" | "suddha" | "kuda" | "vi" | "pani" | "el" | "ch"
    )
}
