extern crate cc;

#[cfg(not(feature="hash"))]
fn main() {
    println!("Skipping crypto Build Step, feature is only 'random'");
}

#[cfg(feature = "encryption")]
fn main() {
    println!("Running crypto Build Step");

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;


    if os == "windows" {
        cc::Build::new()
            .file("../ffi-deps/chopper-win.cpp")
            .define("_MSC_VER", "1")
            .define("_AMD64_", "1")
            .compile("Chopper")
    } else if os == "linux" {
        if arch == "x86_64" || arch == "x86" {
            cc::Build::new()
                .define("__LINUX__", "1")
                .define("_X86_", "1")
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
                .include("../ffi-deps/K12/lib")
                .include("../ffi-deps/K12/lib/Optimized64")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-AVX512.s")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-AVX2.s")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-AVX512-plainC.c")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-opt64.c")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-timesN-AVX512.c")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-timesN-AVX2.c")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-timesN-SSSE3.c")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-runtimeDispatch.c")
                .file("../ffi-deps/K12/lib/KangarooTwelve.c")
                .flag("-march=native")
                .flag("-mavx512vl")
                .flag("-mavx512f")
                .flag("-msse3")
                .compile("KangarooTwelve");

            cc::Build::new()
                .file("../ffi-deps/chopper-linux.cpp")
                .define("__LINUX__", "1")
                .define("_X86_", "1")
                .compile("Chopper")
        } else {    //ARM
            cc::Build::new()
                .define("__LINUX__", "1")
                .define("_AMD64_", "1")
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
                .include("../ffi-deps/K12/lib")
                .include("../ffi-deps/K12/lib/Optimized64")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-AVX512.s")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-AVX2.s")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-AVX512-plainC.c")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-opt64.c")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-timesN-AVX512.c")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-timesN-AVX2.c")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-timesN-SSSE3.c")
                .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-runtimeDispatch.c")
                .file("../ffi-deps/K12/lib/KangarooTwelve.c")
                .flag("-march=native")
                .flag("-mavx512vl")
                .flag("-mavx512f")
                .flag("-msse3")
                .compile("KangarooTwelve");

            cc::Build::new()
                .file("../ffi-deps/chopper-linux.cpp")
                .define("__LINUX__", "1")
                .define("_AMD64_", "1")
                .compile("Chopper")
        }
    } else {

        if arch == "x86_64" || arch == "x86" {  //Intel Mac
            cc::Build::new()
                .define("__LINUX__", "1")
                .define("_X86_", "1")
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
                .include("../ffi-deps/K12/lib")
                .include("../ffi-deps/K12/lib/Inplace32BI")
                .file("../ffi-deps/K12/lib/Inplace32BI/KeccakP-1600-inplace32BI.c")
                .file("../ffi-deps/K12/lib/KangarooTwelve.c")
                .compile("KangarooTwelve");

            cc::Build::new()
                .file("../ffi-deps/chopper-linux.cpp")
                .define("__LINUX__", "1")
                .define("_AMD64_", "1")
                .compile("Chopper")
        } else {    //Mac M1 Series
            cc::Build::new()
                .define("__LINUX__", "1")
                .define("_AMD64_", "1")
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
                .include("../ffi-deps/K12/lib")
                .include("../ffi-deps/K12/lib/Inplace32BI")
                .file("../ffi-deps/K12/lib/Inplace32BI/KeccakP-1600-inplace32BI.c")
                .file("../ffi-deps/K12/lib/KangarooTwelve.c")
                .compile("KangarooTwelve");

            cc::Build::new()
                .file("../ffi-deps/chopper-linux.cpp")
                .define("__LINUX__", "1")
                .define("_AMD64_", "1")
                .compile("Chopper")
        }
    }
}
