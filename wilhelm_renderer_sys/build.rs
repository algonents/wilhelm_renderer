use std::env;

fn env_off(val: &str) -> bool {
    matches!(val.to_ascii_lowercase().as_str(), "0" | "off" | "false" | "no")
}

fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        // don't build the native dependencies for doc generation
        return;
    }
    println!("cargo:rerun-if-changed=cpp/CMakeLists.txt");
    println!("cargo:rerun-if-changed=cpp/glrenderer.cpp");
    println!("cargo:rerun-if-changed=cpp/glrenderer.h");
    println!("cargo:rerun-if-env-changed=GLRENDERER_BUILD_X11");
    println!("cargo:rerun-if-env-changed=GLRENDERER_LINK_GL");

    // Publish the bundled GLFW include path so direct dependents (e.g.
    // wilhelm_renderer_imgui) can compile their own GLFW-using code against the
    // exact same headers we link against. Available downstream as
    // DEP_WILHELM_RENDERER_INCLUDE (derived from the `links` key in Cargo.toml).
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:include={}/cpp/glfw-3.4/include", manifest_dir);

    let target = env::var("TARGET").unwrap();

    let mut cmake_config = cmake::Config::new("cpp");
    cmake_config.build_target("glrenderer").static_crt(true);

    // Allow Wayland-only builds (no X11 headers required), e.g. for embedded
    // kiosk targets: GLRENDERER_BUILD_X11=OFF disables the GLFW X11 backend.
    if let Ok(val) = env::var("GLRENDERER_BUILD_X11") {
        cmake_config.define("GLRENDERER_BUILD_X11", if env_off(&val) { "OFF" } else { "ON" });
    }

    // GL functions are loaded at runtime by glad; EGL-only platforms whose
    // Mesa ships no libGL (no GLX) set GLRENDERER_LINK_GL=OFF to skip the
    // explicit libGL link.
    let link_gl = !env::var("GLRENDERER_LINK_GL").is_ok_and(|v| env_off(&v));
    if !link_gl {
        cmake_config.define("GLRENDERER_LINK_GL", "OFF");
    }

    let dst = cmake_config.build();

    let cmake_build_output = dst.join("build");

    let profile = env::var("PROFILE").unwrap();

    // handle platform-specific configuration
    if target.contains("linux") {
        println!(
            "cargo:rustc-link-search=native={}",
            cmake_build_output.display()
        );
        println!("cargo:rustc-link-lib=static=glrenderer");
        println!("cargo:rustc-link-lib=static=glfw3");
        // FreeType uses 'd' suffix for debug builds
        if profile == "debug" {
            println!("cargo:rustc-link-lib=static=freetyped");
        } else {
            println!("cargo:rustc-link-lib=static=freetype");
        }

        if link_gl {
            println!("cargo:rustc-link-lib=dylib=GL");
        }
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if target.contains("apple") {
        println!(
            "cargo:rustc-link-search=native={}",
            cmake_build_output.display()
        );
        println!("cargo:rustc-link-lib=static=glrenderer");
        println!("cargo:rustc-link-lib=static=glfw3");
        // FreeType uses 'd' suffix for debug builds
        if profile == "debug" {
            println!("cargo:rustc-link-lib=static=freetyped");
        } else {
            println!("cargo:rustc-link-lib=static=freetype");
        }

        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=CoreVideo");

        println!("cargo:rustc-link-lib=dylib=c++");
    } else if target.contains("windows") {
        let build_dir = if profile == "debug" {
            cmake_build_output.join("Debug")
        } else {
            cmake_build_output.join("Release")
        };

        println!("cargo:rustc-link-search=native={}", build_dir.display());
        println!("cargo:rustc-link-lib=static=glrenderer");
        println!("cargo:rustc-link-lib=static=glfw3");
        // FreeType uses 'd' suffix for debug builds
        if profile == "debug" {
            println!("cargo:rustc-link-lib=static=freetyped");
        } else {
            println!("cargo:rustc-link-lib=static=freetype");
        }

        // Link Windows system libraries
        println!("cargo:rustc-link-lib=dylib=opengl32");
        println!("cargo:rustc-link-lib=dylib=gdi32");
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=shell32");
        println!("cargo:rustc-link-lib=dylib=kernel32");

        // Link the C++ runtime (adjust depending on compiler)
        // println!("cargo:rustc-link-lib=dylib=stdc++"); // for MinGW/gcc
        println!("cargo:rustc-link-lib=dylib=msvcrt"); // for MSVC (uncomment if needed)
    }
}
