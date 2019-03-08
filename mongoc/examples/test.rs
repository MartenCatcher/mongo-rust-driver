use std::ffi::CStr;

use mongoc;

fn main() {
    println!("{:?}", "hello");
    unsafe {
        mongoc::mongoc_init();
        let a = mongoc::bson_get_monotonic_time();
        println!("{:?}", a);

        let b = mongoc::mongoc_get_version();

        let c = CStr::from_ptr(b);

        println!("{:?}", c);
    }
    println!("{:?}", "world");
}
