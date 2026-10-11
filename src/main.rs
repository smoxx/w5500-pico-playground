#![no_std]
#![no_main]

mod button;
mod led;

use button::wait_for_press;
use led::{Mode, run_led};

use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_rp::gpio::{Level, Output};
use embassy_time::Duration;

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
