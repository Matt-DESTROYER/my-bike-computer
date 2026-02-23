#![no_std]
#![no_main]
#![deny(
	clippy::mem_forget,
	reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
	holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use bt_hci::controller::ExternalController;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
	clock::CpuClock,
	gpio::{
		Level,
		Output,
		OutputConfig
	},
	time::Rate,
	timer::timg::TimerGroup,
	spi::{
		master::Spi,
		Mode
	},
	uart::{
		Config,
		Uart
	}
};
use esp_radio::ble::controller::BleConnector;
use trouble_host::prelude::*;
use log::info;

//use sharp_memory_display::SharpMemoryDisplay;
use embedded_graphics::{
	mono_font::{
		ascii::FONT_10X20,
		MonoTextStyle
	},
	pixelcolor::BinaryColor,
	prelude::*,
	text::Text
};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	esp_println::println!("PANIC! {:?}", info);
	loop {}
}

extern crate alloc;

mod nmea;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
	clippy::large_stack_frames,
	reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
	esp_println::logger::init_logger_from_env();
	info!("Board is alive!");

	let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
	let peripherals = esp_hal::init(config);

	esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
	// COEX needs more RAM - so we've added some more
	esp_alloc::heap_allocator!(size: 64 * 1024);

	let timg0 = TimerGroup::new(peripherals.TIMG0);
	esp_rtos::start(timg0.timer0);

	let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
	let (mut _wifi_controller, _interfaces) =
		esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
			.expect("Failed to initialize Wi-Fi controller");
	// find more examples https://github.com/embassy-rs/trouble/tree/main/examples/esp32
	let transport = BleConnector::new(&radio_init, peripherals.BT, Default::default()).unwrap();
	let ble_controller = ExternalController::<_, 1>::new(transport);
	let mut resources: HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
		HostResources::new();
	let _stack = trouble_host::new(ble_controller, &mut resources);

	let _ = spawner;

	info!("Configuring Adafruit 4.2\" RLCD display on SPI2...");

	let cs   = peripherals.GPIO10;
	let mosi = peripherals.GPIO11;
	let sclk = peripherals.GPIO12;
	let busy = peripherals.GPIO13;
	let dc   = peripherals.GPIO14;
	let rst  = peripherals.GPIO15;

	let spi_config = esp_hal::spi::master::Config::default()
		.with_frequency(Rate::from_mhz(1))
		.with_mode(Mode::_0);

	/*
	let spi = esp_hal::spi::master::Spi::new(peripherals.SPI2, spi_config)
		.unwrap()
		.with_sck(sclk)
		.with_mosi(mosi);*/

	let cs_output = Output::new(cs, Level::Low, OutputConfig::default());
	//let mut display = SharpMemoryDisplay::new(spi, cs_output, 400, 300);

	//display.enable();
	//display.clear(BinaryColor::Off);
	//display.flush().unwrap();

	//info!("Display initialised and cleared!");

	info!("Configuring BN-880 GPS on UART1...");

	let uart_config = Config::default().with_baudrate(9600);
	let mut gps_uart = Uart::new(peripherals.UART1, uart_config)
		.unwrap()
		.with_tx(peripherals.GPIO17)
		.with_rx(peripherals.GPIO18)
		.into_async();

	let mut parser = nmea::Parser::new();
	info!("GPS Parser Read. Entering async event loop...");

	let mut byte_buff: [u8; 1] = [0];

	loop {
		match gps_uart.read_async(&mut byte_buff).await {
			Ok(bytes_read) => {
				if bytes_read < 1 {
					continue;
				}

				parser.parse_byte(byte_buff[0]);

				if parser.finished {
					if parser.valid_checksum {
						if let Some(ref result) = parser.result {
							match result {
								nmea::ParserResult::GGA(gga) => {
									info!(
										"GPS Fix: {:?} | Sats: {} | Lat: {:.5} | Long: {:.5} | Alt: {}m",
										gga.quality, gga.numSV, gga.lat, gga.long, gga.alt
									);
								},
								nmea::ParserResult::RMC(rmc) => {
									info!(
										"Time: {:?} | Date: {:?} | Latitude: {:.5} | Longitude: {:.5} | Speed: {:.5}",
										rmc.time, rmc.date, rmc.lat, rmc.long, rmc.spd
									);
								},
								_ => {}
							}
						}
					} else {
						info!("WARNING: Corrupted NMEA sentence dropped!");
					}
				}
			},
			Err(_) => Timer::after(Duration::from_millis(5)).await
		}
	}
}
