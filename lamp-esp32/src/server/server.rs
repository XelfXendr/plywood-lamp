use embassy_net::{tcp::TcpSocket, Stack};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use esp_println::println;

use crate::{
    leds::runner::LedSignal,
    server::{request::LedRequest, response_builder::ResponseBuilder},
};

pub struct Server<'d, const B: usize> {
    rx_buffer: [u8; B],
    tx_buffer: [u8; B],

    stack: Stack<'d>,
    led_signal: &'d LedSignal,
}

impl<'d, const B: usize> Server<'d, B> {
    pub fn new(stack: Stack<'d>, led_signal: &'d LedSignal) -> Self {
        Self {
            rx_buffer: [0; B],
            tx_buffer: [0; B],
            stack,
            led_signal,
        }
    }

    pub async fn run(&mut self) {
        loop {
            println!("Creating socket...");
            let mut socket = TcpSocket::new(self.stack, &mut self.rx_buffer, &mut self.tx_buffer);
            socket.set_timeout(Some(embassy_time::Duration::from_secs(1)));
            socket.set_keep_alive(None);
            println!("trying to accept");
            let v4 = loop {
                match self.stack.config_v4() {
                    Some(v4) => break v4,
                    None => {
                        Timer::after(Duration::from_millis(500)).await;
                    }
                }
            };
            println!("Got IP: {}, creating socket", v4.address);

            let accept_result = socket.accept((v4.address.address(), 8308)).await;
            if let Err(e) = accept_result {
                println!("accept error: {:?}", e);
            } else {
                println!("accepted!");
                let mut buf = [0; 1024];
                loop {
                    println!("Reading");
                    match socket.read(&mut buf).await {
                        Ok(0) => {
                            println!("read EOF");
                            break;
                        }
                        Ok(n) => {
                            let parse_result = LedRequest::parse_http(&buf[..n]);

                            let mut response_builder = ResponseBuilder::new(&mut buf);

                            let response = match parse_result {
                                Ok(request) => {
                                    self.led_signal.signal(request);
                                    response_builder.build_response()
                                }
                                Err(error) => response_builder.build_bad_request(error),
                            };

                            if socket.write_all(response).await.is_ok() {
                                let _ = socket.flush().await;
                            }
                        }
                        Err(e) => {
                            println!("read error: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }
    }
}
