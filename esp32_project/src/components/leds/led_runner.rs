use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};

use crate::components::leds::led_effects::{Color, Effect, EffectStatus, Effects};

use super::led_controller::LedController;


pub type LedSignal = Signal<CriticalSectionRawMutex, LedCommand>;

pub enum LedCommand {
    MoveTo(u8, u8, u8, u64),
}

#[embassy_executor::task]
pub async fn run_leds(mut controller: LedController, led_signal: &'static LedSignal) {
    let mut current_effect: Effects = Effects::move_to(Color::new(0,0,0), Color::new(255,244,200), 1000);
    let mut current_status: EffectStatus;

    loop {
        // update LEDs according to effect
        current_status = current_effect.run();
        controller.send_color(current_status.color()).await.ok();

        // wait either for new command or for a delay till next LED update 
        let signal = match current_status {
            EffectStatus::InProgress(_, millis) => {
                select(led_signal.wait(), Timer::after(Duration::from_millis(millis))).await
            },
            EffectStatus::Finished(_) => {
                Either::First(led_signal.wait().await)
            },
        };

        // if we got command then accept new effect
        if let Either::First(command) = signal {
            match command {
                LedCommand::MoveTo(r, g, b, millis) => {
                    current_effect = Effects::move_to(
                        current_status.color(),
                        Color::new(r, g, b),
                        millis,
                    );
                }
            }
        }
    }
}
