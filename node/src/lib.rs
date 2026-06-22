//! Node.js bindings (napi-rs) for `castle-token`.
//!
//! Thin wrappers over the Rust core: `mintToken`, opaque `Fingerprint` /
//! `LocaleProfile` handles, and the device-catalog loaders. All the wire-format
//! logic lives in the `castle-token` crate — this is only the FFI surface.

use std::path::PathBuf;

use castle_token::fingerprint::{self as fp_core, LocaleProfile as CoreLocaleProfile};
use castle_token::token::{mint_fresh_default, MintOptions};
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// An opaque browser-identity handle. Obtain one from
/// `Fingerprint.chrome148Macos()`, `loadDevices`, or `randomBundledDevice`.
#[napi]
pub struct Fingerprint(fp_core::Fingerprint);

#[napi]
impl Fingerprint {
    /// The bundled Chrome 148 / macOS profile.
    #[napi(factory)]
    pub fn chrome_148_macos() -> Self {
        Fingerprint(fp_core::chrome_148_macos().clone())
    }

    #[napi(getter)]
    pub fn user_agent(&self) -> String {
        self.0.user_agent.clone()
    }
    #[napi(getter)]
    pub fn ua_platform(&self) -> String {
        self.0.ua_platform.clone()
    }
    #[napi(getter)]
    pub fn webgl_renderer(&self) -> String {
        self.0.webgl_renderer.clone()
    }
    #[napi(getter)]
    pub fn locale(&self) -> String {
        self.0.locale.clone()
    }
    #[napi(getter)]
    pub fn time_zone(&self) -> String {
        self.0.time_zone.clone()
    }
}

/// A coherent geo/locale bundle. Use a preset (`LocaleProfile.deDe()` or
/// `LocaleProfile.preset("de-DE")`) or build one with `LocaleProfile.create(...)`.
#[napi]
pub struct LocaleProfile(CoreLocaleProfile);

#[napi]
impl LocaleProfile {
    /// Look up a bundled preset by locale tag (e.g. `"de-DE"`). Throws if there
    /// is no preset for it.
    #[napi(factory)]
    pub fn preset(tag: String) -> Result<Self> {
        CoreLocaleProfile::preset(&tag)
            .map(LocaleProfile)
            .ok_or_else(|| Error::from_reason(format!("no locale preset for {tag:?}")))
    }

    /// Build a profile, deriving `localeDateString` from `locale`. Throws when
    /// the locale's date format is not built in.
    #[napi(factory)]
    pub fn create(
        locale: String,
        time_zone: String,
        timezone_offset: i64,
        summertime_offset: i64,
        languages: Vec<String>,
    ) -> Result<Self> {
        CoreLocaleProfile::new(
            &locale,
            &time_zone,
            timezone_offset,
            summertime_offset,
            languages,
        )
        .map(LocaleProfile)
        .ok_or_else(|| Error::from_reason(format!("no built-in date format for locale {locale:?}")))
    }

    #[napi(factory)]
    pub fn en_us() -> Self {
        LocaleProfile(CoreLocaleProfile::en_us())
    }
    #[napi(factory)]
    pub fn en_gb() -> Self {
        LocaleProfile(CoreLocaleProfile::en_gb())
    }
    #[napi(factory)]
    pub fn de_de() -> Self {
        LocaleProfile(CoreLocaleProfile::de_de())
    }
    #[napi(factory)]
    pub fn fr_fr() -> Self {
        LocaleProfile(CoreLocaleProfile::fr_fr())
    }
    #[napi(factory)]
    pub fn it_it() -> Self {
        LocaleProfile(CoreLocaleProfile::it_it())
    }
    #[napi(factory)]
    pub fn es_es() -> Self {
        LocaleProfile(CoreLocaleProfile::es_es())
    }
    #[napi(factory)]
    pub fn ja_jp() -> Self {
        LocaleProfile(CoreLocaleProfile::ja_jp())
    }

    #[napi(getter)]
    pub fn time_zone(&self) -> String {
        self.0.time_zone.clone()
    }
    #[napi(getter)]
    pub fn timezone_offset(&self) -> i64 {
        self.0.timezone_offset
    }
    #[napi(getter)]
    pub fn summertime_offset(&self) -> i64 {
        self.0.summertime_offset
    }
    #[napi(getter)]
    pub fn locale(&self) -> String {
        self.0.locale.clone()
    }
    #[napi(getter)]
    pub fn language(&self) -> String {
        self.0.language.clone()
    }
    #[napi(getter)]
    pub fn languages(&self) -> Vec<String> {
        self.0.languages.clone()
    }
    #[napi(getter)]
    pub fn voice_language(&self) -> String {
        self.0.voice_language.clone()
    }
    #[napi(getter)]
    pub fn locale_date_string(&self) -> String {
        self.0.locale_date_string.clone()
    }
}

/// Mint an `X-Castle-Request-Token`.
///
/// `cuid`/`pk`/`ig`/`hostname` are required. `fingerprint` defaults to the
/// bundled Chrome 148 macOS profile. `localeProfile` overrides the geo/locale
/// fields; `jitter=true` varies the per-session timing/behavioral signals.
/// Throws on invalid input.
#[napi]
#[allow(clippy::too_many_arguments)]
pub fn mint_token(
    cuid: String,
    pk: String,
    ig: u32,
    hostname: String,
    fingerprint: Option<&Fingerprint>,
    locale_profile: Option<&LocaleProfile>,
    jitter: Option<bool>,
    init_time_ms: Option<f64>,
    now_ms: Option<f64>,
) -> Result<String> {
    let fp: &fp_core::Fingerprint = match fingerprint {
        Some(f) => &f.0,
        None => fp_core::chrome_148_macos(), // &'static coerces to &'a
    };
    let opts = MintOptions {
        cuid: &cuid,
        fingerprint: fp,
        init_time_ms: init_time_ms.map(|x| x as i64),
        pk: &pk,
        ig: ig as i64,
        now_ms: now_ms.map(|x| x as i64),
        hostname: &hostname,
        locale_profile: locale_profile.map(|p| &p.0),
        jitter: jitter.unwrap_or(false),
    };
    mint_fresh_default(&opts).map_err(|e| Error::from_reason(e.to_string()))
}

/// Load a custom device catalog (a JSON array of fingerprints) from `path`.
#[napi]
pub fn load_devices(path: String) -> Result<Vec<Fingerprint>> {
    fp_core::load_devices(PathBuf::from(path))
        .map(|v| v.into_iter().map(Fingerprint).collect())
        .map_err(|e| Error::from_reason(e.to_string()))
}

/// A uniformly random fingerprint from the bundled catalog.
#[napi]
pub fn random_bundled_device() -> Fingerprint {
    Fingerprint(fp_core::random_bundled_device(&mut rand::thread_rng()).clone())
}
