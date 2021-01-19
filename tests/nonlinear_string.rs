/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate mozjs;
extern crate mozjs_sys;
extern crate libc;

use mozjs_sys::jsgc::IntoHandle;
use mozjs::conversions::jsstr_to_string;
use mozjs::jsapi::JSAutoRealm;
use mozjs::jsapi::JS_ConcatStrings;
use mozjs::jsapi::JS_NewGlobalObject;
use mozjs::jsapi::JS_NewStringCopyZ;
use mozjs::jsapi::JS_NewUCStringCopyZ;
use mozjs::jsapi::JS_DeprecatedStringHasLatin1Chars;
use mozjs::jsapi::JS_StringIsLinear;
use mozjs::jsapi::OnNewGlobalHookOption;
use mozjs::rust::{JSEngine, RealmOptions, Runtime, SIMPLE_GLOBAL_CLASS};
use std::ptr;

#[test]
fn nonlinear_string() {
    let engine = JSEngine::init().unwrap();
    let runtime = Runtime::new(engine.handle());
    let context = runtime.cx();
    let h_option = OnNewGlobalHookOption::FireOnNewGlobalHook;
    let c_option = RealmOptions::default();

    unsafe {
        let global = JS_NewGlobalObject(context, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(), h_option, &*c_option);
        rooted!(in(context) let global_root = global);
        let global = global_root.handle();
        let _ac = JSAutoRealm::new(context, global.get());
        let s = b"abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz\0";
        rooted!(in(context) let left = JS_NewStringCopyZ(context, s.as_ptr() as *const _));
        assert!(!left.is_null());
        assert!(JS_StringIsLinear(*left));

        rooted!(in(context) let right = JS_NewStringCopyZ(context, s.as_ptr() as *const _));
        assert!(!right.is_null());
        assert!(JS_StringIsLinear(*right));

        rooted!(in(context) let joined = JS_ConcatStrings(context, left.handle().into_handle(), right.handle().into_handle()));
        assert!(!joined.is_null());
        assert!(!JS_StringIsLinear(*joined));

        let rust_str = jsstr_to_string(context, *joined);
        let expected = String::from_utf8(s[..s.len() - 1].to_owned()).unwrap();
        assert_eq!(rust_str, expected.clone() + &expected);

        let s = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxy\u{abcd}\0";
        let utf16_chars: Vec<u16> = s.encode_utf16().collect();
        rooted!(in(context) let left = JS_NewUCStringCopyZ(context, utf16_chars.as_ptr() as *const _));
        assert!(!left.is_null());
        assert!(!JS_DeprecatedStringHasLatin1Chars(*left));
        assert!(JS_StringIsLinear(*left));

        rooted!(in(context) let right = JS_NewUCStringCopyZ(context, utf16_chars.as_ptr() as *const _));
        assert!(!right.is_null());
        assert!(!JS_DeprecatedStringHasLatin1Chars(*right));
        assert!(JS_StringIsLinear(*right));

        rooted!(in(context) let joined = JS_ConcatStrings(context, left.handle().into_handle(), right.handle().into_handle()));
        assert!(!joined.is_null());
        assert!(!JS_StringIsLinear(*joined));

        let rust_str = jsstr_to_string(context, *joined);
        let expected = s[..s.len() - 1].to_owned();
        assert_eq!(rust_str, expected.clone() + &expected);
    }
}
