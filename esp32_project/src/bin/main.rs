#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use blinky::components::{
    leds::{
        controller::LedController,
        runner::{run_leds, LedSignal},
    },
    server::Server,
    wifi::{connection, net_task},
};

use embassy_executor::Spawner;
use embassy_net::StackResources;
use esp_hal::clock::CpuClock;
use esp_hal::{
    gpio::Pin,
    rng::Rng,
    timer::{systimer::SystemTimer, timg::TimerGroup},
};
use esp_println::println;
use static_cell::make_static;

extern crate alloc;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");
const NUM_LEDS: usize = 12;

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

    let led_signal = make_static!(LedSignal::new());

    let strip_pin = peripherals.GPIO3.degrade();
    let controller = LedController::new(strip_pin, peripherals.RMT, NUM_LEDS).unwrap();

    spawner.spawn(run_leds(controller, led_signal)).ok();

    println!("Starting server...");
    let mut server = Server::<4096>::new(stack, led_signal);

    server.run().await;
}
