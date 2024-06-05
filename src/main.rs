#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

mod hardware;

use defmt_rtt as _;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::RgbColor, Drawable};
use panic_probe as _;

use embassy_executor::Spawner;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_nrf::init(Default::default());
    let (mut watch_display, _watch_button, mut watch_battery) =
        hardware::setup_hardware(peripherals).await;

    let text_style = embedded_graphics::mono_font::MonoTextStyle::new(
        &embedded_graphics::mono_font::ascii::FONT_9X15_BOLD,
        RgbColor::WHITE,
    );

    loop {
        if watch_battery.is_charging() {
            let mv = watch_battery.check_battery_voltage().await;
            let mut buffer = itoa::Buffer::new();
            let mv = buffer.format(mv);

            watch_display.backlight_mid();
            watch_display.clear(RgbColor::BLACK).unwrap();
            embedded_graphics::text::Text::new(
                mv,
                embedded_graphics::geometry::Point::new(20, 30),
                text_style,
            )
            .draw(&mut watch_display)
            .unwrap();
        } else {
            watch_display.backlight_off();
            watch_display.clear(RgbColor::BLACK).unwrap();
        }

        embassy_time::Timer::after_secs(5).await;
    }
}
