# ockam_ebpf

[![crate][crate-image]][crate-link]
[![docs][docs-image]][docs-link]
[![license][license-image]][license-link]
[![discuss][discuss-image]][discuss-link]

Ockam is a library for building devices that communicate securely, privately
and trustfully with cloud services and other devices.

This crate contains the eBPF part of Ockam Reliable TCP Portals.

### Build

This crate exposes eBPF binary through the `EBPF_BINARY` static constant in the root of the crate. That binary can be
used to attach Ockam eBPF to network devices.

### Features

By default, this crate ships a prebuilt eBPF binary downloaded from the corresponding GitHub release artifacts. This
allows to build Ockam without all the dependencies that are required to build eBPF.

 * build - build the eBPF locally instead of downloading the prebuilt binary. This might be useful during development and debugging.
 * logging - this will enable logs for eBPF. Note that eBPF sends logs to the user space using `AsyncPerfEventArray`, therefore it implies performance penalty.

```bash
cargo build
```

### Requirements to build eBPF

Please refer to [ockam_ebpf_impl/README.md](ockam_ebpf_impl/README.md)

### Requirements to use eBPF

Using ockam with eBPFs requires:
 - Linux
 - root (CAP_BPF, CAP_NET_RAW, CAP_NET_ADMIN, CAP_SYS_ADMIN)

## Usage

Add this to your `Cargo.toml`:

```
[dependencies]
ockam_ebpf = "0.5.0"
```

## License

This code is licensed under the terms of the [Apache License 2.0][license-link].

[crate-image]: https://img.shields.io/crates/v/ockam_ebpf.svg
[crate-link]: https://crates.io/crates/ockam_ebpf

[docs-image]: https://docs.rs/ockam_ebpf/badge.svg
[docs-link]: https://docs.rs/ockam_ebpf

[license-image]: https://img.shields.io/badge/License-Apache%202.0-green.svg
[license-link]: https://github.com/build-trust/ockam/blob/HEAD/LICENSE

[discuss-image]: https://img.shields.io/badge/Discuss-Github%20Discussions-ff70b4.svg
[discuss-link]: https://github.com/build-trust/ockam/discussions
