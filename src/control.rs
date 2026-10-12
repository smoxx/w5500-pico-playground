use embassy_futures::select::{Either, select};
use embassy_rp::pwm::{Config, Pwm};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};

use crate::button::wait_for_press;
use embassy_rp::{Peri, peripherals::BOOTSEL};

#[derive(Clone, Copy)]
pub(crate) enum Mode {
    Heartbeat,
    Slow,
    Breathe,
    On,
    Off,
}

impl Mode {
    fn next(self) -> Self {
        match self {
            Mode::Heartbeat => Mode::Breathe,
            Mode::Breathe => Mode::Slow,
            Mode::Slow => Mode::On,
            Mode::On => Mode::Off,
            Mode::Off => Mode::Heartbeat,
        }
    }
}

pub(crate) enum Command {
    Next,
    Set(Mode),
    Bootloader,
}

pub(crate) static COMMANDS: Channel<CriticalSectionRawMutex, Command, 8> = Channel::new();

pub fn pwm_config() -> Config {
    let mut config = Config::default();
    config.top = 999;
    config.divider = 125u8.into();
    config.compare_b;

    config
}

fn set_brightness(pwm: &mut Pwm<'_>, config: &mut Config, level: u16) {
    config.compare_b = level.min(100) * 10;
    pwm.set_config(config);
}

async fn pulse(pwm: &mut Pwm<'_>, config: &mut Config, duration_ms: u64) {
    set_brightness(pwm, config, 100);
    Timer::after_millis(duration_ms).await;
    set_brightness(pwm, config, 0);
}

async fn breathe(pwm: &mut Pwm<'_>, config: &mut Config) {
    for level in 0..=100u16 {
        set_brightness(pwm, config, level);
        Timer::after_millis(10).await;
    }
    Timer::after_millis(250).await;

    for level in (0..=100u16).rev() {
        set_brightness(pwm, config, level);
        Timer::after_millis(10).await;
    }
    Timer::after_millis(500).await;
}

async fn run_led(pwm: &mut Pwm<'_>, config: &mut Config, mode: Mode) {
    set_brightness(pwm, config, 0);
    loop {
        match mode {
            Mode::Heartbeat => {
                pulse(pwm, config, 100).await;
                Timer::after_millis(100).await;
                pulse(pwm, config, 100).await;
                Timer::after_millis(700).await;
            }
            Mode::Breathe => breathe(pwm, config).await,
            Mode::Slow => {
                pulse(pwm, config, 500).await;
                Timer::after_millis(500).await;
            }
            Mode::On => {
                set_brightness(pwm, config, 100);
                core::future::pending::<()>().await;
            }
            Mode::Off => {
                core::future::pending::<()>().await;
            }
        }
    }
}

#[embassy_executor::task]
pub(crate) async fn led_task(mut pwm: Pwm<'static>) {
    let mut mode = Mode::Heartbeat;
    let mut config = pwm_config();

    loop {
        match select(run_led(&mut pwm, &mut config, mode), COMMANDS.receive()).await {
            Either::First(()) => unreachable!(),
            Either::Second(Command::Next) => mode = mode.next(),
            Either::Second(Command::Set(set_mode)) => mode = set_mode,
            Either::Second(Command::Bootloader) => {
                config.compare_b = 0;
                pwm.set_config(&config);
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
