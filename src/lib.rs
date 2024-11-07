mod include_bytes_aligned;

#[cfg(feature = "prebuilt")]
pub const EBPF_BINARY: &[u8] =
    include_bytes_aligned!(concat!(env!("OUT_DIR"), "/ockam_ebpf_prebuilt"));

#[cfg(not(feature = "prebuilt"))]
pub const EBPF_BINARY: &[u8] = include_bytes_aligned!(concat!(
    env!("OUT_DIR"),
    "/ockam_ebpf/bpfel-unknown-none/release/ockam_ebpf"
));
