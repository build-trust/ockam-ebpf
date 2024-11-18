use core::cmp::PartialEq;
use core::mem;

use network_types::eth::{EthHdr, EtherType};
use network_types::ip::{IpProto, Ipv4Hdr};
use network_types::tcp::TcpHdr;

use aya_ebpf::bindings::TC_ACT_PIPE;
use aya_ebpf::helpers::bpf_ktime_get_boot_ns;
use aya_ebpf::macros::map;
use aya_ebpf::maps::Queue;
use aya_ebpf::programs::TcContext;

use crate::conversion::{convert_ockam_to_tcp, convert_tcp_to_ockam};
use crate::{error, trace, warn};

pub type Proto = u8;

pub type Port = u16;

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct PortQueueElement {
    port: Port,
    proto: Proto,
}

impl PortQueueElement {
    const fn new() -> Self {
        PortQueueElement { port: 0, proto: 0 }
    }
}

/// Ports that we run on
#[map]
static PORT_QUEUE: Queue<PortQueueElement> = Queue::with_max_entries(1024, 0);

static mut PORTS_LEN: usize = 0;
static PORTS_MAX_LEN: usize = 1024;
static mut PORTS: [PortQueueElement; PORTS_MAX_LEN] = [PortQueueElement::new(); PORTS_MAX_LEN];

#[derive(PartialEq)]
pub enum Direction {
    Ingress,
    Egress,
}

#[inline(always)]
pub fn try_handle(ctx: &TcContext, direction: Direction) -> Result<i32, i32> {
    let ethhdr = match ptr_at::<EthHdr>(ctx, 0) {
        None => {
            // Can it happen?
            warn!(ctx, "SKIP non Ether");
            return Ok(TC_ACT_PIPE);
        }
        Some(ethhdr) => ethhdr,
    };

    if unsafe { (*ethhdr).ether_type } != EtherType::Ipv4 {
        trace!(ctx, "SKIP non IPv4");
        return Ok(TC_ACT_PIPE);
    }

    let ipv4hdr = match ptr_at::<Ipv4Hdr>(ctx, EthHdr::LEN) {
        None => {
            // Should not happen
            error!(ctx, "SKIP invalid IPv4 Header");
            return Ok(TC_ACT_PIPE);
        }
        Some(ipv4hdr) => ipv4hdr,
    };
    let ipv4hdr_stack = unsafe { *ipv4hdr };

    unsafe { update_cache_if_needed() };

    if direction == Direction::Ingress && ipv4hdr_stack.proto == IpProto::Tcp {
        return handle_ingress_tcp_protocol(ctx, ipv4hdr);
    }

    if direction == Direction::Egress && is_ockam_proto(ipv4hdr_stack.proto as Proto) {
        return handle_egress_ockam_protocol(ctx, ipv4hdr);
    }

    Ok(TC_ACT_PIPE)
}

#[inline(always)]
unsafe fn update_cache_if_needed() {
    // static mut LAST_UPDATED_NS: u64 = 0;
    // static UPDATE_INTERVAL_NS: u64 = 5 * 1000 * 1000 * 1000; // 5 seconds
    //
    // let time = bpf_ktime_get_boot_ns();
    //
    // if time - LAST_UPDATED_NS > UPDATE_INTERVAL_NS || LAST_UPDATED_NS == 0 {
    //     update_cache();
    //     LAST_UPDATED_NS = bpf_ktime_get_boot_ns();
    // }
}

// #[inline(always)]
// unsafe fn update_cache() {
//     while let Some(queue_element) = PORT_QUEUE.pop() {
//         PORTS[PORTS_LEN] = queue_element;
//         PORTS_LEN += 1;
//     }
// }

#[inline(always)]
fn is_ockam_proto(proto: Proto) -> bool {
    // 146 to 252 are protocol values to be used for custom protocols on top of IPv4.
    // Each ockam node with eBPF portals will generate a random value for itself to minimize risk
    // of intersection with other nodes. Such intersection would not break anything, but decrease
    // performance, as such nodes will receive a copy of packet dedicated for other nodes
    // and discard them.
    // The fact that protocol value is within this range doesn't guarantee that the packet is
    // OCKAM protocol packet, but allows to early skip packets that are definitely not OCKAM
    // protocol
    proto >= 146 && proto <= 252
}

#[inline(always)]
fn handle_ingress_tcp_protocol(ctx: &TcContext, ipv4hdr: *mut Ipv4Hdr) -> Result<i32, i32> {
    let ipv4hdr_stack = unsafe { *ipv4hdr };
    let ipv4hdr_ihl = ipv4hdr_stack.ihl();

    // IPv4 header length must be between 20 and 60 bytes.
    if ipv4hdr_ihl < 5 || ipv4hdr_ihl > 15 {
        error!(ctx, "SKIP invalid IPv4 Header length for TCP");
        return Ok(TC_ACT_PIPE);
    }
    let ipv4hdr_len = ipv4hdr_ihl as usize * 4;

    let src_ip = ipv4hdr_stack.src_addr();
    let dst_ip = ipv4hdr_stack.dst_addr();

    let tcphdr = match ptr_at::<TcpHdr>(ctx, EthHdr::LEN + ipv4hdr_len) {
        None => {
            // Should not happen
            // I haven't found if it's actually guaranteed, but the kernel code I found makes sure
            // that tcp header is inside contiguous kmalloced piece of memory
            error!(ctx, "SKIP invalid TCP Header for TCP");
            return Ok(TC_ACT_PIPE);
        }
        Some(tcphdr) => tcphdr,
    };
    let tcphdr_stack = unsafe { *tcphdr };

    let src_port = u16::from_be(tcphdr_stack.source);
    let dst_port = u16::from_be(tcphdr_stack.dest);

    let syn = tcphdr_stack.syn();
    let ack = tcphdr_stack.ack();
    let fin = tcphdr_stack.fin();
    let rst = tcphdr_stack.rst();

    /*unsafe {
        #[allow(static_mut_refs)]
        for i in 0..PORTS.len() {
            let port = match PORTS.get(i) {
                Some(port) => *port,
                None => return Ok(TC_ACT_PIPE),
            };

            if port.port == dst_port {
                let proto = port.proto;
                trace!(
                    ctx,
                    "CONVERTING TCP PACKET TO {}. SRC: {}.{}.{}.{}:{}, DST: {}.{}.{}.{}:{}. SYN {} ACK {} FIN {} RST {}.",
                    proto,
                    src_ip.octets()[0],
                    src_ip.octets()[1],
                    src_ip.octets()[2],
                    src_ip.octets()[3],
                    src_port,
                    dst_ip.octets()[0],
                    dst_ip.octets()[1],
                    dst_ip.octets()[2],
                    dst_ip.octets()[3],
                    dst_port,
                    syn,
                    ack,
                    fin,
                    rst
                );

                convert_tcp_to_ockam(ctx, ipv4hdr, proto);

                return Ok(TC_ACT_PIPE);
            }
        }
    }*/

    trace!(
        ctx,
        "SKIPPED TCP PACKET SRC: {}.{}.{}.{}:{}, DST: {}.{}.{}.{}:{}. SYN {} ACK {} FIN {} RST {}.",
        src_ip.octets()[0],
        src_ip.octets()[1],
        src_ip.octets()[2],
        src_ip.octets()[3],
        src_port,
        dst_ip.octets()[0],
        dst_ip.octets()[1],
        dst_ip.octets()[2],
        dst_ip.octets()[3],
        dst_port,
        syn,
        ack,
        fin,
        rst
    );

    Ok(TC_ACT_PIPE)
}

#[inline(always)]
fn handle_egress_ockam_protocol(ctx: &TcContext, ipv4hdr: *mut Ipv4Hdr) -> Result<i32, i32> {
    let ipv4hdr_stack = unsafe { *ipv4hdr };
    let proto = ipv4hdr_stack.proto as u8;
    let ipv4hdr_ihl = ipv4hdr_stack.ihl();
    if ipv4hdr_ihl < 5 || ipv4hdr_ihl > 15 {
        error!(ctx, "SKIP invalid IPv4 Header length for OCKAM");
        return Ok(TC_ACT_PIPE);
    }
    let ipv4hdr_len = ipv4hdr_ihl as usize * 4;

    let src_ip = ipv4hdr_stack.src_addr();
    let dst_ip = ipv4hdr_stack.dst_addr();

    if ptr_at::<TcpHdr>(ctx, EthHdr::LEN + ipv4hdr_len).is_none() {
        if let Err(err) = ctx.pull_data((EthHdr::LEN + ipv4hdr_len + TcpHdr::LEN) as u32) {
            error!(
                ctx,
                "Couldn't pull TCP header into contiguous memory. Err {}", err
            );
            return Err(TC_ACT_PIPE);
        }
    };

    let ipv4hdr = match ptr_at::<Ipv4Hdr>(ctx, EthHdr::LEN) {
        None => {
            error!(ctx, "SKIP invalid IPv4 Header");
            return Ok(TC_ACT_PIPE);
        }
        Some(ipv4hdr) => ipv4hdr,
    };

    let tcphdr = match ptr_at::<TcpHdr>(ctx, EthHdr::LEN + ipv4hdr_len) {
        Some(tcphdr) => tcphdr,
        None => {
            error!(
                ctx,
                "Couldn't get TCP header after pulling it into contiguous memory."
            );
            return Err(TC_ACT_PIPE);
        }
    };
    let tcphdr_stack = unsafe { *tcphdr };

    let src_port = u16::from_be(tcphdr_stack.source);
    let dst_port = u16::from_be(tcphdr_stack.dest);

    let syn = tcphdr_stack.syn();
    let ack = tcphdr_stack.ack();
    let fin = tcphdr_stack.fin();
    let rst = tcphdr_stack.rst();

    // unsafe {
    //     #[allow(static_mut_refs)]
    //     for i in 0..PORTS.len() {
    //         let port = match PORTS.get(i) {
    //             Some(port) => *port,
    //             None => return Ok(0),
    //         };
    //         if port.port == src_port {
    //             if proto == port.proto {
    //                 trace!(
    //                     ctx,
    //                     "CONVERTING OCKAM {} packet to TCP. SRC: {}.{}.{}.{}:{}, DST: {}.{}.{}.{}:{}. SYN {} ACK {} FIN {} RST {}.",
    //                     proto,
    //                     src_ip.octets()[0],
    //                     src_ip.octets()[1],
    //                     src_ip.octets()[2],
    //                     src_ip.octets()[3],
    //                     src_port,
    //                     dst_ip.octets()[0],
    //                     dst_ip.octets()[1],
    //                     dst_ip.octets()[2],
    //                     dst_ip.octets()[3],
    //                     dst_port,
    //                     syn,
    //                     ack,
    //                     fin,
    //                     rst
    //                 );
    //
    //                 convert_ockam_to_tcp(ctx, ipv4hdr, tcphdr);
    //
    //                 return Ok(TC_ACT_PIPE);
    //             }
    //         }
    //     }
    // }

    trace!(
        ctx,
        "SKIPPED OCKAM {} PACKET SRC: {}.{}.{}.{}:{}, DST: {}.{}.{}.{}:{}. SYN {} ACK {} FIN {} RST {}.",
        proto,
        src_ip.octets()[0],
        src_ip.octets()[1],
        src_ip.octets()[2],
        src_ip.octets()[3],
        src_port,
        dst_ip.octets()[0],
        dst_ip.octets()[1],
        dst_ip.octets()[2],
        dst_ip.octets()[3],
        dst_port,
        syn,
        ack,
        fin,
        rst
    );

    Ok(TC_ACT_PIPE)
}

#[inline(always)]
pub fn ptr_at<T>(ctx: &TcContext, offset: usize) -> Option<*mut T> {
    let start = ctx.data() + offset;
    let end = ctx.data_end();

    if start + mem::size_of::<T>() > end {
        return None;
    }

    Some((start as *mut u8).cast::<T>())
}