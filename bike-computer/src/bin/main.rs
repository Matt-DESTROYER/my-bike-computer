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
use embassy_time::{Delay, Duration, Ticker, Timer};
use embassy_sync::{
	blocking_mutex::raw::CriticalSectionRawMutex,
	watch::Watch
};
use esp_hal::{
	Blocking, Config, clock::CpuClock, gpio::{
		Level,
		Output,
		OutputConfig
	},
	i2c::master::{
		Config as I2cConfig,
		I2c
	},
	spi::Mode, time::Rate, timer::timg::TimerGroup, uart::{
		Config as UartConfig,
		Uart
	}
};
use shtcx::{
	PowerMode,
	ShtC3,
	shtc3
};
use esp_radio::{ble::controller::BleConnector, /*wifi::event::StaWpsErTimeout*/};
//use smoltcp::socket;
use trouble_host::prelude::*;
use log::info;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	esp_println::println!("PANIC! {:?}", info);
	loop {}
}

use bike_computer::nmea;
use bike_computer::rlcd;
use bike_computer::app::App;

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

	let config = Config::default().with_cpu_clock(CpuClock::max());
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

	// I2C
	let i2c_sda = peripherals.GPIO13;
	let i2c_scl = peripherals.GPIO14;

	// TF Card
	//let sdmmc_cmd  = peripherals.GPIO21;
	//let sdmmc_clk  = peripherals.GPIO38;
	//let sdmmc_data = peripherals.GPIO39;
	//let sd_cd      = peripherals.GPIO17;

	// PCF85063
	//let rtc_int = peripherals.GPIO15;

	// ES8311 + ES7210
	//let i2s_dsdin  = peripherals.GPIO8;
	//let i2s_sclk   = peripherals.GPIO9;
	//let i2s_asout  = peripherals.GPIO10;
	//let i2s_mclk   = peripherals.GPIO16;
	//let i2s_lrck   = peripherals.GPIO45;
	//let pa_ctrl    = peripherals.GPIO46;

	// RLCD
	let rlcd_ds     = peripherals.GPIO5;
	//let rlcd_te     = peripherals.GPIO6;
	let rlcd_sclk  = peripherals.GPIO11;
	let rlcd_din   = peripherals.GPIO12;
	let rlcd_cs    = peripherals.GPIO40;
	let rlcd_rst   = peripherals.GPIO41;

	// configure RLCD display
	let cs_output  = Output::new(rlcd_cs, Level::High, OutputConfig::default());
	let dc_output  = Output::new(rlcd_ds, Level::Low, OutputConfig::default());
	let rst_output = Output::new(rlcd_rst, Level::High, OutputConfig::default());

	let rlcd_spi_config = esp_hal::spi::master::Config::default()
		.with_frequency(Rate::from_mhz(10))
		.with_mode(Mode::_0);

	let rlcd_spi = esp_hal::spi::master::Spi::new(peripherals.SPI2, rlcd_spi_config)
		.unwrap()
		.with_sck(rlcd_sclk)
		.with_mosi(rlcd_din);

	let display = rlcd::Display::new(rlcd_spi, cs_output, dc_output, rst_output);

	// configure SHTC3
	let i2c_config = I2cConfig::default()
		.with_frequency(Rate::from_khz(100));
	let i2c_bus = I2c::new(peripherals.I2C0, i2c_config)
		.unwrap()
		.with_sda(i2c_sda)
		.with_scl(i2c_scl);

	let shtc3_sensor = shtc3(i2c_bus);
	spawner.spawn(temperature_worker(shtc3_sensor)).unwrap();
	let mut temp_rx = TEMPERATURE_WATCHER.receiver().unwrap();

	// configure GPS
	let uart_config = UartConfig::default().with_baudrate(9600);
	let mut gps_uart = Uart::new(peripherals.UART1, uart_config)
		.unwrap()
		.with_tx(peripherals.GPIO17)
		.with_rx(peripherals.GPIO18)
		.into_async();

	// initialise app
	let mut app = App::new(display);
	app.init().await;

	// init parser
	let mut parser = nmea::Parser::new();
	let mut byte_buff: [u8; 1] = [0];

	loop {
		// read data from GPS
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
								nmea::ParserResult::RMC(rmc) => {
									let latest_temp = temp_rx.try_get().unwrap_or(0.0);
									app.update_gps(rmc.lat, rmc.long, rmc.spd * 1.852, latest_temp);
									app.render();
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

static TEMPERATURE_WATCHER: Watch<CriticalSectionRawMutex, f32, 2> = Watch::new();

#[embassy_executor::task]
async fn temperature_worker(mut sensor: ShtC3<I2c<'static, Blocking>>) {
	let mut ticker = Ticker::every(Duration::from_secs(30));
	let temp_tx = TEMPERATURE_WATCHER.sender();

	let mut delay = Delay;

	loop {
		match sensor.measure_temperature(PowerMode::NormalMode, &mut delay) {
			Ok(temp) =>
				temp_tx.send(temp.as_degrees_celsius()),
			Err(_) => {}
		}

		ticker.next().await;
	}
}
