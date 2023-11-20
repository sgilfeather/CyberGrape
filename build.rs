use std::path::PathBuf;

fn main() {
    let libdir_path = PathBuf::from("libsaf")
        .canonicalize()
        .expect("cannot canonicalize path");

    let header_path = PathBuf::from("libsafwrapper.h")
        .canonicalize()
        .expect("cannot canonicalize path");
    let header_path_str = header_path.to_str().expect("Path is not a valid string");

    let saf_header_path = libdir_path.join("framework").join("include");
    let saf_header_path_str = saf_header_path
        .to_str()
        .expect("Path is not a valid string");

    let saf_examples_header_path = libdir_path.join("examples").join("include");
    let saf_examples_header_path_str = saf_examples_header_path
        .to_str()
        .expect("Path is not a valid string");

    let dst = cmake::Config::new("libsaf")
        .define("SAF_PERFORMANCE_LIB", "SAF_USE_APPLE_ACCELERATE")
        .define("SAF_ENABLE_SOFA_READER_MODULE", "1")
        .define("SAF_BUILD_TESTS", "0")
        .no_build_target(true)
        .build();

    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build").join("framework").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build").join("examples").display()
    );
    println!("cargo:rustc-link-lib=static=saf");
    println!("cargo:rustc-link-lib=static=saf_example_binauraliser_nf");
    println!("cargo:rustc-link-lib=framework=Accelerate");
    println!("cargo:rerun-if-changed={}", header_path_str);

    let bindings = bindgen::Builder::default()
        .header(header_path_str)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_arg(format!("-I{}", saf_header_path_str))
        .clang_arg(format!("-I{}", saf_examples_header_path_str))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
