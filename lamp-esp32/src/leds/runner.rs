use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};

use crate::{
    server::LedRequest,
    types::Color
};

use super::{
    controller::LedController,
    effects::{DaylightCycle, EffectEnum, EffectStatus, MoveTo},
};

pub type LedSignal = Signal<CriticalSectionRawMutex, LedRequest>;

#[embassy_executor::task]
pub async fn run_leds(mut controller: LedController, led_signal: &'static LedSignal) {
    let mut current_effect: EffectEnum =
        MoveTo::new(Color::new(0, 0, 0), Color::new(255, 244, 200), 1000).into();

    loop {
        // update LEDs according to effect
        let (current_color, current_status) = current_effect.step();
        let _ = controller.send_color(current_color).await;

        // wait either for new command or for a delay till next LED update
        let signal = match current_status {
            EffectStatus::InProgress(millis) => {
                select(
                    led_signal.wait(),
                    Timer::after(Duration::from_millis(millis)),
                )
                .await
            }
            EffectStatus::Finished => Either::First(led_signal.wait().await),
        };

        // if we got command then accept new effect
        if let Either::First(command) = signal {
            current_effect = match command {
                LedRequest::Set(color, duration) => {
                    MoveTo::new(current_color, color, duration.as_millis()).into()
                }
                LedRequest::DaylightCycle(color, current_time, minutes) => {
                    DaylightCycle::new(current_color, color, current_time, minutes).into()
                }
            }
        }
    }
}
