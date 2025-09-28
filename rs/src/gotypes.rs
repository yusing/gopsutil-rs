#[cfg(target_pointer_width = "64")]
use std::ffi::c_longlong;
use std::{
    collections::HashMap,
    ffi::{OsStr, c_void},
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GoSlice<T> {
    pub p: *const T,
    pub n: GoInt,
    pub cap: GoInt,
}

#[repr(C)]
pub struct GoString {
    p: *const u8,
    n: GoInt,
}

#[repr(C)]
pub struct GoType {
    size: usize,
    ptr_bytes: usize,
    hash: u32,
    tflag: u8,
    align: u8,
    field_align: u8,
    kind: u8,
    equal: extern "C" fn(a: *const (), b: *const ()) -> bool,
    gcdata: *const u8,
    str: i32,
    ptr_to_this: i32,
}

pub type GoMapHeader = *const c_void;
pub type GoMapType = *const c_void;

// pub type GoByteSlice = GoSlice<u8>;

// Vec<T> -> GoSlice<T>
impl<T> From<Vec<T>> for GoSlice<T> {
    fn from(vec: Vec<T>) -> Self {
        Self {
            p: vec.as_ptr(),
            n: vec.len() as GoInt,
            cap: vec.capacity() as GoInt,
        }
    }
}

// [T] -> GoSlice<T>
impl<T> From<&[T]> for GoSlice<T> {
    fn from(slice: &[T]) -> Self {
        Self {
            p: slice.as_ptr(),
            n: slice.len() as GoInt,
            cap: slice.len() as GoInt,
        }
    }
}

pub fn go_strmap_set(m: GoMapHeader, m_type: GoMapType, key: &GoString, value: *const c_void) {
    unsafe { _go_strmap_set(m, m_type, key, value) };
}

pub fn go_slice_clone_into<T>(
    dst: *const GoSlice<T>,
    src: *const GoSlice<T>,
    elem_type: *const GoType,
) {
    unsafe {
        _go_slice_clone_into(
            dst as *const GoSlice<c_void>,
            src as *const GoSlice<c_void>,
            elem_type,
        )
    }
}

unsafe extern "C" {
    #[link_name = "StrMapSet"]
    pub fn _go_strmap_set(m: GoMapHeader, m_type: GoMapType, key: &GoString, value: *const c_void);
    #[link_name = "SliceCloneInto"]
    pub fn _go_slice_clone_into(
        dst: *const GoSlice<c_void>,
        src: *const GoSlice<c_void>,
        elem_type: *const GoType,
    );
}

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

// String -> GoString
impl From<String> for GoString {
    fn from(string: String) -> Self {
        let interned = intern_string(string.as_str());
        Self {
            p: interned.as_ptr(),
            n: interned.len() as GoInt,
        }
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

// anything -> *const ()
pub fn any_to_c_void<T>(any: &T) -> *const c_void {
    any as *const T as *const c_void
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
