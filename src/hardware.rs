use display_interface_spi::SPIInterface;
use embassy_time::Delay;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
};
use embedded_hal_bus::spi::ExclusiveDevice;

use embassy_nrf::{
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals::SPI2,
    saadc::{self, Saadc},
    spim::{self, Spim},
    Peripherals,
};
use mipidsi::{models::ST7789, Display};

embassy_nrf::bind_interrupts!(struct DisplaySpi {
    SPIM2_SPIS2_SPI2 => spim::InterruptHandler<SPI2>;
});

embassy_nrf::bind_interrupts!(struct VoltageSaadc {
    SAADC => saadc::InterruptHandler;
});

type PermOutput = Output<'static>;
type PermInput = Input<'static>;
type PineDisplay = Display<
    SPIInterface<ExclusiveDevice<Spim<'static, SPI2>, PermOutput, Delay>, PermOutput>,
    ST7789,
    PermOutput,
>;

pub struct WatchDisplay {
    display: PineDisplay,
    backlight_high: PermOutput,
    backlight_mid: PermOutput,
    backlight_low: PermOutput,
}

pub struct WatchButton {
    button_active: PermOutput,
    button_pressed: PermInput,
}

pub struct WatchBattery {
    battery_voltage: Saadc<'static, 1>,
    charging_input: PermInput,
}

pub async fn setup_hardware(p: Peripherals) -> (WatchDisplay, WatchButton, WatchBattery) {
    // Setup display
    let mut spi_cfg = spim::Config::default();
    spi_cfg.mode = spim::MODE_3;
    spi_cfg.frequency = spim::Frequency::M8;

    let spi_bus = spim::Spim::new(p.SPI2, DisplaySpi, p.P0_02, p.P0_04, p.P0_03, spi_cfg);

    let cmd_data = Output::new(p.P0_18, Level::Low, OutputDrive::Standard);
    let chip_select = Output::new(p.P0_25, Level::High, OutputDrive::Standard);

    let display_spi_interface = SPIInterface::new(
        ExclusiveDevice::new(spi_bus, chip_select, Delay).unwrap(),
        cmd_data,
    );

    let display_reset = Output::new(p.P0_26, Level::High, OutputDrive::Standard);
    let display = mipidsi::Builder::new(ST7789, display_spi_interface)
        .invert_colors(mipidsi::options::ColorInversion::Inverted)
        .reset_pin(display_reset)
        .init(&mut Delay)
        .unwrap();

    let backlight_high = Output::new(p.P0_23, Level::Low, OutputDrive::Standard);
    let backlight_mid = Output::new(p.P0_22, Level::High, OutputDrive::Standard);
    let backlight_low = Output::new(p.P0_14, Level::High, OutputDrive::Standard);

    let watch_display = WatchDisplay {
        display,
        backlight_high,
        backlight_mid,
        backlight_low,
    };

    // Setup button
    let button_active = Output::new(p.P0_15, Level::Low, OutputDrive::Standard);
    let button_pressed = Input::new(p.P0_13, Pull::None);

    let watch_button = WatchButton {
        button_active,
        button_pressed,
    };

    // Setup battery voltage
    let config = saadc::Config::default();
    let channel_config = saadc::ChannelConfig::single_ended(p.P0_31);
    let battery_voltage = Saadc::new(p.SAADC, VoltageSaadc, config, [channel_config]);
    let charging_input = Input::new(p.P0_12, Pull::None);

    battery_voltage.calibrate().await;

    let watch_battery = WatchBattery {
        battery_voltage,
        charging_input,
    };

    (watch_display, watch_button, watch_battery)
}

impl WatchDisplay {
    pub fn backlight_off(&mut self) {
        self.backlight_high.set_high();
        self.backlight_mid.set_high();
        self.backlight_low.set_high();
    }

    pub fn backlight_low(&mut self) {
        self.backlight_high.set_high();
        self.backlight_mid.set_high();
        self.backlight_low.set_low();
    }

    pub fn backlight_mid(&mut self) {
        self.backlight_high.set_high();
        self.backlight_mid.set_low();
        self.backlight_low.set_high();
    }

    pub fn backlight_high(&mut self) {
        self.backlight_high.set_low();
        self.backlight_mid.set_high();
        self.backlight_low.set_high();
    }
}

impl OriginDimensions for WatchDisplay {
    fn size(&self) -> Size {
        self.display.size()
    }
}

impl DrawTarget for WatchDisplay {
    type Color = <ST7789 as mipidsi::models::Model>::ColorFormat;
    type Error = mipidsi::error::Error;

    fn fill_contiguous<I>(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        colors: I,
    ) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.display.fill_contiguous(area, colors)
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        self.display.fill_solid(area, color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.display.clear(color)
    }

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        self.display.draw_iter(pixels)
    }
}

impl WatchButton {
    pub fn is_pressed(&mut self) -> bool {
        self.button_active.set_high();
        let result = self.button_pressed.is_high();
        self.button_active.set_low();

        result
    }
}

impl WatchBattery {
    pub async fn check_battery_voltage(&mut self) -> i32 {
        let mut battery_buf = [0];
        self.battery_voltage.sample(&mut battery_buf).await;
        let analog_value: i32 = battery_buf[0].into();
        analog_value * 2000 / 1241
    }

    pub fn is_charging(&self) -> bool {
        self.charging_input.is_low()
    }
}
