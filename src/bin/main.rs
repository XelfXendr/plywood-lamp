#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use blinky::components::{
    led_controller::LedController,
    server::Server,
    wifi::{connection, net_task},
};

use embassy_executor::Spawner;
use embassy_net::StackResources;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::{
    gpio::{AnyPin, Level, Output, OutputConfig, Pin},
    rng::Rng,
    timer::{systimer::SystemTimer, timg::TimerGroup},
};
use esp_println::println;
use static_cell::make_static;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[embassy_executor::task]
async fn _run(led_pin: AnyPin) {
    let mut led = Output::new(led_pin, Level::Low, OutputConfig::default());
    led.set_high();
    loop {
        Timer::after(Duration::from_secs(1)).await;
        led.toggle();
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
        make_static!(esp_wifi::init(timer1.timer0, rng, peripherals.RADIO_CLK,).unwrap());

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

    spawner
        .spawn(connection(wifi_controller, SSID, PASSWORD))
        .ok();
    spawner.spawn(net_task(runner)).ok();

    println!("Init done!, {} {}", SSID, PASSWORD);

    /*
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
    */

    let strip_pin = peripherals.GPIO3.degrade();
    let mut controller = LedController::new(strip_pin, peripherals.RMT).unwrap();
    controller.set_strip(true).await.unwrap();
    
    println!("Starting server...");
    let mut server = Server::<4096>::new(stack, controller);

    server.run().await;
}
