use core::mem;

use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{AnyPin, Level},
    peripheral::Peripheral,
    peripherals::RMT,
    rmt::{Channel, Error, PulseCode, Rmt, TxChannel, TxChannelConfig, TxChannelCreator},
    time::Rate, 
    Blocking,
};

pub struct LedController {
    channel: Option<Channel<Blocking, 0>>,
}

impl LedController {
    pub fn new(pin: AnyPin, peripheral: impl Peripheral<P = RMT>) -> Result<Self, Error> {
        let freq = Rate::from_mhz(80);
        let rmt = Rmt::new(peripheral, freq)?;

        let tx_config = TxChannelConfig::default().with_clk_divider(1);
        let channel = rmt.channel0.configure(pin, tx_config)?;

        Ok(Self { channel: Some(channel) })
    }

    pub async fn set_strip(&mut self, value: bool) -> Result<(), Error> {

        let color = if value { [244, 255, 100] } else { [0, 0, 0] };

        let codes = [
            PulseCode::new(Level::High, 24, Level::Low, 48),
            PulseCode::new(Level::High, 48, Level::Low, 24),
        ];

        let mut data = [codes[0]; 25];
        let mut data_idx = 0;
        for c in 0..3 {
            let byte = color[c];
            for i in (0..8).rev() {
                let bit = (byte >> i) & 1;
                data[data_idx] = codes[bit];
                data_idx += 1;
            }
        }
        data[24] = PulseCode::empty();

        let mut channel = mem::replace(&mut self.channel, None).ok_or(Error::TransmissionError)?;

        for _ in 0..12 {
            channel = match channel.transmit(&data)?.wait() {
                Ok(c) => c,
                Err((_e, c)) => c,
            }
        };
        self.channel = Some(channel);
        
        Timer::after(Duration::from_millis(1)).await;
        Ok(())
    }
}
