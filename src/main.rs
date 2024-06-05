#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use nrf52832_hal as hal;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut i: u64 = 0;
    loop {
        defmt::info!("Hello, world! Count={}", i);
        i += 1;

        if i > 10000 {
            panic!("Oops");
        }
    }
}
