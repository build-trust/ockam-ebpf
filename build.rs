use std::path::PathBuf;

#[cfg(feature = "build")]
fn build_ebpf() {
    println!("cargo:rerun-if-changed=./ockam_ebpf_impl");

    use std::env;
    use std::process::Command;

    use fs_extra::dir::CopyOptions;

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let output_file = out_dir.join("ockam_ebpf");
    let ebpf_subdir = out_dir.join("ebpf");

    let ockam_ebpf_impl_subdir = ebpf_subdir.join("ockam_ebpf_impl");
    let ockam_ebpf_impl_target_subdir = ebpf_subdir.join("target");
    let cargo_toml_hidden = ockam_ebpf_impl_subdir.join("Cargo.toml.hidden");
    let cargo_toml = ockam_ebpf_impl_subdir.join("Cargo.toml");

    // Delete the directories for eBPF crate otherwise it doesn't want to recompile after files are
    // updated
    _ = std::fs::remove_dir_all(&ebpf_subdir);

    std::fs::create_dir(&ebpf_subdir).unwrap();
    std::fs::create_dir(&ockam_ebpf_impl_subdir).unwrap();
    std::fs::create_dir(&ockam_ebpf_impl_target_subdir).unwrap();

    // Copy the impl crate contents to build it
    fs_extra::copy_items(
        &[PathBuf::from("./ockam_ebpf_impl")],
        &ebpf_subdir,
        &CopyOptions::new(),
    )
    .unwrap();

    // Copy Cargo.toml.hidden to Cargo.toml
    std::fs::copy(&cargo_toml_hidden, &cargo_toml).unwrap();

    #[allow(unused_mut)]
    let mut args = vec!["build", "--release"];

    #[cfg(feature = "logging")]
    args.extend_from_slice(&["-F", "logging"]);

    let output = Command::new("cargo")
        .current_dir(&ockam_ebpf_impl_subdir)
        .env_remove("RUSTUP_TOOLCHAIN")
        .env_remove("RUSTC")
        .args(&args)
        .env("CARGO_TARGET_DIR", &ockam_ebpf_impl_target_subdir)
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

    let build_output_file =
        ockam_ebpf_impl_target_subdir.join("bpfel-unknown-none/release/ockam_ebpf");
    std::fs::copy(build_output_file, output_file).expect("Couldn't copy ockam_ebpf file");
}

#[cfg(not(feature = "build"))]
fn download_ebpf() {
    use reqwest::blocking::Client;
    use std::env;
    use std::str::FromStr;
    use std::time::Duration;
    use url::Url;

    let version = format!("v{}", env::var("CARGO_PKG_VERSION").unwrap());

    let out_dir = env::var("OUT_DIR").unwrap();

    let output_versioned = PathBuf::from_str(&out_dir)
        .unwrap()
        .join(format!("ockam_ebpf_{version}"));
    let output_file = PathBuf::from_str(&out_dir).unwrap().join("ockam_ebpf");

    // Check if we already downloaded that file
    if output_versioned.exists() {
        std::fs::copy(&output_versioned, &output_file).unwrap();
        return;
    }

    let url = format!(
        "https://github.com/build-trust/ockam-ebpf/releases/download/{version}/ockam_ebpf",
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
