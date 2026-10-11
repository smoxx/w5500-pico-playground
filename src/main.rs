#![no_std]
#![no_main]

mod led;
use led::{Mode, run_led};

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_rp::{
    Peri,
    bootsel::is_bootsel_pressed,
    gpio::{Level, Output},
    peripherals::BOOTSEL,
};
use embassy_time::{Duration, Instant, Timer};

use panic_halt as _;

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let mut led = Output::new(peripherals.PIN_25, Level::Low);
    let mut bootsel = peripherals.BOOTSEL;
    let mut mode = Mode::Heartbeat;

    loop {
        let result = select(run_led(&mut led, mode), wait_for_press(&mut bootsel)).await;

        if let Either::Second(duration) = result {
            if duration >= Duration::from_secs(1) {
                led.set_low();
                embassy_rp::rom_data::reset_to_usb_boot(0, 0);
            } else {
                mode = match mode {
                    Mode::Heartbeat => Mode::Slow,
                    Mode::Slow => Mode::On,
                    Mode::On => Mode::Heartbeat,
                }
            }
        }
    }
}

async fn wait_for_press(bootsel: &mut Peri<'_, BOOTSEL>) -> Duration {
    loop {
        if is_bootsel_pressed(bootsel.reborrow()) {
            Timer::after_millis(30).await;

            if is_bootsel_pressed(bootsel.reborrow()) {
                break;
            }
        }

        Timer::after_millis(20).await;
    }

    let started = Instant::now();

    loop {
        if !is_bootsel_pressed(bootsel.reborrow()) {
            Timer::after_millis(30).await;

            if !is_bootsel_pressed(bootsel.reborrow()) {
                break;
            }
        }

        Timer::after_millis(20).await
    }
    started.elapsed()
}
