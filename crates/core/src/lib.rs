mod engine;

use quickjs_wasm_rs::{messagepack, Context};

use messagepack::{transcode_input, transcode_output};
use once_cell::sync::OnceCell;
use std::alloc::{alloc, dealloc, Layout};
use std::io::{self};
use std::ptr::copy_nonoverlapping;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static mut JS_CONTEXT: OnceCell<Context> = OnceCell::new();
static SCRIPT_NAME: &str = "script.js";

// Unlike C's realloc, zero-length allocations need not have
// unique addresses, so a zero-length allocation may be passed
// in and also requested, but it's ok to return anything that's
// non-zero to indicate success.
const ZERO_SIZE_ALLOCATION_PTR: *mut u8 = 1 as _;

#[export_name = "init_engine"]
pub extern "C" fn init_engine() {
    unsafe {
        let mut context = Context::default();
        context
            .register_globals(io::stderr(), io::stderr())
            .unwrap();
        JS_CONTEXT.set(context).unwrap();
    }
}

#[export_name = "init_src"]
pub unsafe extern "C" fn init_src(js_str_ptr: *mut u8, js_str_len: usize) {
    // TODO: Who is supposed to own this pointer? Is it the caller who allocated, or this module?
    let js = String::from_utf8(Vec::from_raw_parts(js_str_ptr, js_str_len, js_str_len)).unwrap();
    let context = JS_CONTEXT.get().unwrap();
    let _ = context.eval_global(SCRIPT_NAME, &js).unwrap();
}

#[export_name = "execute"]
pub unsafe extern "C" fn execute(
    func_obj_path_is_some: u32,
    func_obj_path_ptr: *mut u8,
    func_obj_path_len: usize,
) {
    let func_obj_path = match func_obj_path_is_some {
        0 => "Shopify.main".to_string(),
        _ => String::from_utf8(Vec::from_raw_parts(
            func_obj_path_ptr,
            func_obj_path_len,
            func_obj_path_len,
        ))
        .unwrap(),
    };

    assert!(func_obj_path != "");

    let context = JS_CONTEXT.get().unwrap();
    let (this, func) = func_obj_path.split('.').fold(
        (
            context.global_object().unwrap(),
            context.global_object().unwrap(),
        ),
        |(_this, func), obj| {
            let next = func.get_property(obj).unwrap();
            (func, next)
        },
    );

    let input_bytes = engine::load().expect("Couldn't load input");
    let input_value = transcode_input(&context, &input_bytes).unwrap();
    let output_value = func.call(&this, &[input_value]);

    if output_value.is_err() {
        panic!("{}", output_value.unwrap_err().to_string());
    }
    let output = transcode_output(output_value.unwrap()).unwrap();
    engine::store(&output).expect("Couldn't store output");
}

#[export_name = "canonical_abi_realloc"]
pub unsafe extern "C" fn canonical_abi_realloc(
    original_ptr: *mut u8,
    original_size: usize,
    alignment: usize,
    new_size: usize,
) -> *mut std::ffi::c_void {
    // 1. Allocate memory of new_size with alignment.
    // 2. If original_ptr != 0
    //    a. copy min(new_size, original_size) bytes from original_ptr to new memory
    //    b. de-allocate original_ptr
    // 3. return new memory ptr

    // https://doc.rust-lang.org/std/alloc/struct.Layout.html
    // https://doc.rust-lang.org/std/alloc/fn.alloc.html
    assert!(new_size >= original_size);

    let new_mem = match new_size {
        0 => ZERO_SIZE_ALLOCATION_PTR,
        _ => alloc(Layout::from_size_align(new_size, alignment).unwrap()),
    };

    if !original_ptr.is_null() && original_size != 0 {
        copy_nonoverlapping(original_ptr, new_mem, original_size);
        canonical_abi_free(original_ptr, original_size, alignment);
    }
    new_mem as _
}

#[export_name = "canonical_abi_free"]
pub unsafe extern "C" fn canonical_abi_free(ptr: *mut u8, size: usize, alignment: usize) {
    if size > 0 {
        dealloc(ptr, Layout::from_size_align(size, alignment).unwrap())
    };
}