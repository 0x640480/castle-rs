//! Part-0 field encoders and their lookup tables (fp_lists slots 0/0–0/31).

use super::encode::*;
use super::Fingerprint;

/// Assembles every part-0 slot (0/0–0/31) into encoded order.
pub(super) fn encode(fp: &Fingerprint, init: i64) -> Vec<String> {
    vec![
        platform(&fp.platform, init),
        vendor(&fp.vendor, init),
        language(&fp.language, init),
        device_memory(fp.device_memory),
        screen(
            fp.screen_width,
            fp.screen_avail_width,
            fp.screen_height,
            fp.screen_avail_height,
        ),
        color_depth(fp.color_depth),
        hardware_concurrency(fp.hardware_concurrency),
        device_pixel_ratio(fp.device_pixel_ratio),
        timezone(fp.timezone_offset, fp.summertime_offset),
        mime_types(&fp.mime_types),
        plugins(&fp.plugins),
        navigator_bits(&fp.navigator_bits),
        user_agent(&fp.user_agent, init),
        canvas_hash(&fp.canvas_hash, init),
        enumerate_devices(&fp.enumerate_devices_bits),
        product_sub(&fp.product_sub, init),
        canvas_hash2(&fp.canvas_hash2, init),
        webgl_renderer(&fp.webgl_renderer, init),
        locale_date_string(&fp.locale_date_string, init),
        automation_bits(&fp.automation_bits),
        eval_length(fp.eval_length),
        max_call_stack_size(fp.max_call_stack_size),
        call_stack_error_msg(&fp.call_stack_error_msg, init),
        call_stack_error_name(&fp.call_stack_error_name, init),
        call_stack_error_stack_len(fp.call_stack_error_stack_len),
        touch_signature(&fp.touch_signature_hex),
        property_error_msg(&fp.property_error_msg, init),
        navigator_properties(&fp.navigator_properties),
        can_play_type(&fp.can_play_type),
    ]
}

pub const PLATFORM_LUT: &[&str] = &[
    "MacIntel",
    "Win32",
    "iPhone",
    "Linux armv8l",
    "iPad",
    "Linux armv81",
    "Linux aarch64",
    "Linux x86_64",
    "Linux armv7l",
];

pub const VENDOR_LUT: &[&str] = &["Google Inc.", "Apple Computer, Inc."];

pub const LANGUAGE_LUT: &[&str] = &[
    "US-US", "ES-ES", "FR-FR", "BR-BR", "GB-GB", "DE-DE", "RU-RU", "us-us", "gb-gb", "CN-CN",
    "ID-ID", "US-US", "IT-IT", "MX-MX", "PL-PL",
];

pub const PRODUCT_SUB_LUT: &[&str] = &["20030107", "20100101"];

pub const CALL_STACK_MESSAGE_LUT: &[&str] = &[
    "Maximum call stack size exceeded",
    "Maximum call stack size exceeded.",
    "too much recursion",
];

pub const CALL_STACK_NAME_LUT: &[&str] = &["InternalError", "RangeError", "Error"];

pub const PROPERTY_ERROR_LUT: &[&str] = &[
    "Cannot read property 'b' of undefined",
    "(void 0) is undefined",
    "undefined is not an object (evaluating '(void 0).b')",
    "Cannot read properties of undefined (reading 'b')",
];

/// Slot 0/0: navigator.platform.
pub fn platform(platform: &str, init: i64) -> String {
    encode_optional_index(0, index_of(PLATFORM_LUT, platform), platform, init)
}

/// Slot 0/1: navigator.vendor.
pub fn vendor(vendor: &str, init: i64) -> String {
    encode_optional_index(1, index_of(VENDOR_LUT, vendor), vendor, init)
}

/// Slot 0/2: navigator.language.
pub fn language(language: &str, init: i64) -> String {
    encode_optional_index(2, index_of(LANGUAGE_LUT, language), language, init)
}

/// Slot 0/3: navigator.deviceMemory.
pub fn device_memory(device_memory: f64) -> String {
    case5_or_6(3, device_memory)
}

/// Slot 0/4: screen.width / availWidth / height / availHeight.
pub fn screen(width: i64, avail_width: i64, height: i64, avail_height: i64) -> String {
    case7(
        4,
        &(pack15_16(width, avail_width) + &pack15_16(height, avail_height)),
    )
}

/// Slot 0/5: screen.colorDepth.
pub fn color_depth(color_depth: i64) -> String {
    case5(5, color_depth)
}

/// Slot 0/6: navigator.hardwareConcurrency.
pub fn hardware_concurrency(hc: i64) -> String {
    case5(6, hc)
}

/// Slot 0/7: window.devicePixelRatio.
pub fn device_pixel_ratio(dpr: f64) -> String {
    case5_or_6(7, dpr)
}

/// Slot 0/8: timezone offset and summertime difference (each `/15`).
pub fn timezone(tz_offset: i64, summertime_diff: i64) -> String {
    case7(
        8,
        &(n_hex((tz_offset / 15) as u64, 2) + &n_hex((summertime_diff / 15) as u64, 2)),
    )
}

/// Slot 0/9: mime-type set (sorted, joined, MMH3-hashed).
pub fn mime_types(mime_types: &[String]) -> String {
    case7(9, &mmh3_string_set_hex(mime_types))
}

/// Slot 0/10: plugin set (sorted, joined, MMH3-hashed).
pub fn plugins(plugins: &[String]) -> String {
    case7(10, &mmh3_string_set_hex(plugins))
}

/// Slot 0/11: 12-bool navigator probe vector.
pub fn navigator_bits(bits: &[bool]) -> String {
    case7(11, &bits_to_hex(bits))
}

/// Slot 0/12: navigator.userAgent (XXTEA frame).
pub fn user_agent(user_agent: &str, init: i64) -> String {
    case7(12, &encode_xxtea_frame(12, user_agent, init))
}

/// Slot 0/13: first canvas-fingerprint hash.
pub fn canvas_hash(hash_hex: &str, init: i64) -> String {
    case4(13, hash_hex, init)
}

/// Slot 0/14: 3-bool mediaDevices probe.
pub fn enumerate_devices(bits: &[bool]) -> String {
    case7(14, &bits_to_hex(bits))
}

/// Slot 0/17: navigator.productSub.
pub fn product_sub(product_sub: &str, init: i64) -> String {
    encode_optional_index(
        17,
        index_of(PRODUCT_SUB_LUT, product_sub),
        product_sub,
        init,
    )
}

/// Slot 0/18: second canvas-fingerprint hash.
pub fn canvas_hash2(hash_hex: &str, init: i64) -> String {
    case4(18, hash_hex, init)
}

/// Slot 0/19: WebGL UNMASKED_RENDERER.
pub fn webgl_renderer(renderer: &str, init: i64) -> String {
    case4(19, renderer, init)
}

/// Slot 0/20: epoch+2-months locale-formatted date.
pub fn locale_date_string(s: &str, init: i64) -> String {
    case4(20, s, init)
}

/// Slot 0/21: 8-bool automation-detection vector.
pub fn automation_bits(bits: &[bool]) -> String {
    case7(21, &bits_to_hex(bits))
}

/// Slot 0/22: window.eval.toString().length.
pub fn eval_length(n: i64) -> String {
    case5(22, n)
}

/// Slot 0/24: maximum call-stack depth.
pub fn max_call_stack_size(n: i64) -> String {
    case5(24, n)
}

/// Slot 0/25: recursion-error message.
pub fn call_stack_error_msg(msg: &str, init: i64) -> String {
    encode_optional_index(25, index_of(CALL_STACK_MESSAGE_LUT, msg), msg, init)
}

/// Slot 0/26: recursion-error type name.
pub fn call_stack_error_name(name: &str, init: i64) -> String {
    encode_optional_index(26, index_of(CALL_STACK_NAME_LUT, name), name, init)
}

/// Slot 0/27: recursion-error stack length.
pub fn call_stack_error_stack_len(n: i64) -> String {
    case5(27, n)
}

/// Slot 0/28: touch-event probe signature (raw hex payload).
pub fn touch_signature(hex_payload: &str) -> String {
    case7(28, hex_payload)
}

/// Slot 0/29: read-undefined-property error message.
pub fn property_error_msg(msg: &str, init: i64) -> String {
    encode_optional_index(29, index_of(PROPERTY_ERROR_LUT, msg), msg, init)
}

/// Slot 0/30: sorted navigator-property name set.
pub fn navigator_properties(props: &[String]) -> String {
    case7(30, &mmh3_string_set_hex(props))
}

/// Slot 0/31: 8-element canPlayType vector, 2 bits each → 16 bits → 4 hex.
pub fn can_play_type(values: &[i64]) -> String {
    let mut v: u64 = 0;
    for &x in values {
        v = (v << 2) | (x as u64 & 3);
    }
    case7(31, &n_hex(v, 4))
}
