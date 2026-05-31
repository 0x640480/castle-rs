//! The typed browser-identity bundle and its fp_lists / ce encoders.

mod devices;
mod encode;
mod part0;
mod part4;
mod part7;

pub use devices::{bundled_devices, chrome_148_macos, load_devices, random_bundled_device};

use serde::{Deserialize, Serialize};

use crate::ce;
use crate::error::Result;

/// Every browser trait the fp_lists encoders need, plus a default `ce`
/// payload. Field doc comments give each field's slot index (part/slot).
///
/// JSON uses PascalCase keys; acronym fields carry explicit `rename`
/// attributes so existing device catalogs deserialize unchanged.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Fingerprint {
    // --- Part 0 ---
    pub user_agent: String,                // (0/12)
    pub platform: String,                  // (0/0)
    pub vendor: String,                    // (0/1)
    pub language: String,                  // (0/2)
    pub device_memory: f64,                // (0/3)
    pub screen_width: i64,                 // (0/4)
    pub screen_avail_width: i64,           // (0/4)
    pub screen_height: i64,                // (0/4)
    pub screen_avail_height: i64,          // (0/4)
    pub color_depth: i64,                  // (0/5)
    pub hardware_concurrency: i64,         // (0/6)
    pub device_pixel_ratio: f64,           // (0/7)
    pub timezone_offset: i64,              // (0/8)
    pub summertime_offset: i64,            // (0/8)
    pub mime_types: Vec<String>,           // (0/9)
    pub plugins: Vec<String>,              // (0/10)
    pub navigator_bits: Vec<bool>,         // (0/11)
    pub canvas_hash: String,               // (0/13)
    pub enumerate_devices_bits: Vec<bool>, // (0/14)
    pub product_sub: String,               // (0/17)
    pub canvas_hash2: String,              // (0/18)
    #[serde(rename = "WebGLRenderer")]
    pub webgl_renderer: String, // (0/19)
    pub locale_date_string: String,        // (0/20)
    pub automation_bits: Vec<bool>,        // (0/21)
    pub eval_length: i64,                  // (0/22)
    pub max_call_stack_size: i64,          // (0/24)
    pub call_stack_error_msg: String,      // (0/25)
    pub call_stack_error_name: String,     // (0/26)
    pub call_stack_error_stack_len: i64,   // (0/27)
    pub touch_signature_hex: String,       // (0/28)
    pub property_error_msg: String,        // (0/29)
    pub navigator_properties: Vec<String>, // (0/30)
    pub can_play_type: Vec<i64>,           // (0/31)

    // --- Part 4 ---
    pub time_zone: String,               // (4/1)
    pub languages: Vec<String>,          // (4/2)
    pub vendor_number: i64,              // (4/6)
    pub castle_runtime_flags: Vec<bool>, // (4/10)
    pub to_fixed_err_len: i64,           // (4/12)
    pub bot_detection_flags: Vec<bool>,  // (4/13)
    #[serde(rename = "UAPlatformMissing")]
    pub ua_platform_missing: bool, // (4/14)
    pub worker_integrity_flags: Vec<bool>, // (4/16)
    pub browser_feature_flags: Vec<bool>, // (4/17)
    pub suspicious_environment: bool,    // (4/18)
    pub static01_payload: String,        // (4/21)
    pub locale: String,                  // (4/22)
    pub window_outer_width: i64,         // (4/24)
    pub window_inner_width: i64,         // (4/24)
    pub window_outer_height: i64,        // (4/24)
    pub window_inner_height: i64,        // (4/24)
    #[serde(rename = "UAHighEntropyEmpty")]
    pub ua_high_entropy_empty: bool, // (4/26)
    #[serde(rename = "UAHighEntropyFlags")]
    pub ua_high_entropy_flags: Vec<bool>, // (4/27)
    #[serde(rename = "UAArchitecture")]
    pub ua_architecture: String, // (4/28)
    #[serde(rename = "UAModel")]
    pub ua_model: String, // (4/29)
    #[serde(rename = "UAFullVersion")]
    pub ua_full_version: String, // (4/30)

    // --- Part 7 ---
    #[serde(rename = "UAPlatform")]
    pub ua_platform: String, // (7/0)
    #[serde(rename = "UAPlatformVersion")]
    pub ua_platform_version: String, // (7/1)
    pub browser_brand: String, // (7/2)
    pub time_diff: i64,        // (7/3)
    pub hostname: String,      // (7/5)
    #[serde(rename = "ObjectJSON")]
    pub object_json: String, // (7/6)
    pub bits46: Vec<bool>,     // (7/7)
    pub linux_font_count: i64, // (7/8)
    pub mac_font_count: i64,   // (7/9)
    pub windows_font_count: i64, // (7/10)
    pub canvas_fingerprinting_len: i64, // (7/11)
    pub navigation_timing: Vec<f64>, // (7/12)
    pub iframe_navigator_accessible: bool, // (7/13)
    pub canvas_integrity_flags: Vec<bool>, // (7/14)
    pub canvas_error_message: String, // (7/15)
    pub memory_info: Vec<i64>, // (7/16)
    pub screen_integrity_flags: Vec<bool>, // (7/17)
    pub window_inner_width7: i64, // (7/18)
    pub window_outer_width7: i64, // (7/18)
    pub window_inner_height7: i64, // (7/18)
    pub window_outer_height7: i64, // (7/18)
    pub avail_left: i64,       // (7/19)
    pub avail_top: i64,        // (7/19)
    pub screen_orientation: String, // (7/20)
    pub screen_orientation_angle: i64, // (7/21)
    pub scroll_bar_width: i64, // (7/22)
    pub scroll_bar_height: i64, // (7/22)
    pub canvas_perf_ratio: f64, // (7/23)
    pub voice_language: String, // (7/24)
    pub voices_length: i64,    // (7/25)
    pub local_voices_length: i64, // (7/26)
    pub google_voices_length: i64, // (7/27)
    #[serde(rename = "VoiceOS")]
    pub voice_os: String, // (7/28)
    pub render_latency: i64,   // (7/29)
    pub keyboard_hash: String, // (7/30)

    /// Typed `ce` events; when `None`, [`Fingerprint::default_ce_hex`] is used
    /// verbatim instead.
    #[serde(default, rename = "DefaultCEEvents")]
    pub default_ce_events: Option<Vec<ce::Event>>,

    /// Pre-encoded `ce` blob (lowercase hex) used when `default_ce_events` is `None`.
    #[serde(default, rename = "DefaultCEHex")]
    pub default_ce_hex: String,
}

impl Fingerprint {
    /// Renders the full fp_lists hex string at the given init time.
    ///
    /// `init_time_ms` is baked into every encrypted slot's XXTEA ciphertext via
    /// per-slot key derivation; it must equal the timestamp encoded in the
    /// matching `time_token_v1`. `utc_minutes` is the
    /// `new Date(init_time).getUTCMinutes()` probe (slot 7/4), computed by the
    /// caller so this method stays dependency-free.
    pub fn encode_fp(&self, init_time_ms: i64, utc_minutes: i64) -> String {
        let init = init_time_ms;
        let part0 = vec![
            part0::platform(&self.platform, init),
            part0::vendor(&self.vendor, init),
            part0::language(&self.language, init),
            part0::device_memory(self.device_memory),
            part0::screen(
                self.screen_width,
                self.screen_avail_width,
                self.screen_height,
                self.screen_avail_height,
            ),
            part0::color_depth(self.color_depth),
            part0::hardware_concurrency(self.hardware_concurrency),
            part0::device_pixel_ratio(self.device_pixel_ratio),
            part0::timezone(self.timezone_offset, self.summertime_offset),
            part0::mime_types(&self.mime_types),
            part0::plugins(&self.plugins),
            part0::navigator_bits(&self.navigator_bits),
            part0::user_agent(&self.user_agent, init),
            part0::canvas_hash(&self.canvas_hash, init),
            part0::enumerate_devices(&self.enumerate_devices_bits),
            part0::product_sub(&self.product_sub, init),
            part0::canvas_hash2(&self.canvas_hash2, init),
            part0::webgl_renderer(&self.webgl_renderer, init),
            part0::locale_date_string(&self.locale_date_string, init),
            part0::automation_bits(&self.automation_bits),
            part0::eval_length(self.eval_length),
            part0::max_call_stack_size(self.max_call_stack_size),
            part0::call_stack_error_msg(&self.call_stack_error_msg, init),
            part0::call_stack_error_name(&self.call_stack_error_name, init),
            part0::call_stack_error_stack_len(self.call_stack_error_stack_len),
            part0::touch_signature(&self.touch_signature_hex),
            part0::property_error_msg(&self.property_error_msg, init),
            part0::navigator_properties(&self.navigator_properties),
            part0::can_play_type(&self.can_play_type),
        ];
        let part4 = vec![
            part4::always0(),
            part4::time_zone(&self.time_zone, init),
            part4::languages(&self.languages, init),
            part4::vendor_number(self.vendor_number),
            part4::castle_runtime_flags(&self.castle_runtime_flags),
            part4::to_fixed_err_len(self.to_fixed_err_len),
            part4::bot_detection_flags(&self.bot_detection_flags),
            part4::ua_platform_missing(self.ua_platform_missing),
            part4::worker_integrity_flags(&self.worker_integrity_flags),
            part4::browser_feature_flags(&self.browser_feature_flags),
            part4::suspicious_environment(self.suspicious_environment),
            part4::static01(&self.static01_payload),
            part4::locale(&self.locale, init),
            part4::window_dims(
                self.window_outer_width,
                self.window_inner_width,
                self.window_outer_height,
                self.window_inner_height,
            ),
            part4::ua_high_entropy_empty(self.ua_high_entropy_empty),
            part4::ua_high_entropy_flags(&self.ua_high_entropy_flags),
            part4::ua_architecture(&self.ua_architecture, init),
            part4::ua_model(&self.ua_model, init),
            part4::ua_full_version(&self.ua_full_version, init),
        ];
        let part7 = vec![
            part7::ua_platform(&self.ua_platform, init),
            part7::ua_platform_version(&self.ua_platform_version, init),
            part7::browser_brand(&self.browser_brand),
            part7::time_diff(self.time_diff),
            part7::utc_minutes(utc_minutes),
            part7::hostname(&self.hostname, init),
            part7::object_json(&self.object_json, init),
            part7::bits46(&self.bits46),
            part7::linux_font_count(self.linux_font_count),
            part7::mac_font_count(self.mac_font_count),
            part7::windows_font_count(self.windows_font_count),
            part7::canvas_length(self.canvas_fingerprinting_len),
            part7::navigation_timing(&self.navigation_timing),
            part7::iframe_navigator_accessible(self.iframe_navigator_accessible),
            part7::canvas_integrity_flags(&self.canvas_integrity_flags),
            part7::canvas_error_message(&self.canvas_error_message, init),
            part7::memory_info(&self.memory_info),
            part7::screen_integrity_flags(&self.screen_integrity_flags),
            part7::window_dims(
                self.window_inner_width7,
                self.window_outer_width7,
                self.window_inner_height7,
                self.window_outer_height7,
            ),
            part7::avail_left_top(self.avail_left, self.avail_top),
            part7::screen_orientation(&self.screen_orientation, init),
            part7::screen_orientation_angle(self.screen_orientation_angle),
            part7::scroll_bar(self.scroll_bar_width, self.scroll_bar_height),
            part7::canvas_perf_ratio(self.canvas_perf_ratio),
            part7::voice_language(&self.voice_language, init),
            part7::voices_length(self.voices_length),
            part7::local_voices_length(self.local_voices_length),
            part7::google_voices_length(self.google_voices_length),
            part7::voice_os(&self.voice_os, init),
            part7::render_latency(self.render_latency),
            part7::keyboard_hash(&self.keyboard_hash, init),
        ];
        encode::encode_lists(&[part0, part4, part7, Vec::new(), Vec::new()])
    }

    /// Renders the `ce` hex blob: typed events via [`ce::encode`] when present,
    /// otherwise the stored hex verbatim.
    pub fn encode_ce(&self) -> Result<String> {
        match &self.default_ce_events {
            Some(events) => ce::encode(events),
            None => Ok(self.default_ce_hex.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAPTURED_INIT_TIME_MS: i64 = 1_778_379_452_408;

    #[test]
    fn chrome_148_macos_encode_fp_golden() {
        // utc_minutes = (1778379452408 / 60000) % 60 == 17
        let got = chrome_148_macos().encode_fp(CAPTURED_INIT_TIME_MS, 17);
        assert!(
            got.starts_with("001d03000b001408"),
            "prefix mismatch: {}",
            &got[..40.min(got.len())]
        );
        assert!(got.len() >= 1000, "fp too short: {}", got.len());
    }

    #[test]
    fn chrome_148_macos_encode_ce_golden() {
        let got = chrome_148_macos().encode_ce().unwrap();
        assert_eq!(
            got,
            "5607863f198ec60038088ec600380211d3c6000000380c5602863f191212120cd3c51100003f0c8ec6011f021112125601d3c60100001f0c"
        );
    }

    #[test]
    fn bundled_acronym_fields_present() {
        let fp = chrome_148_macos();
        // Catches a missed serde rename — these would be empty if mis-keyed.
        assert_eq!(fp.ua_platform, "macOS");
        assert_eq!(fp.object_json, "{}");
        assert!(fp.webgl_renderer.starts_with("ANGLE"));
        assert!(!fp.default_ce_hex.is_empty());
    }
}
