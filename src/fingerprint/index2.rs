//! Part-7 field encoders and their lookup tables (fp_lists slots 7/0–7/30).

use super::encode::*;

pub const UA_PLATFORM_LUT: &[&str] = &["Android", "iOS", "macOS", "Linux", "Windows", "Unknown"];

pub const BROWSER_BRAND_LUT: &[&str] = &[
    "Chromium",
    "Google Chrome",
    "Opera",
    "Brave",
    "Microsoft Edge",
];

pub const CANVAS_ERROR_LUT: &[&str] = &[
    "Illegal invocation",
    "Can only call CanvasRenderingContext2D.getImageData on instances of CanvasRenderingContext2D",
    "'getImageData' called on an object that does not implement interface CanvasRenderingContext2D.",
];

pub const SCREEN_ORIENTATION_LUT: &[&str] = &[
    "landscape-primary",
    "portrait-primary",
    "landscape-secondary",
    "portrait-secondary",
];

pub const VOICE_OS_LUT: &[&str] = &["macOS", "Windows", "ChromeOS", "Android", "Other"];

/// Slot 7/0: UA-data platform.
pub fn ua_platform(platform: &str, init: i64) -> String {
    encode_optional_index(0, index_of(UA_PLATFORM_LUT, platform), platform, init)
}

/// Slot 7/1: UA-data platformVersion.
pub fn ua_platform_version(version: &str, init: i64) -> String {
    case4(1, version, init)
}

/// Slot 7/2: UA-data brand.
pub fn browser_brand(brand: &str) -> String {
    encode_optional_index(2, index_of(BROWSER_BRAND_LUT, brand), brand, 0)
}

/// Slot 7/3: time-source consistency diff.
pub fn time_diff(diff: i64) -> String {
    case5(3, diff)
}

/// Slot 7/4: new Date(initTime).getUTCMinutes() probe.
pub fn utc_minutes(minutes: i64) -> String {
    case5(4, minutes)
}

/// Slot 7/5: window.location.hostname.
pub fn hostname(hostname: &str, init: i64) -> String {
    case4(5, hostname, init)
}

/// Slot 7/6: empty-object JSON probe (XXTEA frame).
pub fn object_json(s: &str, init: i64) -> String {
    case7(6, &encode_xxtea_frame(6, s, init))
}

/// Slot 7/7: 46-bit reversed-and-packed feature vector.
pub fn bits46(bits: &[bool]) -> String {
    let rev: Vec<bool> = bits.iter().rev().copied().collect();
    case7(7, &bits_to_hex(&rev))
}

/// Slot 7/8: count of Linux-specific available fonts.
pub fn linux_font_count(n: i64) -> String {
    case5(8, n)
}

/// Slot 7/9: count of macOS-specific available fonts.
pub fn mac_font_count(n: i64) -> String {
    case5(9, n)
}

/// Slot 7/10: count of Windows-specific available fonts.
pub fn windows_font_count(n: i64) -> String {
    case5(10, n)
}

/// Slot 7/11: canvas-fingerprint dataURL length.
pub fn canvas_length(n: i64) -> String {
    case5(11, n)
}

/// Slot 7/12: navigation-timing duration floats.
pub fn navigation_timing(durations: &[f64]) -> String {
    case7(12, &arr_12_dig_hex_seq(durations))
}

/// Slot 7/13: iframe-navigator probe.
pub fn iframe_navigator_accessible(accessible: bool) -> String {
    case_empty(13, if accessible { 2 } else { 1 })
}

/// Slot 7/14: canvas-tamper bool vector.
pub fn canvas_integrity_flags(bits: &[bool]) -> String {
    case7(14, &bits_to_hex(bits))
}

/// Slot 7/15: canvas-tamper error message.
pub fn canvas_error_message(msg: &str, init: i64) -> String {
    encode_optional_index(15, index_of(CANVAS_ERROR_LUT, msg), msg, init)
}

/// Slot 7/16: [jsHeapSizeLimit, totalJSHeapSize], each scaled by 1/1000.
pub fn memory_info(values: &[i64]) -> String {
    let floats: Vec<f64> = values.iter().map(|&v| v as f64 / 1000.0).collect();
    case7(16, &arr_12_dig_hex_seq(&floats))
}

/// Slot 7/17: 8-bool screen-tamper vector.
pub fn screen_integrity_flags(bits: &[bool]) -> String {
    case7(17, &bits_to_hex(bits))
}

/// Slot 7/18: (innerWidth, outerWidth, innerHeight, outerHeight).
pub fn window_dims(inner_w: i64, outer_w: i64, inner_h: i64, outer_h: i64) -> String {
    case7(
        18,
        &(pack15_16(inner_w, outer_w) + &pack15_16(inner_h, outer_h)),
    )
}

/// Slot 7/19: (screen.availLeft, screen.availTop).
pub fn avail_left_top(left: i64, top: i64) -> String {
    case7(19, &(n_hex(left as u64, 4) + &n_hex(top as u64, 4)))
}

/// Slot 7/20: screen.orientation.type.
pub fn screen_orientation(orientation: &str, init: i64) -> String {
    encode_optional_index(
        20,
        index_of(SCREEN_ORIENTATION_LUT, orientation),
        orientation,
        init,
    )
}

/// Slot 7/21: screen.orientation.angle.
pub fn screen_orientation_angle(angle: i64) -> String {
    case5(21, angle)
}

/// Slot 7/22: (scrollBarWidth, scrollBarHeight).
pub fn scroll_bar(width: i64, height: i64) -> String {
    case7(22, &(n_hex(width as u64, 4) + &n_hex(height as u64, 4)))
}

/// Slot 7/23: canvas read-performance ratio (`"0000" + round(ratio*1000)`).
pub fn canvas_perf_ratio(ratio: f64) -> String {
    let v = round_scaled(ratio, 1000.0);
    case7(23, &(String::from("0000") + &n_hex(v as u64, 8)))
}

/// Slot 7/24: default speech-synthesis voice's language tag.
pub fn voice_language(lang: &str, init: i64) -> String {
    case4(24, lang, init)
}

/// Slot 7/25: total speech-synthesis voice count.
pub fn voices_length(n: i64) -> String {
    case5(25, n)
}

/// Slot 7/26: local-voice count.
pub fn local_voices_length(n: i64) -> String {
    case5(26, n)
}

/// Slot 7/27: Google-voice count.
pub fn google_voices_length(n: i64) -> String {
    case5(27, n)
}

/// Slot 7/28: inferred host OS from the voice list.
pub fn voice_os(os: &str, init: i64) -> String {
    encode_optional_index(28, index_of(VOICE_OS_LUT, os), os, init)
}

/// Slot 7/29: render-latency probe value.
pub fn render_latency(latency: i64) -> String {
    case5(29, latency)
}

/// Slot 7/30: keyboard-layout fingerprint hash.
pub fn keyboard_hash(hash: &str, init: i64) -> String {
    case4(30, hash, init)
}
