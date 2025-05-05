use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{AnyPin, Level},
    peripheral::Peripheral,
    peripherals::RMT,
    rmt::{Channel, Error, PulseCode, Rmt, TxChannelAsync, TxChannelConfig, TxChannelCreatorAsync},
    time::Rate,
    Async,
};

pub struct LedController {
    channel: Channel<Async, 0>,
}

impl LedController {
    pub fn new(pin: AnyPin, peripheral: impl Peripheral<P = RMT>) -> Result<Self, Error> {
        let freq = Rate::from_mhz(80);
        let rmt = Rmt::new(peripheral, freq)?;

        let rmt = rmt.into_async();
        let tx_config = TxChannelConfig::default().with_clk_divider(1);

        let channel = rmt.channel0.configure(pin, tx_config)?;

        Ok(Self { channel })
    }

    pub async fn set_strip(&mut self, value: bool) -> Result<(), Error> {
        let color = if value { [0, 200, 200] } else { [0, 0, 0] };

        let codes = [
            PulseCode::new(Level::High, 24, Level::Low, 72),
            PulseCode::new(Level::High, 72, Level::Low, 24),
        ];

        let mut data = [codes[0]; 25];
        for c in 0..3 {
            let byte = color[c];
            for i in (0..8).rev() {
                let bit = (byte >> i) & 1;
                data[c * 8 + i] = codes[bit];
            }
        }
        data[24] = PulseCode::empty();

        for _ in 0..100 {
            self.channel.transmit(&data).await?;
        }
        Timer::after(Duration::from_millis(1)).await;
        Ok(())
    }
}
