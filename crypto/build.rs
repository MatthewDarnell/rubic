extern crate cc;

fn main() {
    /*

    MS FourQ Lib Builder for Windows

    cc::Build::new()
        .define("__WINDOWS__", "1")
       // .define("_AMD64_", "1")
        .define("_X86_", "1")
        .define("_AVX_", "1")
        .define("USE_ENDO", "true")
        .include("FourQlib/FourQ_32bit")
        //.file("FourQlib/FourQ_32bit/AMD64/consts.c")
       // .file("FourQlib/FourQ_32bit/eccp2_core.c")
        .file("FourQlib/FourQ_32bit/eccp2.c")
        .file("FourQlib/FourQ_32bit/eccp2_no_endo.c")
        .file("FourQlib/FourQ_32bit/crypto_util.c")
        .file("FourQlib/FourQ_32bit/schnorrq.c")
        //.file("FourQlib/FourQ_32bit/hash_to_curve.c")
        .file("FourQlib/FourQ_32bit/kex.c")
        .file("FourQlib/random/random.c")
        .file("FourQlib/sha512/sha512.c")
        .compile("libFourQ");

     */

    // Chopper (CFB Provided Cpp Crypto Functions)
    cc::Build::new()
        .file("../ffi-deps/chopper.cpp")
        .define("_AMD64_", "1")
        .define("_AVX_", "1")
        .compile("Chopper")
}
