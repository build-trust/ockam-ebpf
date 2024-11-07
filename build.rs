use std::path::PathBuf;

#[cfg(not(feature = "prebuilt"))]
fn build_ebpf() {
    println!("cargo:rerun-if-changed=ockam_ebpf");

    use std::env;
    use std::process::Command;

    let target_dir = if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
        target_dir
    } else {
        PathBuf::from(env::var("OUT_DIR").unwrap())
            .join("ockam_ebpf")
            .to_str()
            .unwrap()
            .to_string()
    };

    let status = Command::new("cargo")
        .current_dir(PathBuf::from("./ockam_ebpf"))
        .env_remove("RUSTUP_TOOLCHAIN")
        .args(&["build", "--release"])
        .env("CARGO_TARGET_DIR", target_dir)
        .status()
        .expect("failed to build bpf program");

    assert!(status.success());
}

#[cfg(feature = "prebuilt")]
fn download_ebpf() {
    use std::env;
    use std::str::FromStr;
    use std::time::Duration;
    use url::Url;

    let out_dir = env::var("OUT_DIR").unwrap();

    let output_file = PathBuf::from_str(&out_dir)
        .unwrap()
        .join("ockam_ebpf_prebuilt");

    // TODO: Handle updates
    // if output_file.exists() {
    //     return;
    // }

    let url = "https://github.com/build-trust/ockam-ebpf/releases/download/v0.1.0/ockam_ebpf";

    let url = Url::parse(url).unwrap();

    let client_builder = reqwest::blocking::Client::builder();

    // TODO: Also respect other variables, like CARGO_HTTP_PROXY
    let client_builder = if let Ok(http_timeout) = env::var("CARGO_HTTP_TIMEOUT") {
        if let Ok(http_timeout) = u64::from_str(&http_timeout) {
            client_builder.timeout(Some(Duration::from_secs(http_timeout)))
        } else {
            client_builder
        }
    } else {
        client_builder
    };

    let client = client_builder.build().unwrap();

    let ebpf = client
        .get(url)
        .send()
        .expect("Error downloading eBPF")
        .bytes()
        .expect("Error downloading eBPF");

    std::fs::write(&output_file, ebpf).unwrap();
}

fn main() {
    #[cfg(feature = "prebuilt")]
    download_ebpf();

    #[cfg(not(feature = "prebuilt"))]
    build_ebpf();
}
