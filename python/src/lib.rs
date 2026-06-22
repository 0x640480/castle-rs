//! Python bindings (PyO3) for `castle-token`.
//!
//! Thin wrappers over the Rust core: `mint_token`, opaque `Fingerprint` /
//! `LocaleProfile` handles, and the device-catalog loaders. All the wire-format
//! logic lives in the `castle-token` crate — this is only the FFI surface.

use std::path::PathBuf;

use castle_token::fingerprint::{self as fp_core, LocaleProfile as CoreLocaleProfile};
use castle_token::token::{mint_fresh_default, MintOptions};
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(
    _castle_token,
    CastleError,
    PyException,
    "Error minting a Castle token or loading a device catalog."
);

/// An opaque browser-identity handle. Obtain one from
/// `Fingerprint.chrome_148_macos()`, `load_devices`, or `random_bundled_device`.
#[pyclass(name = "Fingerprint")]
#[derive(Clone)]
struct PyFingerprint(fp_core::Fingerprint);

#[pymethods]
impl PyFingerprint {
    /// The bundled Chrome 148 / macOS profile.
    #[staticmethod]
    fn chrome_148_macos() -> PyFingerprint {
        PyFingerprint(fp_core::chrome_148_macos().clone())
    }

    #[getter]
    fn user_agent(&self) -> String {
        self.0.user_agent.clone()
    }
    #[getter]
    fn ua_platform(&self) -> String {
        self.0.ua_platform.clone()
    }
    #[getter]
    fn webgl_renderer(&self) -> String {
        self.0.webgl_renderer.clone()
    }
    #[getter]
    fn locale(&self) -> String {
        self.0.locale.clone()
    }
    #[getter]
    fn time_zone(&self) -> String {
        self.0.time_zone.clone()
    }

    fn __repr__(&self) -> String {
        format!("Fingerprint(ua_platform={:?})", self.0.ua_platform)
    }
}

/// A coherent geo/locale bundle. Use a preset (`LocaleProfile.de_de()` or
/// `LocaleProfile.preset("de-DE")`) or build one with `LocaleProfile.new(...)`.
#[pyclass(name = "LocaleProfile")]
#[derive(Clone)]
struct PyLocaleProfile(CoreLocaleProfile);

#[pymethods]
impl PyLocaleProfile {
    /// Look up a bundled preset by locale tag (e.g. `"de-DE"`). Raises
    /// `CastleError` if there is no preset for it.
    #[staticmethod]
    fn preset(tag: &str) -> PyResult<PyLocaleProfile> {
        CoreLocaleProfile::preset(tag)
            .map(PyLocaleProfile)
            .ok_or_else(|| CastleError::new_err(format!("no locale preset for {tag:?}")))
    }

    /// Build a profile, deriving `locale_date_string` from `locale`. Raises
    /// `CastleError` when the locale's date format is not built in.
    #[staticmethod]
    fn new(
        locale: &str,
        time_zone: &str,
        timezone_offset: i64,
        summertime_offset: i64,
        languages: Vec<String>,
    ) -> PyResult<PyLocaleProfile> {
        CoreLocaleProfile::new(
            locale,
            time_zone,
            timezone_offset,
            summertime_offset,
            languages,
        )
        .map(PyLocaleProfile)
        .ok_or_else(|| {
            CastleError::new_err(format!("no built-in date format for locale {locale:?}"))
        })
    }

    #[staticmethod]
    fn en_us() -> PyLocaleProfile {
        PyLocaleProfile(CoreLocaleProfile::en_us())
    }
    #[staticmethod]
    fn en_gb() -> PyLocaleProfile {
        PyLocaleProfile(CoreLocaleProfile::en_gb())
    }
    #[staticmethod]
    fn de_de() -> PyLocaleProfile {
        PyLocaleProfile(CoreLocaleProfile::de_de())
    }
    #[staticmethod]
    fn fr_fr() -> PyLocaleProfile {
        PyLocaleProfile(CoreLocaleProfile::fr_fr())
    }
    #[staticmethod]
    fn it_it() -> PyLocaleProfile {
        PyLocaleProfile(CoreLocaleProfile::it_it())
    }
    #[staticmethod]
    fn es_es() -> PyLocaleProfile {
        PyLocaleProfile(CoreLocaleProfile::es_es())
    }
    #[staticmethod]
    fn ja_jp() -> PyLocaleProfile {
        PyLocaleProfile(CoreLocaleProfile::ja_jp())
    }

    #[getter]
    fn time_zone(&self) -> String {
        self.0.time_zone.clone()
    }
    #[getter]
    fn timezone_offset(&self) -> i64 {
        self.0.timezone_offset
    }
    #[getter]
    fn summertime_offset(&self) -> i64 {
        self.0.summertime_offset
    }
    #[getter]
    fn locale(&self) -> String {
        self.0.locale.clone()
    }
    #[getter]
    fn language(&self) -> String {
        self.0.language.clone()
    }
    #[getter]
    fn languages(&self) -> Vec<String> {
        self.0.languages.clone()
    }
    #[getter]
    fn voice_language(&self) -> String {
        self.0.voice_language.clone()
    }
    #[getter]
    fn locale_date_string(&self) -> String {
        self.0.locale_date_string.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "LocaleProfile(locale={:?}, time_zone={:?})",
            self.0.locale, self.0.time_zone
        )
    }
}

/// Mint an `X-Castle-Request-Token`.
///
/// `cuid`/`pk`/`ig`/`hostname` are required. `fingerprint` defaults to the
/// bundled Chrome 148 macOS profile. `locale_profile` overrides the geo/locale
/// fields; `jitter=True` varies the per-session timing/behavioral signals.
/// Raises `CastleError` on invalid input.
#[pyfunction]
#[pyo3(signature = (cuid, pk, ig, hostname, *, fingerprint=None, locale_profile=None,
                    jitter=false, init_time_ms=None, now_ms=None))]
#[allow(clippy::too_many_arguments)]
fn mint_token(
    cuid: &str,
    pk: &str,
    ig: i64,
    hostname: &str,
    fingerprint: Option<PyRef<'_, PyFingerprint>>,
    locale_profile: Option<PyRef<'_, PyLocaleProfile>>,
    jitter: bool,
    init_time_ms: Option<i64>,
    now_ms: Option<i64>,
) -> PyResult<String> {
    let fp: &fp_core::Fingerprint = match &fingerprint {
        Some(f) => &f.0,
        None => fp_core::chrome_148_macos(), // &'static coerces to &'a
    };
    let opts = MintOptions {
        cuid,
        fingerprint: fp,
        init_time_ms,
        pk,
        ig,
        now_ms,
        hostname,
        locale_profile: locale_profile.as_ref().map(|p| &p.0),
        jitter,
    };
    mint_fresh_default(&opts).map_err(|e| CastleError::new_err(e.to_string()))
}

/// Load a custom device catalog (a JSON array of fingerprints) from `path`.
#[pyfunction]
fn load_devices(path: PathBuf) -> PyResult<Vec<PyFingerprint>> {
    fp_core::load_devices(&path)
        .map(|v| v.into_iter().map(PyFingerprint).collect())
        .map_err(|e| CastleError::new_err(e.to_string()))
}

/// A uniformly random fingerprint from the bundled catalog.
#[pyfunction]
fn random_bundled_device() -> PyFingerprint {
    PyFingerprint(fp_core::random_bundled_device(&mut rand::thread_rng()).clone())
}

#[pymodule]
fn _castle_token(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyFingerprint>()?;
    m.add_class::<PyLocaleProfile>()?;
    m.add_function(wrap_pyfunction!(mint_token, m)?)?;
    m.add_function(wrap_pyfunction!(load_devices, m)?)?;
    m.add_function(wrap_pyfunction!(random_bundled_device, m)?)?;
    m.add("CastleError", m.py().get_type::<CastleError>())?;
    Ok(())
}
