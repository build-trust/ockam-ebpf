# ockam_ebpf_impl

This crate is shipped as a part of `ockam_ebpf` crate rather than a stand-alone crate. Please refer to the ../README.md
for more information.

### Build

In order to build the crate it's required to copy `Cargo.toml.hidden` file and rename it to `Cargo.toml`. Note, that
`Cargo.toml` file is added to `.gitignore` and shouldn't be commited, instead all changes should be inside
`Cargo.toml.hidden` file. The reason for that is special cargo behaviour that doesn't allow including other crates as
part of a crate. Therefore, if `ockam_ebpf_impl` subdirectory has `Cargo.toml` file, that directory will be completely
ignored during `ockam_ebpf` crate release even if it's added to `include` field of root `Cargo.toml`.

```bash
cargo build
```
### Requirements

Building eBPFs have roughly following requirements:
 - Linux
 - Rust nightly
 - Some dependencies to be installed

Because of that, crate with the eBPF code is kept out of the workspace.
Example of a virtual machine to build and run eBPF can be found in [ubuntu_arm.yaml](../vm/ubuntu_arm.yaml)
