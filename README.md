# cubism-rs: Rust bindings for Live2D Cubism

A rust wrapper around the [Live2D Cubism SDK](https://live2d.github.io/) with extra functionality.


## General usage notes

The `cubism-core-sys` crate requires the Live2DCubismCore library to build and link properly.
The build script finds the library by reading the environment variable 'CUBISM_CORE' for the path.

If you set the variable to 'third-party' for example, then your project layout should look like this:
```
your-project:
    src/
        *.rs
    third-party/
        lib/
            windows/
                x86/140
                    Live2DCubismCore.lib
                x86_64/140
                    Live2DCubismCore.lib
             ...
    Cargo.toml
```
