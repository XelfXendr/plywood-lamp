use embassy_time::{Duration, Timer};
use esp_hal::{
    Blocking,
    gpio::{AnyPin, Level},
    peripheral::Peripheral,
    peripherals::RMT,
    rmt::{Channel, Error, PulseCode, Rmt, TxChannel, TxChannelConfig, TxChannelCreator},
    time::Rate,
};

use crate::types::Color;

// High/Low pulse code signal times in nanosecs
const T0H: u16 = 300;
const T0L: u16 = 1000;
const T1H: u16 = 1000;
const T1L: u16 = 300;

// Reset signal time in microsecs
const RES: u64 = 300;

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

    pub async fn send_color(&mut self, color: Color) -> Result<(), Error> {
        let grb = color.grb();
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
        self.channel = Self::send_through(self.channel.take(), &data, self.num_leds as u16).await?;

        // wait before we can send new data
        Timer::after(Duration::from_micros(RES)).await;
        Ok(())
    }

    async fn send_through<C: TxChannel>(
        channel: Option<C>,
        data: &[u32],
        loopcount: u16,
    ) -> Result<Option<C>, Error> {
        let mut channel = channel.ok_or(Error::TransmissionError)?;
        let tx = channel.transmit_continuously_with_loopcount(loopcount, &data)?;
        while !tx.is_loopcount_interrupt_set() {
            Timer::after(Duration::from_micros(100)).await;
        }
        channel = match tx.stop_next() {
            Ok(c) => c,
            Err((_e, c)) => c,
        };
        Ok(Some(channel))
    }
}
