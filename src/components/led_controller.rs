use core::{cmp::min, mem};

use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::{
    gpio::{AnyPin, Level},
    peripheral::Peripheral,
    peripherals::RMT,
    rmt::{Channel, Error, PulseCode, Rmt, TxChannel, TxChannelConfig, TxChannelCreator},
    time::Rate,
    Blocking,
};
use esp_println::println;

// High/Low signal times in nanosecs
const T0H: u16 = 300;
const T0L: u16 = 1000;
const T1H: u16 = 1000;
const T1L: u16 = 300;

// Reset signal time in microsecs
const RES: u64 = 300;

pub enum LedCommand {
    MoveTo(u8, u8, u8, u64),
}

pub type LedSignal = Signal<CriticalSectionRawMutex, LedCommand>;

pub struct LedController {
    channel: Option<Channel<Blocking, 0>>,
    num_leds: usize,
    bit_codes: [u32; 2],
}

impl LedController {
    pub fn new(
        pin: AnyPin,
        peripheral: impl Peripheral<P = RMT>,
        num_leds: usize,
    ) -> Result<Self, Error> {
        let freq = Rate::from_mhz(80);
        let rmt = Rmt::new(peripheral, freq)?;

        let tx_config = TxChannelConfig::default().with_clk_divider(1);
        let channel = rmt.channel0.configure(pin, tx_config)?;

        Ok(Self {
            channel: Some(channel),
            num_leds,
            bit_codes: [
                PulseCode::new(Level::High, T0H * 2 / 25, Level::Low, T0L * 2 / 25),
                PulseCode::new(Level::High, T1H * 2 / 25, Level::Low, T1L * 2 / 25),
            ],
        })
    }

    pub async fn send_color(&mut self, rgb: [u8; 3]) -> Result<(), Error> {
        let grb = [rgb[1], rgb[0], rgb[2]];

        // prepare data
        let mut data = [self.bit_codes[0]; 25];
        let mut data_idx = 0;
        for c in 0..3 {
            let byte = grb[c];
            for i in (0..8).rev() {
                let bit = ((byte >> i) & 1) as usize;
                data[data_idx] = self.bit_codes[bit];
                data_idx += 1;
            }
        }
        data[24] = PulseCode::empty();

        // send data
        let mut channel = mem::replace(&mut self.channel, None).ok_or(Error::TransmissionError)?;
        let tx = channel.transmit_continuously_with_loopcount(self.num_leds as u16, &data)?;
        while !tx.is_loopcount_interrupt_set() {
            Timer::after(Duration::from_micros(100)).await;
        }
        channel = match tx.stop_next() {
            Ok(c) => c,
            Err((_e, c)) => c,
        };
        self.channel = Some(channel);

        // wait before we can send new data
        Timer::after(Duration::from_micros(RES)).await;
        Ok(())
    }
}

#[embassy_executor::task]
pub async fn run_leds(mut controller: LedController, led_signal: &'static LedSignal) {
    let mut t0: Instant = Instant::now();
    let mut goal: LedCommand = LedCommand::MoveTo(255, 244, 100, 1000);
    let mut starting_state = [0,0,0];
    let mut current_state = [0,0,0];
    let mut finished = false;
    loop {

        let signal = if finished {
            Either::First(led_signal.wait().await)
        } else {
            select(led_signal.wait(), Timer::after(Duration::from_millis(20))).await
        };

        match signal {
            Either::First(command) => match command {
                LedCommand::MoveTo(r, g, b, millis) => {
                    t0 = Instant::now();
                    goal = command;
                    starting_state = current_state;
                    finished = false;
                }
            },
            Either::Second(_) => {}
        }

        if finished {
            continue;
        }

        match goal {
            LedCommand::MoveTo(r, g, b, millis) => {
                let mut dt = t0.elapsed().as_millis();
                if dt >= millis {
                    finished = true;
                    dt = millis;
                }

                current_state = [
                    (starting_state[0] as i64 + (r as i64 - starting_state[0] as i64) * dt as i64 / millis as i64) as u8,
                    (starting_state[1] as i64 + (g as i64 - starting_state[1] as i64) * dt as i64 / millis as i64) as u8,
                    (starting_state[2] as i64 + (b as i64 - starting_state[2] as i64) * dt as i64 / millis as i64) as u8,
                ];
                controller.send_color(current_state).await.ok();
            },
        }
    }
}
