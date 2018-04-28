extern crate libc;

use std::slice;
use std::ptr::null;
use std::cmp::min;
use libc::{c_void, memcmp};
use std::ffi::CString;
use std::ffi::CStr;
use std::os::raw::c_char;

#[repr(C)]
enum leveldb_compression {
    no_compression = 0,
    #[warn(dead_code)]
    snappy_compression = 1,
}

#[repr(C)]
struct leveldb_t;
#[repr(C)]
struct leveldb_cache_t;
#[repr(C)]
struct leveldb_comparator_t;
#[repr(C)]
struct leveldb_env_t;
//#[repr(C)] struct leveldb_filelock_t;
//#[repr(C)] struct leveldb_filterpolicy_t;
//#[repr(C)] struct leveldb_iterator_t;
#[repr(C)]
struct leveldb_logger_t;
#[repr(C)]
struct leveldb_options_t;
//#[repr(C)] struct leveldb_randomfile_t;
#[repr(C)]
struct leveldb_readoptions_t;
//#[repr(C)] struct leveldb_seqfile_t;
//#[repr(C)] struct leveldb_snapshot_t;
//#[repr(C)] struct leveldb_writablefile_t;
//#[repr(C)] struct leveldb_writebatch_t;
#[repr(C)]
struct leveldb_writeoptions_t;

// XXX use cfg_if! macro
//#[link(name = "leveldb", kind = "static")]
//#[link(name = "stdc++", kind = "static")]
#[link(name = "reveldb")]
extern "C" {
    // XXX create more type-safe wrapper interface
    // XXX use CStr instead of *mut char
    // https://stackoverflow.com/questions/24145823/how-do-i-convert-a-c-string-into-a-rust-string-and-back-via-ffi

    fn leveldb_open(
        options: *const leveldb_options_t,
        name: *const c_char,
        errptr: *mut *mut c_char,
    ) -> *mut leveldb_t;
    fn leveldb_close(db: *const leveldb_t);
    fn leveldb_put(
        db: *mut leveldb_t,
        options: *const leveldb_writeoptions_t,
        key: *const c_char,
        keylen: usize,
        val: *const c_char,
        vallen: usize,
        errptr: *mut *mut c_char,
    );

    /* Returns NULL if not found.  A malloc()ed array otherwise.
   Stores the length of the array in *vallen. */
    fn leveldb_get(
        db: *mut leveldb_t,
        options: *const leveldb_readoptions_t,
        key: *const c_char,
        keylen: usize,
        vallen: *mut usize,
        errptr: *mut *mut c_char,
    ) -> *mut c_char;

    fn leveldb_comparator_create(
        //state: *mut c_void,
        state: *const c_void,
        destructor: extern "C" fn(*mut c_void),
        compare: extern "C" fn(*mut c_void, *const c_void, usize, *const c_void, usize) -> i64,
        name: extern "C" fn(*mut c_void) -> *const str,
    ) -> *mut leveldb_comparator_t;

    fn leveldb_create_default_env() -> *mut leveldb_env_t;
    fn leveldb_cache_create_lru(capacity: usize) -> *mut leveldb_cache_t;
    fn leveldb_env_get_test_directory(env: *mut leveldb_env_t) -> *const c_char;

    // Management operations
    fn leveldb_destroy_db(
        options: *const leveldb_options_t,
        name: *const c_char,
        errptr: *mut *mut c_char,
    );

    // Options
    fn leveldb_options_create() -> *mut leveldb_options_t;

    fn leveldb_options_set_comparator(opt: *mut leveldb_options_t, cmp: *mut leveldb_comparator_t);
    fn leveldb_options_set_create_if_missing(opt: *mut leveldb_options_t, v: bool);
    fn leveldb_options_set_error_if_exists(opt: *mut leveldb_options_t, v: bool);
    fn leveldb_options_set_cache(opt: *mut leveldb_options_t, c: *mut leveldb_cache_t);
    fn leveldb_options_set_env(opt: *mut leveldb_options_t, env: *mut leveldb_env_t);
    fn leveldb_options_set_info_log(opt: *mut leveldb_options_t, l: Option<&mut leveldb_logger_t>);
    fn leveldb_options_set_write_buffer_size(opt: *mut leveldb_options_t, s: usize);
    fn leveldb_options_set_paranoid_checks(opt: *mut leveldb_options_t, v: u8);
    fn leveldb_options_set_max_open_files(opt: *mut leveldb_options_t, n: i64);
    fn leveldb_options_set_block_size(opt: *mut leveldb_options_t, s: usize);
    fn leveldb_options_set_block_restart_interval(opt: *mut leveldb_options_t, n: i64);
    fn leveldb_options_set_max_file_size(opt: *mut leveldb_options_t, s: usize);
    fn leveldb_options_set_compression(opt: *mut leveldb_options_t, t: leveldb_compression);

    // Read options
    fn leveldb_readoptions_create() -> *mut leveldb_readoptions_t;
    fn leveldb_readoptions_set_verify_checksums(opt: *mut leveldb_readoptions_t, v: bool);
    fn leveldb_readoptions_set_fill_cache(opt: *mut leveldb_readoptions_t, v: bool);

    // Write options
    fn leveldb_writeoptions_create() -> *mut leveldb_writeoptions_t;
    fn leveldb_writeoptions_set_sync(opt: *mut leveldb_writeoptions_t, v: bool);

    fn leveldb_compact_range(
        db: *mut leveldb_t,
        start_key: *const c_char,
        start_key_len: usize,
        limit_key: *const c_char,
        limit_key_len: usize,
    );

    // Utility
    fn leveldb_free(ptr: *mut c_char);
    fn leveldb_major_version() -> i64;
    fn leveldb_minor_version() -> i64;
}

extern "C" fn cmp_destroy(_: *mut c_void) {}
extern "C" fn cmp_compare(
    _: *mut c_void,
    a: *const c_void,
    alen: usize,
    b: *const c_void,
    blen: usize,
) -> i64 {
    let n = min(alen, blen);
    let r = unsafe {
        // XXX more Rusty way
        memcmp(a, b, n)
    };
    if r != 0 {
        return r as i64;
    }
    if alen < blen {
        return -1;
    } else if alen > blen {
        return 1;
    } else {
        return 0;
    }
}

extern "C" fn cmp_name(_: *mut c_void) -> *const str {
    return "foo";
}

fn free_c_str_if_not_null(ptr: *mut *mut c_char) {
    unsafe {
        if !(*ptr).is_null() {
            libc::free(*ptr as *mut c_void);
            *ptr = null::<char>() as *mut c_char;
        }
    }
}

fn check_get(
    db: *mut leveldb_t,
    roptions: *mut leveldb_readoptions_t,
    key: &str,
    expected: Option<&str>,
) {
    let mut err: *mut c_char = null::<char>() as *mut c_char;
    let key = CString::new(key).unwrap();
    let mut val_len: usize = 0;
    let mut val = unsafe {
        leveldb_get(
            db,
            roptions,
            key.as_ptr(),
            key.to_bytes().len(),
            &mut val_len,
            &mut err,
        )
    };
    assert!(err.is_null());
    check_equal(expected, val, val_len);
    // XXX This should be leveldb_free(&mut val as *mut).
    //     See comment above leveldb_free decl in leveldb/c.h.
    free_c_str_if_not_null(&mut val);
}

fn check_equal(expected: Option<&str>, v: *const c_char, n: usize) {
    if let Some(expected) = expected {
        assert!(!v.is_null());

        // It does not have terminatating nul.
        let expected = CString::new(expected).unwrap().into_bytes();

        let val = unsafe { slice::from_raw_parts(v as *const u8, n) };
        assert_eq!(expected, val);
    } else {
        assert!(v.is_null());
    }
}

fn main() {
    unsafe {
        assert!(leveldb_major_version() >= 1);
        assert!(leveldb_minor_version() >= 1);

        /* Phase: create_objects */

        let cmp = leveldb_comparator_create(null(), cmp_destroy, cmp_compare, cmp_name);
        let env = leveldb_create_default_env();
        let cache = leveldb_cache_create_lru(100000);
        let dbname = leveldb_env_get_test_directory(env);
        assert!(!dbname.is_null());

        let options = leveldb_options_create();
        leveldb_options_set_comparator(options, cmp);
        leveldb_options_set_error_if_exists(options, true); // XXX 1
        leveldb_options_set_cache(options, cache);
        leveldb_options_set_env(options, env);
        leveldb_options_set_info_log(options, None);
        leveldb_options_set_write_buffer_size(options, 100000);
        leveldb_options_set_paranoid_checks(options, 1);
        leveldb_options_set_max_open_files(options, 10);
        leveldb_options_set_block_size(options, 1024);
        leveldb_options_set_block_restart_interval(options, 8);
        leveldb_options_set_max_file_size(options, 3 << 20);
        leveldb_options_set_compression(options, leveldb_compression::no_compression);

        let roptions = leveldb_readoptions_create();
        leveldb_readoptions_set_verify_checksums(roptions, true); // XXX 1
        leveldb_readoptions_set_fill_cache(roptions, false); // XXX 0

        let woptions = leveldb_writeoptions_create();
        leveldb_writeoptions_set_sync(woptions, true); // XXX 1

        // Phase: destroy

        let mut err: *mut c_char = null::<char>() as *mut c_char;
        leveldb_destroy_db(options, dbname, &mut err as *mut *mut c_char);
        free_c_str_if_not_null(&mut err as *mut *mut c_char);

        // Phase: open_error

        let mut err: *mut c_char = null::<char>() as *mut c_char;
        let db = leveldb_open(options, dbname, &mut err);
        assert!(!err.is_null()); // missing db file
        free_c_str_if_not_null(&mut err as *mut *mut c_char);

        // Phase: leveldb_free

        let mut err: *mut c_char = null::<char>() as *mut c_char;
        let db = leveldb_open(options, dbname, &mut err);
        assert!(!err.is_null());
        println!("{:?}", CStr::from_ptr(err).to_string_lossy()); // XXX debug
        leveldb_free(err);

        // Phase: open

        let mut err: *mut c_char = null::<char>() as *mut c_char;
        leveldb_options_set_create_if_missing(options, true); // XXX 1
        let db = leveldb_open(options, dbname, &mut err);
        assert!(err.is_null());
        check_get(db, roptions, "foo", None);

        // Phase: put
        let mut err: *mut c_char = null::<char>() as *mut c_char;
        let key = CString::new("foo").unwrap();
        let val = CString::new("hello").unwrap();
        leveldb_put(
            db,
            woptions,
            key.as_ptr(),
            key.as_bytes().len(),
            val.as_ptr(), // can be non-null terminated
            val.as_bytes().len(),
            &mut err,
        );
        assert!(err.is_null());
        check_get(db, roptions, "foo", Some("hello"));

        // Phase: compactall
        leveldb_compact_range(db, null::<c_char>(), 0, null::<c_char>(), 0);
        check_get(db, roptions, "foo", Some("hello"));
    }
    println!("done"); // XXX debug
}
