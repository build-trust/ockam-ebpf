/// Clone of aya::include_bytes_aligned
#[macro_export]
macro_rules! include_bytes_aligned {
    ($path:expr) => {{
        #[repr(align(32))]
        pub struct Aligned32;

        #[repr(C)]
        pub struct Aligned<Bytes: ?Sized> {
            pub _align: [Aligned32; 0],
            pub bytes: Bytes,
        }

        const ALIGNED: &Aligned<[u8]> = &Aligned {
            _align: [],
            bytes: *include_bytes!($path),
        };

        &ALIGNED.bytes
    }};
}
