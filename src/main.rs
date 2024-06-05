#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

// Setup logging
use defmt_rtt as _;
use embassy_time::Timer;
use panic_probe as _;

use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let _ = embassy_nrf::init(Default::default());
    defmt::info!("Starting");

    let _ = spawner.spawn(every_second());
}

#[embassy_executor::task]
async fn every_second() -> ! {
    let mut i: u64 = 0;
    loop {
        defmt::info!("Hello, world! Seconds since boot={}", i);
        i += 1;
        Timer::after_secs(1).await;
    }
}
