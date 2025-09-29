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

pub type GoPtr = *const c_void;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GoSlice {
    pub p: GoPtr,
    pub n: GoInt,
    pub cap: GoInt,
}

#[repr(C)]
pub struct GoString {
    pub p: *const u8,
    pub n: GoInt,
}

#[repr(C)]
pub struct GoType {
    pub size: usize,
    ptr_bytes: usize,
    hash: u32,
    tflag: u8,
    align: u8,
    field_align: u8,
    pub kind: u8,
    equal: extern "C" fn(a: GoPtr, b: GoPtr) -> bool,
    gcdata: *const u8,
    str: i32,
    ptr_to_this: i32,
}

#[repr(C)]
pub struct GoMapHeader {
    used: u64,
    seed: usize,
    dir_ptr: GoPtr,
    dir_len: GoInt,
    global_depth: u8,
    global_shift: u8,
    writing: u8,
    tombstone_possible: bool,
    clear_seq: u64,
}

#[repr(C)]
pub struct GoMapType {
    pub this: GoType,
    pub key: &'static GoType,
    pub elem: &'static GoType,
    group: &'static GoType,
    hasher: extern "C" fn(a: GoPtr, b: GoPtr) -> u32,
    group_size: usize,
    slot_size: usize,
    elem_off: usize,
    flags: u32,
}

// pub type GoByteSlice = GoSlice<u8>;

// Vec<T> -> GoSlice<T>
impl<T> From<Vec<T>> for GoSlice {
    fn from(vec: Vec<T>) -> Self {
        Self {
            p: vec.as_ptr() as GoPtr,
            n: vec.len() as GoInt,
            cap: vec.capacity() as GoInt,
        }
    }
}

// [T] -> GoSlice<T>
impl<T> From<&[T]> for GoSlice {
    fn from(slice: &[T]) -> Self {
        Self {
            p: slice.as_ptr() as GoPtr,
            n: slice.len() as GoInt,
            cap: slice.len() as GoInt,
        }
    }
}

pub fn go_strmap_set(m: &GoMapHeader, m_type: &GoMapType, key: GoString, value: GoPtr) {
    unsafe { strmap_set(m, m_type, key, value) }
}

pub fn go_slice_clone_into(dst: &GoSlice, src: &GoSlice, elem_type: &GoType) {
    unsafe { _go_slice_clone_into(dst, src, elem_type) }
}

unsafe extern "C" {
    #[link_name = "StrMapSet"]
    pub fn strmap_set(m: &GoMapHeader, m_type: &GoMapType, key: GoString, value: GoPtr);
    #[link_name = "SliceCloneInto"]
    pub fn _go_slice_clone_into(dst: &GoSlice, src: &GoSlice, elem_type: &GoType);
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
        intern_string(string.as_str())
    }
}

// &String -> GoString
impl From<&String> for GoString {
    fn from(string: &String) -> Self {
        intern_string(string)
    }
}

// &str -> GoString
impl From<&str> for GoString {
    fn from(string: &str) -> Self {
        intern_string(string)
    }
}

// &Path -> GoString
impl From<&Path> for GoString {
    fn from(path: &Path) -> Self {
        intern_os_string(path.as_os_str())
    }
}

// &OsStr -> GoString
impl From<&OsStr> for GoString {
    fn from(os_str: &OsStr) -> Self {
        intern_os_string(os_str)
    }
}

// anything -> *const ()
pub fn any_to_go_ptr<T>(any: &T) -> GoPtr {
    any as *const T as GoPtr
}

// Static string interning system - strings live forever, same content allocated once.
// This allows go to reference the string directly without concerning about use-after-free.
static STRING_INTERNER: LazyLock<RwLock<HashMap<String, &'static str>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Intern a string into the static map, returning a pointer to the long-lived string
pub fn intern_string(s: &str) -> GoString {
    // First try read lock to see if string already exists
    {
        let map = STRING_INTERNER.read().unwrap();
        if let Some(&interned) = map.get(s) {
            GoString {
                p: interned.as_ptr(),
                n: interned.len() as GoInt,
            };
        }
    }

    // Need to insert new string - acquire write lock
    let mut map = STRING_INTERNER.write().unwrap();

    // Double-check in case another thread inserted it while we waited for write lock
    if let Some(&interned) = map.get(s) {
        return GoString {
            p: interned.as_ptr(),
            n: interned.len() as GoInt,
        };
    }

    // Create a new owned string and leak it to make it 'static
    let owned = s.to_string();
    let leaked: &'static str = Box::leak(owned.into_boxed_str());

    // Store in map and return
    map.insert(s.to_string(), leaked);
    GoString {
        p: leaked.as_ptr(),
        n: leaked.len() as GoInt,
    }
}

/// Convert OsStr to interned string, handling invalid UTF-8 gracefully
pub fn intern_os_string(os_str: &OsStr) -> GoString {
    match os_str.to_str() {
        Some(valid_str) => intern_string(valid_str),
        None => {
            // For invalid UTF-8, use lossy conversion
            let lossy = os_str.to_string_lossy();
            intern_string(&lossy)
        }
    }
}
