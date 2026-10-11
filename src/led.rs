use embassy_rp::gpio::Output;
use embassy_time::Timer;

#[derive(Clone, Copy)]
pub(crate) enum Mode {
    Heartbeat,
    Slow,
    On,
}

pub(crate) async fn run_led(led: &mut Output<'_>, mode: Mode) {
    led.set_low();

    match mode {
        Mode::Heartbeat => beat(led, 100).await,
        Mode::Slow => beat(led, 500).await,
        Mode::On => {
            led.set_high();
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
