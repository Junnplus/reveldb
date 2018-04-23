use std::mem::transmute;
use std::ffi::CString;
use libc::c_void;
use std::os::raw::c_char;

// Struct

#[repr(u8)]
enum CompressionType {
    NoCompression,
    Snappy,
}

// C Struct

pub struct leveldb_t {}

pub struct leveldb_env_t {}

pub struct leveldb_cache_t {}

pub struct leveldb_logger_t {}

// TODO introduce NonZero
pub struct leveldb_comparator_t {
    state: *mut c_void,
    destructor: extern "C" fn(arg: *mut c_void),
    compare: extern "C" fn(
        arg: *mut c_void,
        a: *mut char,
        alen: *mut usize,
        b: *mut char,
        blen: *mut usize,
    ) -> i32,
    name: extern "C" fn(arg: *mut c_void) -> *mut char,
}

impl Drop for leveldb_comparator_t {
    fn drop(&mut self) {
        (self.destructor)(self.state);
    }
}

pub struct leveldb_options_t<'a> {
    comparator: Option<&'a mut leveldb_comparator_t>,
    create_if_missing: bool,
    cache: Option<&'a mut leveldb_cache_t>,
    env: Option<&'a mut leveldb_env_t>,
    info_log: Option<&'a mut leveldb_logger_t>,
    write_buffer_size: usize,
    paranoid_checks: bool,
    max_open_files: u32,
    block_size: usize,
    block_restart_interval: u32,
    max_file_size: usize,
    compression: CompressionType,
}

// Misc

#[no_mangle]
pub extern "C" fn leveldb_major_version() -> i8 {
    1
}

#[no_mangle]
pub extern "C" fn leveldb_minor_version() -> i8 {
    21
}

// DB

#[no_mangle]
pub extern "C" fn leveldb_open() -> *mut leveldb_t {
    let db = Box::new(leveldb_t {});
    Box::into_raw(db)
}

#[no_mangle]
pub extern "C" fn leveldb_close(db: *mut leveldb_t) {}

#[no_mangle]
pub extern "C" fn leveldb_free(db: *mut leveldb_t) {
    unsafe { Box::from_raw(db) };
}

// Comparator

#[no_mangle]
pub extern "C" fn leveldb_comparator_create(
    state: *mut c_void,
    destructor: extern "C" fn(arg: *mut c_void),
    compare: extern "C" fn(
        arg: *mut c_void,
        a: *mut char,
        alen: *mut usize,
        b: *mut char,
        blen: *mut usize,
    ) -> i32,
    name: extern "C" fn(arg: *mut c_void) -> *mut char,
) -> *mut leveldb_comparator_t {
    let comparator = Box::new(leveldb_comparator_t {
        state: state,
        destructor: destructor,
        compare: compare,
        name: name,
    });
    Box::into_raw(comparator)
}

#[no_mangle]
pub extern "C" fn leveldb_comparator_destroy(cmp: *mut leveldb_comparator_t) {
    unsafe { Box::from_raw(cmp) };
}

// Env
#[no_mangle]
pub extern "C" fn leveldb_create_default_env() -> *mut leveldb_env_t {
    let env = Box::new(leveldb_env_t {});
    Box::into_raw(env)
}

#[no_mangle]
pub extern "C" fn leveldb_env_destroy(env: *mut leveldb_env_t) {
    unsafe { Box::from_raw(env) };
}

// Cache

#[no_mangle]
pub extern "C" fn leveldb_cache_create_lru(capacity: usize) -> *mut leveldb_cache_t {
    let cache = Box::new(leveldb_cache_t {});
    Box::into_raw(cache)
}

#[no_mangle]
pub extern "C" fn leveldb_cache_destroy(cache: *mut leveldb_cache_t) {
    unsafe { Box::from_raw(cache) };
}

// Options

#[no_mangle]
pub extern "C" fn leveldb_options_create<'a>() -> *mut leveldb_options_t<'a> {
    println!("DEBUG: leveldb_options_create");
    // TODO set the default value as original leveldb impl
    let options = Box::new(leveldb_options_t {
        comparator: None,
        create_if_missing: false,
        cache: None,
        env: None,
        info_log: None,
        write_buffer_size: 4 * 1024 * 1024, // 4MB
        paranoid_checks: false,
        max_open_files: 1000,
        block_size: 4 * 1024, // 4KB
        block_restart_interval: 16,
        max_file_size: 10,
        compression: CompressionType::Snappy,
    });
    Box::into_raw(options)
}

#[no_mangle]
pub extern "C" fn leveldb_options_destroy(options: *mut leveldb_options_t) {
    unsafe { Box::from_raw(options) };
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_comparator(
    opt: *mut leveldb_options_t,
    cmp: *mut leveldb_comparator_t,
) {
    let cmp = unsafe { cmp.as_mut().expect("null pointer") };
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    opt.comparator = Some(cmp);
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_error_if_exists(opt: *mut leveldb_options_t, v: u8) {
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    opt.create_if_missing = v != 0;
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_cache(
    opt: *mut leveldb_options_t,
    cache: *mut leveldb_cache_t,
) {
    let cache = unsafe { cache.as_mut().expect("null pointer") };
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    opt.cache = Some(cache);
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_env(opt: *mut leveldb_options_t, env: *mut leveldb_env_t) {
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    if let Some(env) = unsafe { env.as_mut() } {
        opt.env = Some(env);
    }
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_info_log(
    opt: *mut leveldb_options_t,
    l: *mut leveldb_logger_t,
) {
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    if let Some(l) = unsafe { l.as_mut() } {
        opt.info_log = Some(l);
    }
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_write_buffer_size(opt: *mut leveldb_options_t, s: usize) {
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    opt.write_buffer_size = s;
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_max_open_files(opt: *mut leveldb_options_t, n: u32) {
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    opt.max_open_files = n;
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_block_size(opt: *mut leveldb_options_t, n: usize) {
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    opt.block_size = n;
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_block_restart_interval(opt: *mut leveldb_options_t, n: u32) {
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    opt.block_restart_interval = n;
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_max_file_size(opt: *mut leveldb_options_t, s: usize) {
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    opt.max_file_size = s;
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_compression(opt: *mut leveldb_options_t, t: u8) {
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    if 0 <= t && t <= 1 {
        opt.compression = unsafe { transmute(t) };
    } else {
        panic!("expect");
    }
}

#[no_mangle]
pub extern "C" fn leveldb_options_set_paranoid_checks(opt: *mut leveldb_options_t, v: u8) {
    let opt = unsafe { opt.as_mut().expect("null pointer") };
    opt.paranoid_checks = v != 0;
}

#[no_mangle]
pub extern "C" fn leveldb_env_get_test_directory(env: *mut leveldb_env_t) -> *const c_char {
    // XXX uniqueify
    CString::new("/tmp/reveldbtest-0").unwrap().as_ptr()
}
