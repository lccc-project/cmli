use std::{fs::ReadDir, io::Write, path::PathBuf};

const NON_ARCH_FEATURES: &[&str] = &[
    "error-track-caller",
    "debug-error-track-caller",
    "default",
    "default-archs",
    "all-archs",
];

fn main() -> std::io::Result<()> {
    println!("cargo::rerun-if-changed=build.rs");

    for arch in std::env::var("CARGO_CFG_FEATURE")
        .expect("cargo sets CARGO_CFG_FEATURE, right?")
        .split(',')
        .filter(|f| !NON_ARCH_FEATURES.contains(&f))
    {
        let mut path = PathBuf::from("arch");
        path.push(arch);

        let mut arch_path =
            PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo sets `OUT_DIR` right?"));

        arch_path.push(arch);
        arch_path.set_extension("rs");

        println!("cargo::rustc-env=CMLI_DEF_{arch}={}", arch_path.display());

        let mut file = std::fs::File::create(arch_path)?;
        writeln!(file, "crate::instr_set!{{")?;

        match std::fs::read_dir(&path) {
            Ok(dir) => {
                for dent in dir {
                    let dent = dent?;

                    let path = dent.path();
                    if path.extension().is_some_and(|v| v == "ainfo") {
                        println!("cargo::rerun-if-changed={}", path.display());
                        std::io::copy(&mut std::fs::File::open(path)?, &mut file)?;
                    }

                    writeln!(file)?;
                }
            }
            Err(_) => {
                path.set_extension("ainfo");

                match std::fs::File::open(&path) {
                    Ok(mut input) => {
                        println!("cargo::rerun-if-changed={}", path.display());
                        std::io::copy(&mut file, &mut input)?;
                        writeln!(file)?;
                    }
                    Err(e) => {
                        println!(
                            "cargo::warning=Architecture {arch} selected but `arch/{arch}.ainfo` or `arch/{arch}/` not defined/? {e}"
                        );
                    }
                }
            }
        }

        writeln!(file, "}}")?;
    }
    Ok(())
}
