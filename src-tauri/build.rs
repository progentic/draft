fn main() {
    emit_build_identity();
    tauri_build::build();
}

fn emit_build_identity() {
    println!("cargo:rerun-if-env-changed=DRAFT_BUILD_COMMIT");
    let commit = std::env::var("DRAFT_BUILD_COMMIT").unwrap_or_else(|_| "development".to_owned());
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_owned());
    println!("cargo:rustc-env=DRAFT_BUILD_COMMIT={commit}");
    println!("cargo:rustc-env=DRAFT_BUILD_PROFILE={profile}");
}
