use std::{io, path::PathBuf};

fn main() {
    Envs::new();
}
struct Envs;

impl Envs {
    #[cfg(target_os = "macos")]
    fn new() {
        const ENV1: &'static str = "LIBTORCH_USE_PYTORCH";
        const ENV2: &'static str = "DYLD_LIBRARY_PATH";
        let env1 = "1".to_string();
        let old = std::env::var(ENV2).unwrap_or("".to_string());
        let Ok(mut env2) = find_libtorch() else {
            return warn_libtorch_not_found();
        };
        if !old.is_empty() {
            env2 += &format!(":{old}");
        }
        println!("cargo::rustc-env={}={}", ENV1, env1);
        println!("cargo::rustc-env={}={}", ENV2, env2);
    }

    #[cfg(not(target_os = "macos"))]
    fn new() {
        const ENV1: &'static str = "LD_LIBRARY_PATH";
        const ENV2: &'static str = "LIBTORCH";
        let old = std::env::var(ENV1).unwrap_or("".to_string());
        let Ok(mut env1) =
            path_canonicalize(format!("{}/../libtorch/lib", env!("CARGO_MANIFEST_DIR")))
        else {
            return warn_libtorch_not_found();
        };
        if !old.is_empty() {
            env1 += &format!(":{old}");
        }
        let old = std::env::var(ENV2).unwrap_or("".to_string());
        let Ok(mut env2) = path_canonicalize(format!("{}/../libtorch", env!("CARGO_MANIFEST_DIR")))
        else {
            return warn_libtorch_not_found();
        };
        if !old.is_empty() {
            env2 += &format!(":{old}");
        }
        println!("cargo::rustc-env={}={}", ENV1, env1);
        println!("cargo::rustc-env={}={}", ENV2, env2);
    }
}

#[cfg(target_os = "macos")]
fn find_libtorch() -> io::Result<String> {
    use walkdir::WalkDir;
    let path = format!("{}/../pytorch", env!("CARGO_MANIFEST_DIR"));
    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() & path.ends_with("torch") {
            return path_canonicalize(path);
        }
    }
    Err(io::Error::other("libtorch not found"))
}

fn path_canonicalize(path: impl Into<PathBuf>) -> io::Result<String> {
    let path = path.into();
    Ok(path.canonicalize()?.to_string_lossy().to_string())
}

fn warn_libtorch_not_found() {
    println!("cargo::warning=libtorch not found; Run `dist/setup_<your_os>` first");
    panic!("libtorch not found");
}
