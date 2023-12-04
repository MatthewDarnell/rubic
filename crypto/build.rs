extern crate cc;

#[cfg(not(feature="hash"))]
fn main() {
    println!("Skipping crypto Build Step, feature is only 'random'");
}

#[cfg(feature = "encryption")]
fn main() {
    println!("Running crypto Build Step");


    cc::Build::new()
        .define("__LINUX__", "1")
        .define("_ARM_", "1")
        .define("_AVX_", "1")
        .define("USE_ENDO", "true")
        .include("../ffi-deps/FourQlib/FourQ_32bit")
        .file("../ffi-deps/FourQlib/FourQ_32bit/eccp2.c")
        .file("../ffi-deps/FourQlib/FourQ_32bit/eccp2_no_endo.c")
        .file("../ffi-deps/FourQlib/FourQ_32bit/crypto_util.c")
        .file("../ffi-deps/FourQlib/FourQ_32bit/schnorrq.c")
        .file("../ffi-deps/FourQlib/FourQ_32bit/kex.c")
        .file("../ffi-deps/FourQlib/random/random.c")
        .file("../ffi-deps/FourQlib/sha512/sha512.c")
        .compile("libFourQ");




    cc::Build::new()
        .file("../ffi-deps/chopper.cpp")
        .define("_AMD64_", "1")
        .compile("Chopper")


}
