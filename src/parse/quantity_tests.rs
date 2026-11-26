use super::*;
use crate::lang::bind;
use crate::parse::normalize::tokenize;

fn toks(lang: &str, text: &str) -> Vec<String> {
    let _b = bind(&[lang.into()]);
    tokenize(text)
}

#[test]
fn signs_and_decimals_survive_tokenize() {
    assert_eq!(toks("en", "-5 degrees"), vec!["-5", "degrees"]);
    assert_eq!(toks("en", "19.5 degrees"), vec!["19.5", "degrees"]);
    assert_eq!(toks("de", "19,5 Grad"), vec!["19.5", "grad"]);
    assert_eq!(toks("en", "1:30"), vec!["1:30"]);
}

#[test]
fn minus_word_and_decimal_quantity() {
    let t = toks("en", "minus 5 degrees");
    assert_eq!(first_quantity(&t).map(|q| q.value), Some(-5.0));
    let t = toks("en", "19.5 degrees");
    assert_eq!(first_quantity(&t).map(|q| q.value), Some(19.5));
}

#[test]
fn compound_and_fraction_durations() {
    let _en = bind(&["en".into()]);
    let half = extract_duration(&tokenize("set a timer for half an hour")).expect("half");
    assert_eq!(half.minutes, Some(30));
    assert!(half.hours.is_none());
    let long = extract_duration(&tokenize("set a timer for two hours and thirty minutes")).expect("compound");
    assert_eq!(long.hours, Some(2));
    assert_eq!(long.minutes, Some(30));
    let clock = extract_duration(&tokenize("timer 1:30")).expect("clock");
    assert_eq!(clock.hours, Some(1));
    assert_eq!(clock.minutes, Some(30));
    assert!(extract_duration(&tokenize("set a timer for 0 minutes")).is_none());
    drop(_en);
    let _de = bind(&["de".into()]);
    let de = extract_duration(&tokenize("Timer auf 2 Stunden 30 Minuten")).expect("de compound");
    assert_eq!(de.hours, Some(2));
    assert_eq!(de.minutes, Some(30));
    let halb = extract_duration(&tokenize("Timer auf eineinhalb Stunden")).expect("eineinhalb");
    assert_eq!(halb.total_seconds(), 5400);
}

#[test]
fn valued_split_skips_duration_and() {
    let _en = bind(&["en".into()]);
    let tokens = tokenize("set timer two hours and thirty minutes");
    let at = tokens.iter().position(|t| t == "and").expect("and");
    assert!(!both_sides_valued(&tokens, at), "{tokens:?}");
    let tokens = tokenize("set timer an hour and a half");
    let at = tokens.iter().position(|t| t == "and").expect("and");
    assert!(!both_sides_valued(&tokens, at), "{tokens:?}");
    let tokens = tokenize("set hall light to 30 percent and office light to 80 percent");
    let at = tokens.iter().position(|t| t == "and").expect("and");
    assert!(both_sides_valued(&tokens, at), "{tokens:?}");
}

#[test]
fn relative_temp_steps() {
    let t = toks("en", "one degree warmer");
    assert_eq!(temperature_step(&t), Some(1.0));
    let t = toks("de", "zwei Grad wärmer");
    assert_eq!(temperature_step(&t), Some(2.0));
    let t = toks("de", "im Büro ist es zu kalt");
    assert_eq!(temperature_step(&t), Some(1.0));
    assert!(absolute_temperature(&t).is_none());
}

#[test]
fn bedroom_cuarto_quarto_keep_digit_setpoint() {
    // gl/es "cuarto" and pt "quarto" name the bedroom; ASCII temps must stay Celsius.
    let gl = toks("gl", "pon calefaccion cuarto 23");
    assert_eq!(absolute_temperature(&gl).map(format_quantity).as_deref(), Some("23"), "{gl:?}");
    let gl_ac = toks("gl", "pon ac cuarto 21");
    assert_eq!(absolute_temperature(&gl_ac).map(format_quantity).as_deref(), Some("21"), "{gl_ac:?}");
    let pt = toks("pt", "poe aquecimento quarto 23");
    assert_eq!(absolute_temperature(&pt).map(format_quantity).as_deref(), Some("23"), "{pt:?}");
    let pt_ac = toks("pt", "poe ac quarto 21");
    assert_eq!(absolute_temperature(&pt_ac).map(format_quantity).as_deref(), Some("21"), "{pt_ac:?}");
}
