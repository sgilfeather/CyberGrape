use std::path::PathBuf;

fn main() {
    // Get a full path to the SAF library source tree
    let libdir_path = PathBuf::from("libsaf")
        .canonicalize()
        .expect("cannot canonicalize path");
    let libdir_path_str = libdir_path.to_str().expect("Path is not a valid string");

    // Get a full path to the wrapper header file for SAF libs
    let header_path = PathBuf::from("libsafwrapper.h")
        .canonicalize()
        .expect("cannot canonicalize path");
    let header_path_str = header_path.to_str().expect("Path is not a valid string");

    // Get full paths to the two include directories in SAF
    let saf_header_path = libdir_path.join("framework").join("include");
    let saf_header_path_str = saf_header_path
        .to_str()
        .expect("Path is not a valid string");

    let saf_examples_header_path = libdir_path.join("examples").join("include");
    let saf_examples_header_path_str = saf_examples_header_path
        .to_str()
        .expect("Path is not a valid string");

    // Run cmake to build SAF, and record where it was stored
    let dst = cmake::Config::new("libsaf")
        .define("SAF_PERFORMANCE_LIB", "SAF_USE_APPLE_ACCELERATE")
        .define("SAF_ENABLE_SOFA_READER_MODULE", "1")
        .define("CMAKE_OSX_ARCHITECTURES", "arm64;x86_64")
        .define("SAF_BUILD_TESTS", "0")
        .no_build_target(true)
        .build();

    // Tell cargo where to find the static library files we just built
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build").join("framework").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build").join("examples").display()
    );

    // Tell cargo we want to link against SAF, the Binauralizer, and
    // Apple's Accelerate fast math framework
    println!("cargo:rustc-link-lib=static=saf");
    println!("cargo:rustc-link-lib=static=saf_example_Binauralizer_nf");
    println!("cargo:rustc-link-lib=framework=Accelerate");

    // Tell cargo to rebuild SAF if the library changes
    println!("cargo:rerun-if-changed={}", libdir_path_str);

    // Generate Rust bindings for the header files #included in libsafwrapper.h
    let bindings = bindgen::Builder::default()
        .header(header_path_str)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_arg(format!("-I{}", saf_header_path_str))
        .clang_arg(format!("-I{}", saf_examples_header_path_str))
        .generate()
        .expect("Unable to generate bindings");

    // Write those bindings to a file that we'll use in saf_raw.rs
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("bindings.rs");
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}
