#![no_std]
#![no_main]

use nrf52832_hal as hal;
use rtt_target::{rprintln, rtt_init_print};

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();
    let mut i: u64 = 0;
    loop {
        rprintln!("Hello, world! Count={}", i);
        i += 1;
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rprintln!("{}", info);
    loop {}
}
