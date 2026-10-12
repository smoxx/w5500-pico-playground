#![no_std]
#![no_main]

mod button;
mod control;
mod network;
mod udp;

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

    let stack = network::start(
        spawner,
        network::Pins {
            spi: p.SPI0,
            miso: p.PIN_16,
            cs: p.PIN_17,
            clk: p.PIN_18,
            mosi: p.PIN_19,
            reset: p.PIN_20,
            int: p.PIN_21,
            tx_dma: p.DMA_CH0,
            rx_dma: p.DMA_CH1,
        },
    )
    .await;

    spawner.spawn(udp::udp_task(stack).unwrap());

    core::future::pending::<()>().await;
}
