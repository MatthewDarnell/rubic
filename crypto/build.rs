extern crate cc;

#[cfg(not(feature="hash"))]
fn main() {
    println!("Skipping crypto Build Step, feature is only 'random'");
}

#[cfg(feature = "encryption")]
fn main() {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let cpu: &str = match arch {
        "x86" => "_X86_",
        "x86_64" => "_X86_",
        _ => "_AMD64_"
    };
    let extra_four_q_define: &str = match cpu {
        "_X86_" => "_BOGUS_",
        _ => "_ARM_"    //Mac M1 need this
    };

    if os == "windows" {
        return cc::Build::new()
            .file("../ffi-deps/chopper-win.cpp")
            .define("_MSC_VER", "1")
            .define("_AMD64_", "1")
            .compile("Chopper");
    }

    cc::Build::new()
        .define("__LINUX__", "1")
        .define(cpu, "1")
        .define(extra_four_q_define, "1")
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

    let mut binding = cc::Build::new();
    let k12 = binding
        .include("../ffi-deps/K12/lib")
        .file("../ffi-deps/K12/lib/KangarooTwelve.c");

    if os == "linux" {
        k12
            .include("../ffi-deps/K12/lib/Optimized64")
            .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-AVX512.s")
            .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-AVX2.s")
            .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-AVX512-plainC.c")
            .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-opt64.c")
            .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-timesN-AVX512.c")
            .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-timesN-AVX2.c")
            .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-timesN-SSSE3.c")
            .file("../ffi-deps/K12/lib/Optimized64/KeccakP-1600-runtimeDispatch.c")
            .flag("-march=native")
            .flag("-mavx512vl")
            .flag("-mavx512f")
            .flag("-msse3")
            .compile("KangarooTwelve");

        cc::Build::new()
            .file("../ffi-deps/chopper-linux.cpp")
            .define("__LINUX__", "1")
            .define(cpu, "1")
            .compile("Chopper")
    } else {
        k12
            .include("../ffi-deps/K12/lib/Inplace32BI")
            .file("../ffi-deps/K12/lib/Inplace32BI/KeccakP-1600-inplace32BI.c")
            .compile("KangarooTwelve");

        cc::Build::new()
            .file("../ffi-deps/chopper-linux.cpp")
            .define("__LINUX__", "1")
            .define("_AMD64_", "1")
            .compile("Chopper")

    }
}
