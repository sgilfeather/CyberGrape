use cmake::Config;

fn main() {
    let dst = Config::new("libsaf")
        .define("SAF_PERFORMANCE_LIB", "SAF_USE_APPLE_ACCELERATE")
        .define("SAF_ENABLE_SOFA_READER_MODULE", "1")
        .define("SAF_BUILD_TESTS", "0")
        .no_build_target(true)
        .build();

    println!("cargo:rustc-link-search=native={}", dst.join("build").join("framework").display());
    println!("cargo:rustc-link-search=native={}", dst.join("build").join("examples").display());
    println!("cargo:rustc-link-lib=static=saf");
    println!("cargo:rustc-link-lib=static=saf_example_binauraliser_nf");
}
