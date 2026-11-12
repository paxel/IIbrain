//! The strings, generated at build time from `lib/l10n/*.arb` and the
//! desktop's own `desktop/l10n/*.arb` by `iibrain-l10n`. One accessor
//! per key; the ones with placeholders take typed arguments.

include!(concat!(env!("OUT_DIR"), "/l10n.rs"));

/// The app's locale for a system locale such as `de_DE.UTF-8` or
/// `en-US`: the language part when the app speaks it, else English.
pub fn locale_for_system(system: Option<&str>) -> &'static str {
    let lang = system
        .unwrap_or("en")
        .split(['-', '_', '.'])
        .next()
        .unwrap_or("en")
        .to_lowercase();
    LOCALES.iter().copied().find(|l| *l == lang).unwrap_or("en")
}
