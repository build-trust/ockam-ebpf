//! This crate contains the eBPF part of Ockam Reliable TCP Portals.
//!
//! ## Build
//!
//! Building the eBPF have prerequisites described here https://aya-rs.dev/book/start/development/
//!
//! Using ockam with eBPFs requires:
//!  - Linux
//!  - root (CAP_BPF, CAP_NET_RAW, CAP_NET_ADMIN, CAP_SYS_ADMIN)
//!
//! Example of a virtual machine capable of both building eBPF and running Ockam Privileged Portals
//! can be found in `vm/ubuntu_arm.yaml`.

#![no_std]
#![no_main]

use aya_ebpf::macros::classifier;
use aya_ebpf::programs::TcContext;

mod checksum;
mod checksum_helpers;
mod common;
mod conversion;

#[cfg(feature = "logging")]
mod logger_aya;

#[cfg(not(feature = "logging"))]
mod logger_noop;

use crate::common::Direction;

#[classifier]
pub fn ockam_ingress(ctx: TcContext) -> i32 {
    common::try_handle(&ctx, Direction::Ingress).unwrap_or_else(|ret| ret)
}

#[classifier]
pub fn ockam_egress(ctx: TcContext) -> i32 {
    common::try_handle(&ctx, Direction::Egress).unwrap_or_else(|ret| ret)
}

// TODO: Check if eBPF code can panic at all
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
