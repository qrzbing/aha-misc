//! Some chaos keywords

use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;

/// A list of antonym words
pub static ANTONYM_VEC: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "enable", "disable", "on", "off", "true", "false", "enabled", "disabled", "allow", "deny",
        "start", "stop", "yes", "no",
    ]
});

/// A set of antonym words
pub static ANTONYM_SET: Lazy<HashSet<&'static str>> =
    Lazy::new(|| ANTONYM_VEC.iter().cloned().collect());

/// A dictionary of antonym words
pub static ANTONYM_DICT: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("enable", "disable");
    m.insert("disable", "enable");
    m.insert("on", "off");
    m.insert("off", "on");
    m.insert("true", "false");
    m.insert("false", "true");
    m.insert("enabled", "disabled");
    m.insert("disabled", "enabled");
    m.insert("allow", "deny");
    m.insert("deny", "allow");
    m.insert("start", "stop");
    m.insert("stop", "start");
    m.insert("yes", "no");
    m.insert("no", "yes");
    m
});

/// Some interesting numbers
pub static INTERESTING_NUMS: Lazy<Vec<usize>> = Lazy::new(|| {
    vec![
        0,
        1,
        u8::MAX as usize,
        i16::MAX as usize,
        i16::MAX as usize - 1,
        u16::MAX as usize,
        u16::MAX as usize + 1,
        i32::MAX as usize,
        i32::MAX as usize - 1,
        u32::MAX as usize,
        u32::MAX as usize + 1,
        usize::MAX,
        usize::MAX - 1,
    ]
});
