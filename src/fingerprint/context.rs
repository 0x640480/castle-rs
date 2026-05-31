//! Per-mint dynamism for a [`Fingerprint`]: caller-supplied deployment/persona
//! context (site hostname, geo/locale bundle) and a jitter pass for the
//! timing/behavioral signals a real browser varies on every page load.
//!
//! Device-identity fields (user agent, GPU, screen, fonts, detection flags, …)
//! are never touched — they define *which* device the profile is and must stay
//! fixed and internally consistent.

use rand::Rng;

use super::Fingerprint;

/// A coherent geo/locale persona: the navigator/`Intl` values that should match
/// the claimed user's region. Supply one via [`crate::token::MintOptions`] to
/// mint for a different locale than the bundled profile's.
///
/// The fields must be mutually consistent (offset ↔ zone ↔ locale); prefer
/// [`Persona::new`], which derives [`Persona::locale_date_string`] for you.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Persona {
    /// IANA time zone, e.g. `"America/New_York"` (slot 4/1).
    pub time_zone: String,
    /// `getTimezoneOffset()` in minutes, e.g. `300` for UTC−5 (slot 0/8).
    pub timezone_offset: i64,
    /// Standard-vs-summer offset difference in minutes (slot 0/8).
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

impl Persona {
    /// Builds a persona, deriving [`Persona::locale_date_string`] from `locale`
    /// and taking `language`/`voice_language` from the head of `languages`.
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

    /// The en-US / America/New_York persona — matches the bundled capture, so
    /// applying it via [`Fingerprint::with_persona`] is a no-op.
    pub fn en_us_new_york() -> Self {
        Self::new(
            "en-US",
            "America/New_York",
            300,
            60,
            vec!["en-US".to_string(), "en".to_string()],
        )
        .expect("en-US locale date format is supported")
    }
}

impl Default for Persona {
    fn default() -> Self {
        Self::en_us_new_york()
    }
}

/// The `LocaleDateString` probe (slot 0/20) for a locale.
///
/// The page constructs `new Date(1970, 2, 1)` (local time) and formats it with
/// the default locale, so the result is the fixed wall-clock `1970-03-01
/// 00:00:00` rendered in that locale's date/time format — it is **timezone
/// independent**. Returns `None` for locales whose format this crate does not
/// know (extend the match as needed).
pub fn locale_date_string(locale: &str) -> Option<String> {
    let s = match locale {
        "en-US" => "3/1/1970, 12:00:00 AM",
        _ => return None,
    };
    Some(s.to_string())
}

impl Fingerprint {
    /// Returns a clone with the geo/locale fields replaced by `persona`.
    pub fn with_persona(&self, persona: &Persona) -> Fingerprint {
        let mut fp = self.clone();
        fp.time_zone = persona.time_zone.clone();
        fp.timezone_offset = persona.timezone_offset;
        fp.summertime_offset = persona.summertime_offset;
        fp.locale = persona.locale.clone();
        fp.language = persona.language.clone();
        fp.languages = persona.languages.clone();
        fp.voice_language = persona.voice_language.clone();
        fp.locale_date_string = persona.locale_date_string.clone();
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
    fn default_persona_matches_bundled_capture() {
        // Applying the default persona changes nothing → byte-identical output.
        let fp = chrome_148_macos();
        let with = fp.with_persona(&Persona::default());
        assert_eq!(with.encode_fp(INIT, 17), fp.encode_fp(INIT, 17));
    }

    #[test]
    fn locale_date_string_known_and_unknown() {
        assert_eq!(
            locale_date_string("en-US").as_deref(),
            Some("3/1/1970, 12:00:00 AM")
        );
        assert_eq!(locale_date_string("zz-ZZ"), None);
    }

    #[test]
    fn overrides_change_only_their_fields() {
        let base = chrome_148_macos();

        let h = base.with_hostname("login.example.com");
        assert_eq!(h.hostname, "login.example.com");

        let paris = Persona {
            time_zone: "Europe/Paris".to_string(),
            timezone_offset: -60,
            summertime_offset: 60,
            locale: "fr-FR".to_string(),
            language: "fr-FR".to_string(),
            languages: vec!["fr-FR".to_string(), "fr".to_string()],
            voice_language: "fr-FR".to_string(),
            locale_date_string: "01/03/1970 00:00:00".to_string(),
        };
        let fp = base.with_persona(&paris);
        assert_eq!(fp.time_zone, "Europe/Paris");
        assert_eq!(fp.locale, "fr-FR");
        assert_eq!(fp.languages, vec!["fr-FR", "fr"]);
        // Device identity is untouched.
        assert_eq!(fp.user_agent, base.user_agent);
        assert_eq!(fp.webgl_renderer, base.webgl_renderer);
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
