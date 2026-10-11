#![no_std]
#![no_main]

mod button;
mod control;

use control::{button_task, led_task};

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};

use panic_halt as _;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

enum Command {
    Next,
    Bootloader,
}

static COMMANDS: Channel<CriticalSectionRawMutex, Command, 8> = Channel::new();

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
async fn main(spawner: Spawner) {
    let peripherals = embassy_rp::init(Default::default());

    let led = Output::new(peripherals.PIN_25, Level::Low);

    spawner.spawn(led_task(led).unwrap());
    spawner.spawn(button_task(peripherals.BOOTSEL).unwrap());

    core::future::pending::<()>().await;
}
