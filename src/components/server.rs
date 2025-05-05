use embassy_net::{tcp::TcpSocket, Stack};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use esp_println::println;

use super::led_controller::LedController;

pub struct Server<'d, const B: usize> {
    rx_buffer: [u8; B],
    tx_buffer: [u8; B],

    stack: Stack<'d>,
    controller: LedController,
}

impl<'d, const B: usize> Server<'d, B> {
    pub fn new(stack: Stack<'d>, controller: LedController) -> Self {
        Self {
            rx_buffer: [0; B],
            tx_buffer: [0; B],
            stack,
            controller,
        }
    }
    
    pub async fn run(&mut self) {
        loop {
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
            let accept_result = socket.accept((v4.address.address(), 8080)).await;
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
                            println!(
                                "Contents:\n{}\nThose were the contents",
                                core::str::from_utf8(&buf[..n]).unwrap()
                            );

                            let mut headers = [httparse::EMPTY_HEADER; 64];
                            let mut req = httparse::Request::new(&mut headers);
                            let status = req.parse(&buf[..n]).unwrap();
                            println!("status: {:?}", status);

                            println!("path: {:?}", req.path);

                            if let Some(path) = req.path {
                                match path {
                                    "/?led=1" => {
                                        println!("LED ON");
                                        let _ = self.controller.set_strip(true).await;
                                    }
                                    "/?led=0" => {
                                        println!("LED OFF");
                                        let _ = self.controller.set_strip(false).await;
                                    }
                                    _ => {}
                                }
                            }

                            for header in req.headers {
                                println!(
                                    "{}: {:?}",
                                    header.name,
                                    core::str::from_utf8(header.value).unwrap()
                                );
                            }

                            let mut response_buf = [0; 1024];

                            fn add_to_buf(buf: &mut [u8], pos: usize, text: &str) -> usize {
                                let text = text.as_bytes();
                                let len = text.len();
                                buf[pos..pos + len].copy_from_slice(text);
                                pos + len
                            }

                            let mut pos = 0;

                            let status_line = "HTTP/1.1 200 OK";
                            let contents = "{\"led\": \"on\"}";

                            pos = add_to_buf(&mut response_buf, pos, status_line);
                            pos = add_to_buf(&mut response_buf, pos, "\r\n");
                            pos = add_to_buf(&mut response_buf, pos, "Content-Length: ");
                            pos = add_to_buf(
                                &mut response_buf,
                                pos,
                                itoa::Buffer::new().format(contents.len()),
                            );
                            pos = add_to_buf(&mut response_buf, pos, "\r\n");
                            pos = add_to_buf(
                                &mut response_buf,
                                pos,
                                "Content-Type: application/json\r\n",
                            );
                            pos = add_to_buf(&mut response_buf, pos, "\r\n");
                            pos = add_to_buf(&mut response_buf, pos, contents);
                            pos = add_to_buf(&mut response_buf, pos, "\r\n");

                            println!(
                                "Response:\n{}\n",
                                core::str::from_utf8(&response_buf[..pos]).unwrap()
                            );

                            if socket.write_all(response_buf[..pos].as_ref()).await.is_ok() {
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
