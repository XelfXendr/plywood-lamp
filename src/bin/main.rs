#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::net::Ipv4Addr;
use core::str::FromStr;

use alloc::string::ToString;
use embassy_executor::Spawner;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Runner, StackResources};
use embassy_time::{Duration, Timer};
use embedded_io_async::{Read, Write};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::DriveMode;
use esp_hal::rmt::{
    Channel, PulseCode, Rmt, TxChannel, TxChannelAsync, TxChannelConfig, TxChannelCreator,
    TxChannelCreatorAsync,
};
use esp_hal::time::Rate;
use esp_hal::{
    gpio::{AnyPin, Level, Output, OutputConfig, Pin},
    rng::Rng,
    timer::{systimer::SystemTimer, timg::TimerGroup},
};
use esp_hal::{Async, Blocking};
use esp_println::println;
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState,
};
use httparse;
use itoa;
use static_cell::make_static;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[embassy_executor::task]
async fn run(led_pin: AnyPin) {
    let mut led = Output::new(led_pin, Level::Low, OutputConfig::default());
    led.set_high();
    loop {
        Timer::after(Duration::from_secs(1)).await;
        led.toggle();
    }
}

async fn set_strip(channel: &mut Channel<Async, 0>, on: bool) {
    let color = if on { [0, 200, 200] } else { [0, 0, 0] };
    
    let mut data: [u32; 25] = [PulseCode::new(Level::High, 24, Level::Low, 72); 25];
    for c in 0..3 {
        let byte = color[c];
        for i in (0..8).rev() {
            let bit = (byte >> i) & 1;
            if bit == 0 {
                data[c * 8 + i] = PulseCode::new(Level::High, 24, Level::Low, 72);
            } else {
                data[c * 8 + i] = PulseCode::new(Level::High, 72, Level::Low, 24);
            }
        }
    }
    data[24] = PulseCode::empty();

    for _ in 0..100 {
        let transmission_result = channel.transmit(&data).await;
    }
    Timer::after(Duration::from_millis(1)).await;
}

#[embassy_executor::task]
async fn control_strip(rmt: Rmt<'static, Blocking>, pin: AnyPin) {
    //let mut strip = Output::new(pin, Level::Low, OutputConfig::default());
    let rmt = rmt.into_async();
    let tx_config = TxChannelConfig::default().with_clk_divider(1);

    let mut channel = rmt.channel0.configure(pin, tx_config).unwrap();
    loop {
        println!("Trynna send color");
        let color = [32, 32, 64];

        let mut data: [u32; 25] = [PulseCode::new(Level::High, 24, Level::Low, 72); 25];

        for c in 0..3 {
            let byte = color[c];
            for i in (0..8).rev() {
                let bit = (byte >> i) & 1;
                if bit == 0 {
                    data[c * 8 + i] = PulseCode::new(Level::High, 24, Level::Low, 72);
                } else {
                    data[c * 8 + i] = PulseCode::new(Level::High, 72, Level::Low, 24);
                }
            }
        }
        data[24] = PulseCode::empty();

        println!("Transmitting");
        for _ in 0..100 {
            let transmission_result = channel.transmit(&data).await;
        }
        /*
        if let Err(e) = transmission_result {
            println!("Error transmitting: {:?}", e);
        } else {
            println!("Transmitted successfully");
        }*/

        Timer::after(Duration::from_millis(1000)).await;
    }
}

async fn send_byte<'a>(byte: u8, out: &mut Output<'a>) {
    const TIMINGS: [u64; 2] = [350, 900];
    let byte = byte as usize;
    for i in (0..8).rev() {
        let bit = (byte >> i) & 1;
        if bit == 0 {
            out.set_high();
            Timer::after_nanos(TIMINGS[0]).await;
            out.set_low();
            Timer::after_nanos(TIMINGS[1]).await;
        } else {
            out.set_high();
            Timer::after_nanos(TIMINGS[1]).await;
            out.set_low();
            Timer::after_nanos(TIMINGS[0]).await;
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.3.1
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
    let mut rng = Rng::new(peripherals.RNG);

    let timer1 = TimerGroup::new(peripherals.TIMG0);

    //WiFi setup
    let esp_wifi_controller =
        make_static!(esp_wifi::init(timer1.timer0, rng.clone(), peripherals.RADIO_CLK,).unwrap());

    let (wifi_controller, wifi_interfaces) =
        esp_wifi::wifi::new(esp_wifi_controller, peripherals.WIFI).unwrap();

    let wifi_interface = wifi_interfaces.sta;

    let net_config = embassy_net::Config::dhcpv4(Default::default());
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        make_static!(StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(connection(wifi_controller)).ok();
    spawner.spawn(net_task(runner)).ok();
    // connect wifi to Home1, psk: 1234
    //esp_wifi::wifi::

    println!("Init done!, {} {}", SSID, PASSWORD);

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    let strip_pin = peripherals.GPIO7.degrade();

    let freq = Rate::from_mhz(80);
    let rmt = Rmt::new(peripherals.RMT, freq).unwrap();

    let rmt = rmt.into_async();
    let tx_config = TxChannelConfig::default().with_clk_divider(1);

    let mut channel = rmt.channel0.configure(strip_pin, tx_config).unwrap();

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(1)));
        socket.set_keep_alive(None);
        println!("trying to accept");
        let accept_result = socket
            .accept((stack.config_v4().unwrap().address.address(), 8080))
            .await;
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
                                    set_strip(&mut channel, true).await;
                                }
                                "/?led=0" => {
                                    println!("LED OFF");
                                    set_strip(&mut channel, false).await;
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
                            &itoa::Buffer::new().format(contents.len()),
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

                        socket
                            .write_all(response_buf[..pos].as_ref())
                            .await
                            .unwrap();

                        println!("wrote all");
                        socket.flush().await.unwrap();
                        println!("flushed");
                        //socket.abort();
                        //socket.flush().await.unwrap();
                        //break;
                    }
                    Err(e) => {
                        println!("read error: {:?}", e);
                        break;
                    }
                }
            }
        }
    }

    /*
    let remote_endpoint = (Ipv4Addr::new(104, 26, 9, 59), 80);
    println!("connecting...");
    let r = socket.connect(remote_endpoint).await;
    if let Err(e) = r {
        println!("connect error: {:?}", e);
    }
    else{
        println!("connected!");
        let mut buf = [0; 1024];
        use embedded_io_async::Write;
        let r = socket
            .write_all(b"GET / HTTP/1.0\r\nHost: api.myip.com\r\n\r\n")
            .await;
        if let Err(e) = r {
            println!("write error: {:?}", e);
        } else {
            match socket.read(&mut buf).await {
                Ok(0) => {
                    println!("read EOF");
                }
                Ok(n) => println!("Contents:\n{}\nThose were the contents", core::str::from_utf8(&buf[..n]).unwrap()),
                Err(e) => {
                    println!("read error: {:?}", e);
                }
            };
        }
    }
    */

    let led_pin = peripherals.GPIO8.degrade();
    spawner.spawn(run(led_pin)).unwrap();

    let strip_pin = peripherals.GPIO7.degrade();

    let freq = Rate::from_mhz(80);
    let rmt = Rmt::new(peripherals.RMT, freq).unwrap();

    spawner.spawn(control_strip(rmt, strip_pin)).unwrap();
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: heapless::String::<32>::from_str(SSID).unwrap(),
                password: heapless::String::<64>::from_str(PASSWORD).unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Starting wifi");
            controller.start_async().await.unwrap();
            println!("Wifi started!");

            println!("Scan");
            let result = controller.scan_n_async::<10>().await.unwrap();
            for ap in result.0 {
                println!("{:?}", ap);
            }
        }
        println!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
