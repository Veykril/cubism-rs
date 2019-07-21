use std::{env, path::PathBuf};

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
    let mut lib_dir = env::var("CUBISM_CORE").map(PathBuf::from).expect(
        "The CUBISM_CORE environment variable hasn't been set! \
         Please set it to your Live2DCubismCore directory before compiling. \
         See the readme for more information.",
    );
    lib_dir.push("Core/lib");
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
    let profile = env::var("PROFILE").unwrap_or_default();
    match (vendor, sys, &*profile) {
        ("pc", "windows", "debug") => println!("cargo:rustc-link-lib=static=Live2DCubismCore_MTd"),
        ("pc", "windows", "release") => println!("cargo:rustc-link-lib=static=Live2DCubismCore_MT"),
        _ => println!("cargo:rustc-link-lib=static=Live2DCubismCore"),
    }
}
