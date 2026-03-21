fn main() {
    let bridge_source = "src/qalculate_bridge.cpp";

    println!("cargo:rerun-if-changed={bridge_source}");

    let libqalculate = pkg_config::Config::new()
        .atleast_version("5.9.0")
        .probe("libqalculate")
        .expect("libqalculate development files are required");

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file(bridge_source)
        .flag_if_supported("-std=c++17");

    for include_path in libqalculate.include_paths {
        build.include(include_path);
    }

    build.compile("qalculate_bridge");
}
