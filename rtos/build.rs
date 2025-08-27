//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put the linker script somewhere the linker can find it
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out.display());

    // The file `memory.x` is loaded by cortex-m-rt's `link.x` script, which
    // is what we specify in `.cargo/config.toml` for Arm builds
    let memory_x = include_bytes!("memory.x");
    let mut f = File::create(out.join("memory.x")).unwrap();
    f.write_all(memory_x).unwrap();
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rerun-if-changed=build.rs");

    // Compiling C tasks
    // TODO: Remove the hardcoded path to the tasks/do it in a better way
    let tasks_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("./tasks");

    println!("{tasks_dir:?}");

    let mut c_files: Vec<PathBuf> = Vec::new();
    visit_dir(&tasks_dir, &mut c_files);

    // Tell Cargo when to rerun the build script
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tasks");
    for f in c_files.iter() {
        println!("cargo:rerun-if-changed={}", f.display());
    }

    let compiler = env::var("CC_arm")
        .or_else(|_| env::var("CC"))
        .unwrap_or_else(|_| "arm-none-eabi-gcc".to_string());

    let mut build = cc::Build::new();

    build.compiler(compiler);
    build.files(c_files);

    build
        .flag_if_supported("-ffreestanding")
        .flag_if_supported("-fno-builtin")
        .flag_if_supported("-fno-exceptions")
        .flag_if_supported("-fno-unwind-tables")
        .flag_if_supported("-fno-asynchronous-unwind-tables")
        .flag_if_supported("-fdata-sections")
        .flag_if_supported("-ffunction-sections")
        .flag_if_supported("-fno-stack-protector")
        .warnings(true);

        build
            .flag_if_supported("-mcpu=cortex-m33")
            .flag_if_supported("-mthumb")
            .flag_if_supported("-mfpu=fpv5-sp-d16")
            .flag_if_supported("-mfloat-abi=hard");

    build.compile("ctasks");

    println!("cargo:rustc-link-lib=static=ctasks");
}

fn visit_dir(dir: &PathBuf, c_out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext {
                "c" => c_out.push(path),
                _ => {}
            }
        }
    }
}
