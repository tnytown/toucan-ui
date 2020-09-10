use std::process::Command;

extern crate cpp_build;

// https://github.com/woboq/qmetaobject-rs/blob/f45dfc13933f106ff833045f6830b79200782a08/examples/graph/build.rs
fn qmake_query(var: &str) -> String {
    String::from_utf8(
        Command::new("qmake")
            .args(&["-query", var])
            .output()
            .expect("Failed to execute qmake. Make sure 'qmake' is in your path")
            .stdout,
    )
    .expect("UTF-8 conversion failed")
}

fn moc_header(hdr: &str) -> String {
    let pth = format!("{}/toucan_moc.cpp", std::env::var("OUT_DIR").unwrap());
    Command::new("moc")
        .args(&["-o", &pth])
        .args(&[hdr])
        .output()
        .expect("unable to execute moc");
    pth
}

fn main() {
    let qt_include_path = qmake_query("QT_INSTALL_HEADERS");
    let qt_library_path = qmake_query("QT_INSTALL_LIBS");

    cpp_build::Config::new()
        .include(qt_include_path.trim())
        .include(qt_include_path.trim().to_owned() + "/QtWidgets")
        .include(qt_include_path.trim().to_owned() + "/QtCore")
        .file(moc_header("qthax.h"))
        .build("src/qthax.rs");

    let macos_lib_search = if cfg!(target_os = "macos") {
        "=framework"
    } else {
        ""
    };
    let macos_lib_framework = if cfg!(target_os = "macos") { "" } else { "5" };

    println!("cargo:rerun-if-changed=qthax.h");
    println!(
        "cargo:rustc-link-search{}={}",
        macos_lib_search,
        qt_library_path.trim()
    );
    println!(
        "cargo:rustc-link-lib{}=Qt{}Widgets",
        macos_lib_search, macos_lib_framework
    );
    println!(
        "cargo:rustc-link-lib{}=Qt{}Core",
        macos_lib_search, macos_lib_framework
    );
}
