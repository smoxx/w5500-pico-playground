use embassy_rp::{Peri, bootsel::is_bootsel_pressed, peripherals::BOOTSEL};
use embassy_time::{Duration, Instant, Timer};

pub(crate) async fn wait_for_press(bootsel: &mut Peri<'_, BOOTSEL>) -> Duration {
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
