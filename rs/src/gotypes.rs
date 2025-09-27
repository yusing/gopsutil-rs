#[cfg(target_pointer_width = "64")]
use std::ffi::c_longlong;
use std::{
    collections::HashMap,
    ffi::OsStr,
    os::unix::ffi::OsStrExt,
    path::Path,
    slice,
    str::FromStr,
    sync::{LazyLock, RwLock},
};

#[cfg(target_pointer_width = "64")]
pub type GoInt = c_longlong;

#[cfg(target_pointer_width = "32")]
pub type GoInt = c_int;

// #[repr(C)]
// pub struct GoSlice<T> {
//     p: *const T,
//     n: GoInt,
//     cap: GoInt,
// }

#[repr(C)]
pub struct GoString {
    p: *const u8,
    n: GoInt,
}

// pub type GoByteSlice = GoSlice<u8>;

// &GoString -> &OsStr
impl From<&GoString> for &OsStr {
    fn from(go_string: &GoString) -> Self {
        unsafe { OsStr::from_bytes(slice::from_raw_parts(go_string.p, go_string.n as usize)) }
    }
}

// &GoString -> &str
impl From<&GoString> for &str {
    fn from(go_string: &GoString) -> Self {
        unsafe {
            std::str::from_utf8_unchecked(slice::from_raw_parts(go_string.p, go_string.n as usize))
        }
    }
}

// &GoString -> String
impl From<&GoString> for String {
    fn from(go_string: &GoString) -> Self {
        String::from_str(go_string.into()).unwrap()
    }
}

// &String -> GoString
impl From<&String> for GoString {
    fn from(string: &String) -> Self {
        let interned = intern_string(string);
        Self {
            p: interned.as_ptr(),
            n: interned.len() as GoInt,
        }
    }
}

// &str -> GoString
impl From<&str> for GoString {
    fn from(string: &str) -> Self {
        let interned = intern_string(string);
        Self {
            p: interned.as_ptr(),
            n: interned.len() as GoInt,
        }
    }
}

// &Path -> GoString
impl From<&Path> for GoString {
    fn from(path: &Path) -> Self {
        let interned = intern_os_string(path.as_os_str());
        Self {
            p: interned.as_ptr(),
            n: interned.len() as GoInt,
        }
    }
}

// &OsStr -> GoString
impl From<&OsStr> for GoString {
    fn from(os_str: &OsStr) -> Self {
        let interned = intern_os_string(os_str);
        Self {
            p: interned.as_ptr(),
            n: interned.len() as GoInt,
        }
    }
}

// Static string interning system - strings live forever, same content allocated once.
// This allows go to reference the string directly without concerning about use-after-free.
static STRING_INTERNER: LazyLock<RwLock<HashMap<String, &'static str>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Intern a string into the static map, returning a pointer to the long-lived string
pub fn intern_string(s: &str) -> &'static str {
    // First try read lock to see if string already exists
    {
        let map = STRING_INTERNER.read().unwrap();
        if let Some(&interned) = map.get(s) {
            return interned;
        }
    }

    // Need to insert new string - acquire write lock
    let mut map = STRING_INTERNER.write().unwrap();

    // Double-check in case another thread inserted it while we waited for write lock
    if let Some(&interned) = map.get(s) {
        return interned;
    }

    // Create a new owned string and leak it to make it 'static
    let owned = s.to_string();
    let leaked: &'static str = Box::leak(owned.into_boxed_str());

    // Store in map and return
    map.insert(s.to_string(), leaked);
    leaked
}

/// Convert OsStr to interned string, handling invalid UTF-8 gracefully
pub fn intern_os_string(os_str: &OsStr) -> &'static str {
    match os_str.to_str() {
        Some(valid_str) => intern_string(valid_str),
        None => {
            // For invalid UTF-8, use lossy conversion
            let lossy = os_str.to_string_lossy();
            intern_string(&lossy)
        }
    }
}
