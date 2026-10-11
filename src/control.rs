use embassy_futures::select::{Either, select};
use embassy_rp::gpio::Output;
use embassy_time::{Duration, Timer};

use crate::{COMMANDS, Command, button::wait_for_press};
use embassy_rp::{Peri, peripherals::BOOTSEL};

#[derive(Clone, Copy)]
pub(crate) enum Mode {
    Heartbeat,
    Slow,
    On,
    Off,
}

impl Mode {
    fn next(self) -> Self {
        match self {
            Mode::Heartbeat => Mode::Slow,
            Mode::Slow => Mode::On,
            Mode::On => Mode::Off,
            Mode::Off => Mode::Heartbeat,
        }
    }
}

#[embassy_executor::task]
pub(crate) async fn led_task(mut led: Output<'static>) {
    let mut mode = Mode::Heartbeat;

    loop {
        match select(run_led(&mut led, mode), COMMANDS.receive()).await {
            Either::First(()) => unreachable!(),

            Either::Second(Command::Next) => mode = mode.next(),

            Either::Second(Command::Bootloader) => {
                led.set_low();
                embassy_rp::rom_data::reset_to_usb_boot(0, 0);
                core::future::pending::<()>().await;
            }
        }
    }
}

#[embassy_executor::task]
pub(crate) async fn button_task(mut bootsel: Peri<'static, BOOTSEL>) {
    loop {
        let held = wait_for_press(&mut bootsel).await;

        let command = if held >= Duration::from_secs(2) {
            Command::Bootloader
        } else {
            Command::Next
        };

        COMMANDS.send(command).await;
    }
}

async fn run_led(led: &mut Output<'_>, mode: Mode) {
    led.set_low();

    match mode {
        Mode::Heartbeat => beat(led, 100).await,
        Mode::Slow => beat(led, 500).await,
        Mode::On => {
            led.set_high();
            Timer::after_millis(100).await;
        }
        Mode::Off => {
            led.set_low();
            Timer::after_millis(100).await;
        }
    }
}

// async fn pulse(led: &mut Output<'_>, duration_ms: u64) {
//     led.set_low();
//     Timer::after_millis(duration_ms).await;
//
//     for i in 0..100 {
//         led.set_drive_strength(Drive);
//     }
// }

async fn beat(led: &mut Output<'_>, duration_ms: u64) {
    led.set_low();
    Timer::after_millis(700).await;

    led.set_high();
    Timer::after_millis(duration_ms).await;

    led.set_low();
    Timer::after_millis(duration_ms).await;

    led.set_high();
    Timer::after_millis(duration_ms).await;
}
