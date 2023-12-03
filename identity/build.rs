extern crate cc;

fn main() {

    cc::Build::new()
        //.define("__WINDOWS__", "1")
        // .define("_AMD64_", "1")
        //.define("_X86_", "1")
        .define("__LINUX__", "1")
        .define("_ARM_", "1")
        .define("_AVX_", "1")
        .define("USE_ENDO", "true")
        .include("../ffi-deps/FourQlib/FourQ_32bit")
        //.file("../ffi-deps/FourQlib/FourQ_32bit/AMD64/consts.c")
        // .file("../ffi-deps/FourQlib/FourQ_32bit/eccp2_core.c")
        .file("../ffi-deps/FourQlib/FourQ_32bit/eccp2.c")
        .file("../ffi-deps/FourQlib/FourQ_32bit/eccp2_no_endo.c")
        .file("../ffi-deps/FourQlib/FourQ_32bit/crypto_util.c")
        .file("../ffi-deps/FourQlib/FourQ_32bit/schnorrq.c")
        //.file("../ffi-deps/FourQlib/FourQ_32bit/hash_to_curve.c")
        .file("../ffi-deps/FourQlib/FourQ_32bit/kex.c")
        .file("../ffi-deps/FourQlib/random/random.c")
        .file("../ffi-deps/FourQlib/sha512/sha512.c")
        .compile("libFourQ");


    println!("cargo:rustc-link-lib=libFourQ");
    println!("cargo:rustc-link-lib=dylib=libFourQ");

    cc::Build::new()
        .file("../ffi-deps/chopper.cpp")
        .define("_AMD64_", "1")
        .define("_AVX_", "1")
        .flag("-mbmi")
        .flag("-msse3")
        .flag("-mavx")
        .flag("-mavx2")
        .include("../ffi-deps/FourQlib/FourQ_32bit/FourQ.h")
        .compile("Chopper")
}

