use embassy_net::{tcp::TcpSocket, Stack};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use esp_println::println;

use crate::components::{
    leds::runner::LedSignal,
    server::request::LedRequest,
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

    fn build_response(buffer: &mut [u8]) -> &[u8] {
        fn add_to_buf(buf: &mut [u8], pos: usize, text: &str) -> usize {
            let text = text.as_bytes();
            let len = text.len();
            buf[pos..pos + len].copy_from_slice(text);
            pos + len
        }

        let mut pos = 0;

        let status_line = "HTTP/1.1 200 OK";
        let contents = "{\"led\": \"on\"}";

        pos = add_to_buf(buffer, pos, status_line);
        pos = add_to_buf(buffer, pos, "\r\n");
        pos = add_to_buf(buffer, pos, "Content-Length: ");
        pos = add_to_buf(buffer, pos, itoa::Buffer::new().format(contents.len()));
        pos = add_to_buf(buffer, pos, "\r\n");
        pos = add_to_buf(buffer, pos, "Content-Type: application/json\r\n");
        pos = add_to_buf(buffer, pos, "\r\n");
        pos = add_to_buf(buffer, pos, contents);
        pos = add_to_buf(buffer, pos, "\r\n");

        &buffer[..pos]
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
                            if let Ok(req) = LedRequest::parse_http(&buf[..n]) {
                                self.led_signal.signal(req);

                                let response = Self::build_response(&mut buf);

                                if socket.write_all(response).await.is_ok() {
                                    let _ = socket.flush().await;
                                }
                            } else {
                                socket.abort();
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
