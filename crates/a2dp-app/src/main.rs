//! CYW43 WiFi LED Blink Test for Pico 2 W (no Bluetooth yet)

#![no_std]
#![no_main]

use cyw43::aligned_bytes;
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

// CYW43 runner task - must run continuously
#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("=== CYW43 WiFi LED Blink Test ===");

    let p = embassy_rp::init(Default::default());
    info!("Embassy RP initialized");

    // CYW43 firmware - included at compile time
    let fw = aligned_bytes!("../../../firmware/43439A0.bin");
    let clm = aligned_bytes!("../../../firmware/43439A0_clm.bin");

    info!("Configuring PIO SPI...");
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);

    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        RM2_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );
    info!("PIO SPI configured");

    // Initialize CYW43 state
    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());

    // Initialize CYW43 (WiFi only, no Bluetooth)
    info!("Initializing CYW43...");
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw, clm).await;
    info!("CYW43 initialized!");

    // Spawn the CYW43 runner task
    spawner.spawn(unwrap!(cyw43_task(runner)));
    info!("CYW43 runner task spawned");

    // Initialize WiFi control
    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;
    info!("CYW43 control initialized");

    // Main loop - blink LED
    info!("Starting LED blink loop");
    let delay = Duration::from_millis(250);
    loop {
        info!("LED on!");
        control.gpio_set(0, true).await;
        Timer::after(delay).await;

        info!("LED off!");
        control.gpio_set(0, false).await;
        Timer::after(delay).await;
    }
}
