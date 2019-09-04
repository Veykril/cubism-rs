use std::{env, path::PathBuf};

fn lookup_cubism_core(arch: &str, vendor: &str, sys: &str, abi: &str, linking_strategy: &str) {
    let mut lib_dir = if let Ok(lib_dir) = env::var("CUBISM_CORE").map(PathBuf::from) {
        lib_dir
    } else {
        // TODO: this won't appear unless the compilation fails.
        println!(
            "cargo:warning=it seems that the CUBISM_CORE environment variable is not set. \
             Please set it to your Live2DCubismCore directory before compiling, \
             or specify the Live2DCubismCore library manually. \
             Check out https://github.com/Veykril/cubism-rs for more information."
        );
        return;
    };

    if linking_strategy == "static" {
        lib_dir.push("Core/lib");
    } else {
        lib_dir.push("Core/dll");
    }

    match (vendor, sys) {
        ("pc", "windows") => {
            lib_dir.push("windows");
            lib_dir.push(match arch {
                "x86_64" => "x86_64",
                "i686" => "x86",
                _ => panic!("unknown windows architecture: {}", arch),
            });
            lib_dir.push("140");
        },
        ("apple", "darwin") => {
            if linking_strategy != "static" {
                panic!(
                    "since Live2DCubismCore is in MH_BUNDLE format (which is deprecated), \
                     dynamic linking on macOS is not supported. \
                     See https://github.com/Veykril/cubism-rs for more information."
                );
            }
            if arch == "i686" {
                panic!("no 32-bit support for macOS.");
            }

            lib_dir.push("macos");
        },
        ("apple", "ios") => {
            unimplemented!("TODO: implement ios linking");
        },
        ("unknown", "linux") => {
            lib_dir.push("linux");
            lib_dir.push(match arch {
                "x86_64" => "x86_64",
                _ => panic!("linux is only supported on x86_64"),
            });
        },
        ("linux", "android") | ("linux", "androideabi") => {
            lib_dir.push("android");
            lib_dir.push(match arch {
                "i686" => "x86",
                "armv7" => "armeabi-v7a",
                "aarch64" => "arm64-v8a",
                _ => panic!("unsupported android architecture: {}", arch),
            });
        },
        _ => panic!(
            "unsupported target triple: {}-{}-{}-{}",
            arch, vendor, sys, abi
        ),
    }
    println!("cargo:rustc-link-search=all={}", lib_dir.display());
}

fn main() {
    println!("cargo:rerun-if-env-changed=CUBISM_CORE");
    let target = env::var("TARGET").unwrap();
    let (arch, vendor, sys, abi) = {
        let mut target_s = target.split('-');
        (
            target_s.next().unwrap_or(""),
            target_s.next().unwrap_or(""),
            target_s.next().unwrap_or(""),
            target_s.next().unwrap_or(""),
        )
    };

    let linking_strategy = if cfg!(feature = "static-link") {
        "static"
    } else {
        "dylib"
    };

    lookup_cubism_core(arch, vendor, sys, abi, linking_strategy);

    let profile = env::var("PROFILE").unwrap_or_default();

    match (vendor, sys, &*profile) {
        ("pc", "windows", "debug") => println!(
            "cargo:rustc-link-lib={}=Live2DCubismCore_MTd",
            linking_strategy
        ),
        ("pc", "windows", "release") => println!(
            "cargo:rustc-link-lib={}=Live2DCubismCore_MT",
            linking_strategy
        ),
        _ => println!("cargo:rustc-link-lib={}=Live2DCubismCore", linking_strategy),
    }
}
