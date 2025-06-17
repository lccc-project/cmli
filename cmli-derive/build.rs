#![feature(string_from_utf8_lossy_owned)]

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!(
        "cargo::rustc-env=TARGET={}",
        std::env::var("TARGET").expect("cargo was supposed to set this")
    );

    match std::process::Command::new(
        std::env::var_os("RUSTC").expect("cargo was supposed to set this"),
    )
    .arg("--version")
    .output()
    {
        Ok(output) => {
            if !output.status.success() {
                println!("cargo::warning=running rustc --version failed");

                let st = String::from_utf8_lossy_owned(output.stderr);

                println!("cargo::rustc-env=HASH_SEED_VERSION={st}");
            } else {
                let st = String::from_utf8_lossy_owned(output.stdout);

                println!("cargo::rustc-env=HASH_SEED_VERSION={st}");
            }
        }
        Err(e) => println!("cargo::rustc-env=HASH_SEED_VERSION={e}"),
    }
}
