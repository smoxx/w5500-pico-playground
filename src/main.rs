#![no_std]
#![no_main]

mod button;
mod control;

use control::{button_task, led_task};

use embassy_executor::Spawner;
use embassy_rp::pwm::{Config, Pwm};

use panic_halt as _;

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let pwm = Pwm::new_output_b(p.PWM_SLICE4, p.PIN_25, Config::default());

    spawner.spawn(led_task(pwm).unwrap());
    spawner.spawn(button_task(p.BOOTSEL).unwrap());

    core::future::pending::<()>().await;
}
