# cubism-rs: Rust bindings for Live2D Cubism

A rust wrapper around the [Live2D Cubism SDK](https://live2d.github.io/) with extra functionality.


## General usage notes

The `cubism-core-sys` crate requires the Live2DCubismCore library to build and link properly.
The build script finds the library by reading the environment variable `CUBISM_CORE` for the path.

An example set up would be the following, where `CUBISM_CORE` would have the path of the third-party dir set to it.
```
your-project:
├─src/
│   └─...
├─third-party/
│   ├─Core/
│   │   ├─lib/
│   │   │   ├─linux/
│   │   │   │   └─x86_64/
│   │   │   │       └─Live2DCubismCore.a
│   ...
│   └─Samples/ # needed for the examples to run
└─Cargo.toml

```
