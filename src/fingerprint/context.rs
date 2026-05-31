//! Per-mint dynamism for a [`Fingerprint`]: caller-supplied deployment/locale
//! context (site hostname, geo/locale bundle) and a jitter pass for the
//! timing/behavioral signals a real browser varies on every page load.
//!
//! Device-identity fields (user agent, GPU, screen, fonts, detection flags, …)
//! are never touched — they define *which* device the profile is and must stay
//! fixed and internally consistent.

use rand::Rng;

use super::Fingerprint;

/// A coherent geo/locale bundle: the navigator/`Intl` values that should match
/// the claimed user's region. Supply one via [`crate::token::MintOptions`] to
/// mint for a different locale than the bundled profile's, or start from a
/// preset like [`LocaleProfile::de_de`] and tweak the public fields.
///
/// The fields must be mutually consistent (offset ↔ zone ↔ locale); prefer
/// [`LocaleProfile::new`], which derives [`locale_date_string`] for you.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocaleProfile {
    /// IANA time zone, e.g. `"America/New_York"` (slot 4/1).
    pub time_zone: String,
    /// `getTimezoneOffset()` in minutes (positive = behind UTC), e.g. `300` for
    /// UTC−5, `-60` for UTC+1 (slot 0/8).
    pub timezone_offset: i64,
    /// Standard-vs-summer offset difference in minutes (`60` for 1h DST, `0` for
    /// zones without DST) (slot 0/8).
    pub summertime_offset: i64,
    /// `Intl.DateTimeFormat().resolvedOptions().locale`, e.g. `"en-US"` (slot 4/22).
    pub locale: String,
    /// `navigator.language`, e.g. `"en-US"` (slot 0/2).
    pub language: String,
    /// `navigator.languages`, e.g. `["en-US", "en"]` (slot 4/2).
    pub languages: Vec<String>,
    /// Default speech-synthesis voice language tag (slot 7/24).
    pub voice_language: String,
    /// The fixed `new Date(1970, 2, 1).toLocaleString()` probe (slot 0/20).
    pub locale_date_string: String,
}

impl LocaleProfile {
    /// Builds a profile, deriving [`locale_date_string`] from `locale` and
    /// taking `language`/`voice_language` from the head of `languages`.
    ///
    /// Returns `None` when the locale's date format is not known to this crate;
    /// construct the struct directly with an explicit `locale_date_string` then.
    pub fn new(
        locale: &str,
        time_zone: &str,
        timezone_offset: i64,
        summertime_offset: i64,
        languages: Vec<String>,
    ) -> Option<Self> {
        let language = languages
            .first()
            .cloned()
            .unwrap_or_else(|| locale.to_string());
        Some(Self {
            time_zone: time_zone.to_string(),
            timezone_offset,
            summertime_offset,
            locale: locale.to_string(),
            language: language.clone(),
            voice_language: language,
            languages,
            locale_date_string: locale_date_string(locale)?,
        })
    }

    /// Looks up a bundled preset by its `locale` tag (e.g. `"de-DE"`), or `None`
    /// if there is no preset for it. See the `*_*` constructors for the full set.
    pub fn preset(locale: &str) -> Option<Self> {
        let p = match locale {
            "en-US" => Self::en_us(),
            "en-GB" => Self::en_gb(),
            "de-DE" => Self::de_de(),
            "fr-FR" => Self::fr_fr(),
            "it-IT" => Self::it_it(),
            "es-ES" => Self::es_es(),
            "ja-JP" => Self::ja_jp(),
            _ => return None,
        };
        Some(p)
    }

    /// en-US / America/New_York — matches the bundled capture, so applying it via
    /// [`Fingerprint::with_locale_profile`] is a no-op.
    pub fn en_us() -> Self {
        Self::built("en-US", "America/New_York", 300, 60, &["en-US", "en"])
    }
    /// en-GB / Europe/London.
    pub fn en_gb() -> Self {
        Self::built("en-GB", "Europe/London", 0, 60, &["en-GB", "en"])
    }
    /// de-DE / Europe/Berlin.
    pub fn de_de() -> Self {
        Self::built("de-DE", "Europe/Berlin", -60, 60, &["de-DE", "de"])
    }
    /// fr-FR / Europe/Paris.
    pub fn fr_fr() -> Self {
        Self::built("fr-FR", "Europe/Paris", -60, 60, &["fr-FR", "fr"])
    }
    /// it-IT / Europe/Rome.
    pub fn it_it() -> Self {
        Self::built("it-IT", "Europe/Rome", -60, 60, &["it-IT", "it"])
    }
    /// es-ES / Europe/Madrid.
    pub fn es_es() -> Self {
        Self::built("es-ES", "Europe/Madrid", -60, 60, &["es-ES", "es"])
    }
    /// ja-JP / Asia/Tokyo (no DST).
    pub fn ja_jp() -> Self {
        Self::built("ja-JP", "Asia/Tokyo", -540, 0, &["ja-JP", "ja"])
    }

    fn built(locale: &str, tz: &str, offset: i64, dst: i64, langs: &[&str]) -> Self {
        let languages = langs.iter().map(|s| s.to_string()).collect();
        Self::new(locale, tz, offset, dst, languages)
            .unwrap_or_else(|| panic!("preset locale {locale} has a known date format"))
    }
}

impl Default for LocaleProfile {
    fn default() -> Self {
        Self::en_us()
    }
}

/// The `LocaleDateString` probe (slot 0/20) for a locale.
///
/// The page constructs `new Date(1970, 2, 1)` (local time) and formats it with
/// the default locale, so the result is the fixed wall-clock `1970-03-01
/// 00:00:00` rendered in that locale's date/time pattern — it is **timezone
/// independent**. Values target modern Chrome/ICU; the exact string is
/// ICU-version-sensitive, so verify against your target build for exotic
/// locales (or set `locale_date_string` explicitly). Returns `None` for an
/// unknown locale.
pub fn locale_date_string(locale: &str) -> Option<String> {
    use Clock::{H12, H24};
    use Order::{Dmy, Mdy, Ymd};

    // (date-field order, date separator, zero-pad day/month, date↔time joiner, clock)
    let f = match locale {
        "en-US" => Fmt(Mdy, '/', false, ", ", H12),
        "en-GB" => Fmt(Dmy, '/', true, ", ", H24 { pad_hour: true }),
        "de-DE" => Fmt(Dmy, '.', false, ", ", H24 { pad_hour: true }),
        "fr-FR" => Fmt(Dmy, '/', true, " ", H24 { pad_hour: true }),
        "it-IT" => Fmt(Dmy, '/', true, ", ", H24 { pad_hour: true }),
        "es-ES" => Fmt(Dmy, '/', false, ", ", H24 { pad_hour: false }),
        "ja-JP" => Fmt(Ymd, '/', false, " ", H24 { pad_hour: false }),
        _ => return None,
    };
    Some(f.render_march_1_1970())
}

enum Order {
    Mdy,
    Dmy,
    Ymd,
}

enum Clock {
    H12,
    H24 { pad_hour: bool },
}

struct Fmt(Order, char, bool, &'static str, Clock);

impl Fmt {
    /// Renders the fixed `1970-03-01 00:00:00` wall-clock under this format.
    fn render_march_1_1970(&self) -> String {
        let Fmt(order, sep, pad, joiner, clock) = self;
        let day = if *pad { "01" } else { "1" };
        let month = if *pad { "03" } else { "3" };
        let year = "1970";
        let date = match order {
            Order::Mdy => format!("{month}{sep}{day}{sep}{year}"),
            Order::Dmy => format!("{day}{sep}{month}{sep}{year}"),
            Order::Ymd => format!("{year}{sep}{month}{sep}{day}"),
        };
        let time = match clock {
            Clock::H12 => "12:00:00 AM".to_string(),
            Clock::H24 { pad_hour: true } => "00:00:00".to_string(),
            Clock::H24 { pad_hour: false } => "0:00:00".to_string(),
        };
        format!("{date}{joiner}{time}")
    }
}

impl Fingerprint {
    /// Returns a clone with the geo/locale fields replaced by `profile`.
    pub fn with_locale_profile(&self, profile: &LocaleProfile) -> Fingerprint {
        let mut fp = self.clone();
        fp.time_zone = profile.time_zone.clone();
        fp.timezone_offset = profile.timezone_offset;
        fp.summertime_offset = profile.summertime_offset;
        fp.locale = profile.locale.clone();
        fp.language = profile.language.clone();
        fp.languages = profile.languages.clone();
        fp.voice_language = profile.voice_language.clone();
        fp.locale_date_string = profile.locale_date_string.clone();
        fp
    }

    /// Returns a clone with `window.location.hostname` (slot 7/5) set to `hostname`.
    pub fn with_hostname(&self, hostname: &str) -> Fingerprint {
        let mut fp = self.clone();
        fp.hostname = hostname.to_string();
        fp
    }

    /// Returns a clone with the per-session timing/behavioral signals jittered,
    /// preserving every consistency invariant (used heap < limit, zero-duration
    /// phases stay zero, ratios stay positive). Device-identity fields are
    /// untouched. Deterministic for a given `rng` state.
    pub fn jittered(&self, rng: &mut impl Rng) -> Fingerprint {
        let mut fp = self.clone();

        // Navigation-timing phases: scale each measured (non-zero) duration ±15%.
        for v in fp.navigation_timing.iter_mut() {
            if *v > 0.0 {
                *v = round1(*v * (1.0 + rng.gen_range(-0.15..0.15)));
            }
        }

        // performance.memory: jitter usedJSHeapSize ±8% (quantized to 10 KB),
        // kept strictly between 0 and the fixed jsHeapSizeLimit.
        if fp.memory_info.len() == 2 {
            let limit = fp.memory_info[0];
            let used = fp.memory_info[1] as f64 * (1.0 + rng.gen_range(-0.08..0.08));
            let used = (used / 10_000.0).round() as i64 * 10_000;
            fp.memory_info[1] = used.clamp(10_000, (limit - 10_000).max(10_000));
        }

        // Render latency (±20%) and canvas read-perf ratio (±15%): stay positive.
        fp.render_latency = jitter_int(fp.render_latency, 0.20, rng).max(1);
        fp.canvas_perf_ratio = (fp.canvas_perf_ratio * (1.0 + rng.gen_range(-0.15..0.15))).max(1.0);

        // Time-source consistency diff: a small absolute jitter.
        fp.time_diff = rng.gen_range(0..=3);

        // ce event timings: nudge each present timestamp's low byte ±4 ticks.
        if let Some(events) = fp.default_ce_events.as_mut() {
            for e in events.iter_mut() {
                if let Some(ts) = e.timestamp.as_mut() {
                    let delta: i16 = rng.gen_range(-4..=4);
                    *ts = (*ts as i16 + delta).rem_euclid(256) as u8;
                }
            }
        }

        fp
    }
}

fn jitter_int(x: i64, pct: f64, rng: &mut impl Rng) -> i64 {
    (x as f64 * (1.0 + rng.gen_range(-pct..pct))).round() as i64
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::chrome_148_macos;
    use rand::{rngs::StdRng, SeedableRng};

    const INIT: i64 = 1_778_379_452_408;

    #[test]
    fn default_profile_matches_bundled_capture() {
        // Applying the default (en-US) profile changes nothing → byte-identical.
        let fp = chrome_148_macos();
        let with = fp.with_locale_profile(&LocaleProfile::default());
        assert_eq!(with.encode_fp(INIT, 17), fp.encode_fp(INIT, 17));
        // The default also matches the en_us preset exactly.
        assert_eq!(LocaleProfile::default(), LocaleProfile::en_us());
    }

    #[test]
    fn locale_date_string_renders_each_locale() {
        let cases = [
            ("en-US", "3/1/1970, 12:00:00 AM"),
            ("en-GB", "01/03/1970, 00:00:00"),
            ("de-DE", "1.3.1970, 00:00:00"),
            ("fr-FR", "01/03/1970 00:00:00"),
            ("it-IT", "01/03/1970, 00:00:00"),
            ("es-ES", "1/3/1970, 0:00:00"),
            ("ja-JP", "1970/3/1 0:00:00"),
        ];
        for (locale, expected) in cases {
            assert_eq!(locale_date_string(locale).as_deref(), Some(expected), "{locale}");
        }
        assert_eq!(locale_date_string("zz-ZZ"), None);
    }

    #[test]
    fn presets_are_consistent_and_distinct() {
        let base = chrome_148_macos();
        let tags = ["en-US", "en-GB", "de-DE", "fr-FR", "it-IT", "es-ES", "ja-JP"];
        let mut seen = std::collections::HashSet::new();
        for tag in tags {
            let p = LocaleProfile::preset(tag).unwrap();
            // The preset is self-consistent: locale tag and derived date agree.
            assert_eq!(p.locale, tag);
            assert_eq!(p.locale_date_string, locale_date_string(tag).unwrap());
            assert_eq!(p.language, p.voice_language);
            // Each preset produces a distinct fp_lists (except en-US == bundled).
            let fp_lists = base.with_locale_profile(&p).encode_fp(INIT, 17);
            if tag == "en-US" {
                assert_eq!(fp_lists, base.encode_fp(INIT, 17));
            } else {
                assert_ne!(fp_lists, base.encode_fp(INIT, 17), "{tag} should differ");
            }
            assert!(seen.insert(fp_lists), "{tag} duplicated another preset");
        }
        assert!(LocaleProfile::preset("zz-ZZ").is_none());
    }

    #[test]
    fn overrides_change_only_their_fields() {
        let base = chrome_148_macos();

        let h = base.with_hostname("login.example.com");
        assert_eq!(h.hostname, "login.example.com");

        let de = base.with_locale_profile(&LocaleProfile::de_de());
        assert_eq!(de.time_zone, "Europe/Berlin");
        assert_eq!(de.locale, "de-DE");
        assert_eq!(de.languages, vec!["de-DE", "de"]);
        assert_eq!(de.locale_date_string, "1.3.1970, 00:00:00");
        // Device identity is untouched.
        assert_eq!(de.user_agent, base.user_agent);
        assert_eq!(de.webgl_renderer, base.webgl_renderer);
    }

    #[test]
    fn jitter_is_deterministic_per_seed_and_varies_across_seeds() {
        let fp = chrome_148_macos();
        let a = fp.jittered(&mut StdRng::seed_from_u64(7));
        let b = fp.jittered(&mut StdRng::seed_from_u64(7));
        let c = fp.jittered(&mut StdRng::seed_from_u64(8));

        // Same seed → identical fp_lists and ce.
        assert_eq!(a.encode_fp(INIT, 17), b.encode_fp(INIT, 17));
        assert_eq!(a.encode_ce().unwrap(), b.encode_ce().unwrap());
        // Different seed → different timing (fp_lists) and behavioral (ce) bytes.
        assert_ne!(a.encode_fp(INIT, 17), c.encode_fp(INIT, 17));
        assert_ne!(a.encode_ce().unwrap(), c.encode_ce().unwrap());
    }

    #[test]
    fn jitter_preserves_invariants() {
        let base = chrome_148_macos();
        let mut rng = StdRng::seed_from_u64(123);
        for _ in 0..300 {
            let j = base.jittered(&mut rng);

            // Device identity untouched.
            assert_eq!(j.user_agent, base.user_agent);
            assert_eq!(j.webgl_renderer, base.webgl_renderer);
            assert_eq!(j.screen_width, base.screen_width);
            assert_eq!(j.automation_bits, base.automation_bits);

            // Used heap positive and strictly below the (fixed) limit.
            assert_eq!(j.memory_info[0], base.memory_info[0]);
            assert!(j.memory_info[1] > 0 && j.memory_info[1] < j.memory_info[0]);

            // Zero phases stay zero; measured phases stay positive.
            for (orig, jit) in base.navigation_timing.iter().zip(&j.navigation_timing) {
                if *orig == 0.0 {
                    assert_eq!(*jit, 0.0);
                } else {
                    assert!(*jit > 0.0);
                }
            }
            assert!(j.render_latency >= 1);
            assert!(j.canvas_perf_ratio >= 1.0);
            assert!((0..=3).contains(&j.time_diff));

            // ce structure preserved (count + classes); only timings move.
            let oe = base.default_ce_events.as_ref().unwrap();
            let je = j.default_ce_events.as_ref().unwrap();
            assert_eq!(oe.len(), je.len());
            for (o, n) in oe.iter().zip(je) {
                assert_eq!(o.class, n.class);
                assert_eq!(o.node_idx, n.node_idx);
            }
        }
    }
}
