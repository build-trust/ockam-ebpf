use std::path::PathBuf;

#[cfg(feature = "build")]
fn build_ebpf() {
    println!("cargo:rerun-if-changed=./ockam_ebpf_impl");

    use std::env;
    use std::process::Command;

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let output_file = out_dir.join("ockam_ebpf");

    let target_dir = out_dir.join("ebpf");

    let output = Command::new("cargo")
        .current_dir(PathBuf::from("./ockam_ebpf_impl"))
        .env_remove("RUSTUP_TOOLCHAIN")
        .args(&["build", "--release"])
        .env("CARGO_TARGET_DIR", &target_dir)
        .output();

    let output = match output {
        Ok(output) => output,
        Err(err) => {
            panic!("Failed to execute eBPF compilation. Error: {}", err);
        }
    };

    if !output.status.success() {
        panic!("Couldn't compile eBPF");
    }

    let build_output_file = target_dir.join("bpfel-unknown-none/release/ockam_ebpf");
    std::fs::copy(build_output_file, output_file).expect("Couldn't copy ockam_ebpf file");
}

#[cfg(not(feature = "build"))]
fn download_ebpf() {
    use reqwest::blocking::Client;
    use std::env;
    use std::str::FromStr;
    use std::time::Duration;
    use url::Url;

    const VERSION: &str = "v0.1.0";

    let out_dir = env::var("OUT_DIR").unwrap();

    let output_versioned = PathBuf::from_str(&out_dir)
        .unwrap()
        .join(format!("ockam_ebpf_{VERSION}"));
    let output_file = PathBuf::from_str(&out_dir).unwrap().join("ockam_ebpf");

    // Check if we already downloaded that file
    if output_versioned.exists() {
        std::fs::copy(&output_versioned, &output_file).unwrap();
        return;
    }

    let url = format!(
        "https://github.com/build-trust/ockam-ebpf/releases/download/{VERSION}/ockam_ebpf",
    );

    let url = Url::parse(&url).unwrap();

    let client_builder = Client::builder();

    // TODO: There are a lot of CARGO_* env variables that we should had respected here
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

    let response = client.get(url).send().expect("Error downloading eBPF");

    let ebpf = match response.error_for_status() {
        Ok(response) => response.bytes().expect("Error parsing eBPF response bytes"),
        Err(err) => {
            panic!("Error downloading EBPF: {err}");
        }
    };

    std::fs::write(&output_versioned, &ebpf).expect("Can't copy ockam_ebpf versioned file");
    std::fs::write(&output_file, &ebpf).expect("Can't copy ockam_ebpf file");
}

fn main() {
    #[cfg(not(feature = "build"))]
    download_ebpf();

    #[cfg(feature = "build")]
    build_ebpf();
}
