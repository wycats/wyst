// Example custom build script.
fn main() {
    println!("cargo:rustc-cfg=procmacro2_semver_exempt")
}
