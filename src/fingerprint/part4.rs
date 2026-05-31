//! Part-4 field encoders and their lookup tables (fp_lists slots 4/0–4/30).

use super::encode::*;
use super::Fingerprint;

/// Assembles every part-4 slot (4/0–4/30) into encoded order.
pub(super) fn encode(fp: &Fingerprint, init: i64) -> Vec<String> {
    vec![
        always0(),
        time_zone(&fp.time_zone, init),
        languages(&fp.languages, init),
        vendor_number(fp.vendor_number),
        castle_runtime_flags(&fp.castle_runtime_flags),
        to_fixed_err_len(fp.to_fixed_err_len),
        bot_detection_flags(&fp.bot_detection_flags),
        ua_platform_missing(fp.ua_platform_missing),
        worker_integrity_flags(&fp.worker_integrity_flags),
        browser_feature_flags(&fp.browser_feature_flags),
        suspicious_environment(fp.suspicious_environment),
        static01(&fp.static01_payload),
        locale(&fp.locale, init),
        window_dims(
            fp.window_outer_width,
            fp.window_inner_width,
            fp.window_outer_height,
            fp.window_inner_height,
        ),
        ua_high_entropy_empty(fp.ua_high_entropy_empty),
        ua_high_entropy_flags(&fp.ua_high_entropy_flags),
        ua_architecture(&fp.ua_architecture, init),
        ua_model(&fp.ua_model, init),
        ua_full_version(&fp.ua_full_version, init),
    ]
}

pub const TIME_ZONE_LUT: &[&str] = &[
    "America/New_York",
    "America/Sao_Paulo",
    "America/Chicago",
    "America/Los_Angeles",
    "America/Mexico_City",
    "Asia/Shanghai",
];

pub const LANGUAGES_LIST_LUT: &[&str] = &[
    "US-US,US",
    "US-US",
    "BR-BR,BR,US-US,US",
    "ES-ES,ES",
    "CN-CN,CN",
];

pub const ARCHITECTURE_LUT: &[&str] = &["arm", "x86"];

/// Slot 4/0: always-zero probe.
pub fn always0() -> String {
    case3(0, 0)
}

/// Slot 4/1: resolved IANA timezone.
pub fn time_zone(tz: &str, init: i64) -> String {
    encode_optional_index(1, index_of(TIME_ZONE_LUT, tz), tz, init)
}

/// Slot 4/2: navigator.languages (joined with "," for the LUT).
pub fn languages(languages: &[String], init: i64) -> String {
    let joined = languages.join(",");
    encode_optional_index(2, index_of(LANGUAGES_LIST_LUT, &joined), &joined, init)
}

/// Slot 4/6: vendor probe number.
pub fn vendor_number(n: i64) -> String {
    case5(6, n)
}

/// Slot 4/10: 4-bool Castle runtime config probes.
pub fn castle_runtime_flags(bits: &[bool]) -> String {
    case7(10, &bits_to_hex(bits))
}

/// Slot 4/12: toFixed-error-message length.
pub fn to_fixed_err_len(n: i64) -> String {
    case5(12, n)
}

/// Slot 4/13: 11-bool bot-detection probe vector.
pub fn bot_detection_flags(bits: &[bool]) -> String {
    case7(13, &bits_to_hex(bits))
}

/// Slot 4/14: whether UA-data platform is empty.
pub fn ua_platform_missing(missing: bool) -> String {
    case_empty(14, if missing { 2 } else { 1 })
}

/// Slot 4/16: 9-bool worker-vs-main consistency probes.
pub fn worker_integrity_flags(bits: &[bool]) -> String {
    case7(16, &bits_to_hex(bits))
}

/// Slot 4/17: 14-bool browser-features vector.
pub fn browser_feature_flags(bits: &[bool]) -> String {
    case7(17, &bits_to_hex(bits))
}

/// Slot 4/18: suspicious-environment flag.
pub fn suspicious_environment(suspicious: bool) -> String {
    case_empty(18, if suspicious { 2 } else { 1 })
}

/// Slot 4/21: literal "01000000" probe.
pub fn static01(payload: &str) -> String {
    case7(21, payload)
}

/// Slot 4/22: Intl.DateTimeFormat resolvedOptions.locale.
pub fn locale(locale: &str, init: i64) -> String {
    case4(22, locale, init)
}

/// Slot 4/24: (outerWidth-innerWidth, outerHeight-innerHeight).
pub fn window_dims(outer_w: i64, inner_w: i64, outer_h: i64, inner_h: i64) -> String {
    let w_diff = outer_w - inner_w;
    let h_diff = outer_h - inner_h;
    case7(24, &(n_hex(w_diff as u64, 4) + &n_hex(h_diff as u64, 4)))
}

/// Slot 4/26: UA-data high-entropy empty flag.
pub fn ua_high_entropy_empty(empty: bool) -> String {
    case_empty(26, if empty { 2 } else { 1 })
}

/// Slot 4/27: 4-bool UA-data flags [is64bit, is32bit, wow64, mobile].
pub fn ua_high_entropy_flags(bits: &[bool]) -> String {
    case7(27, &bits_to_hex(bits))
}

/// Slot 4/28: UA-data architecture.
pub fn ua_architecture(arch: &str, init: i64) -> String {
    encode_optional_index(28, index_of(ARCHITECTURE_LUT, arch), arch, init)
}

/// Slot 4/29: UA-data model.
pub fn ua_model(model: &str, init: i64) -> String {
    case4(29, model, init)
}

/// Slot 4/30: UA-data uaFullVersion.
pub fn ua_full_version(version: &str, init: i64) -> String {
    case4(30, version, init)
}
