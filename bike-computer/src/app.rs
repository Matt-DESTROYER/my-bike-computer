use display_interface::WriteOnlyDataCommand;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;
use embassy_time::Delay;
use st7305::St7305;
use embedded_graphics::{
	mono_font::{
		MonoTextStyle,
		iso_8859_1::FONT_7X13,
		iso_8859_1::FONT_8X13,
		iso_8859_1::FONT_10X20
	},
	text::{
		Alignment,
		Text
	},
	pixelcolor::BinaryColor,
	primitives::{
		Arc,
		Circle,
		Line,
		PrimitiveStyleBuilder
	},
	prelude::*
};
use micromath::F32Ext;

use alloc::format;

use crate::nmea::Time;

const WIDTH: i32 = 400;
const HEIGHT: i32 = 300;
const HALF_WIDTH: i32 = WIDTH / 2;
const HALF_HEIGHT: i32 = HEIGHT / 2;

const MAX_SPEED: f32 = 100.0;

pub struct State {
	pub battery_percentage: f32,

	pub lat: f32,
	pub long: f32,
	pub speed: f32,

	pub temp: f32,

	pub time: Time
}

pub struct App<DI, RST> {
	pub state: State,

	pub display: St7305<DI, RST>
}

impl<DI, RST, PinError> App<DI, RST>
where
	DI: WriteOnlyDataCommand,
	RST: OutputPin<Error = PinError>,
	PinError: core::fmt::Debug
{
	pub fn new(display: St7305<DI, RST>) -> Self {
		App {
			state: State {
				battery_percentage: 0.0,

				lat: 0.0,
				long: 0.0,
				speed: 0.0,

				temp: 0.0,

				time: Time {
					hour: 0,
					minute: 0,
					second: 0.0
				}
			},

			display
		}
	}

	pub async fn init(&mut self) {
		let mut delay = Delay;

		delay.delay_ms(500).await;
		self.display.init_async(&mut delay).await.unwrap();

		self.display.color_clear(st7305::BinaryColor::Off as u8);
		self.display.flush().unwrap();
	}

	pub fn update_battery(&mut self, battery_percentage: f32) {
		self.state.battery_percentage = battery_percentage;
	}

	pub fn update_state(&mut self, lat: f32, long: f32, speed: f32, temp: f32, time: Time) {
		self.state.lat = lat;
		self.state.long = long;
		self.state.speed = speed;

		self.state.temp = temp;

		self.state.time = time;
	}

	pub fn render(&mut self) {
		self.display.color_clear(st7305::BinaryColor::Off as u8);

		// styles
		let large_text_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
		let medium_text_style = MonoTextStyle::new(&FONT_8X13, BinaryColor::On);
		let small_text_style = MonoTextStyle::new(&FONT_7X13, BinaryColor::On);

		let thin_stroke = PrimitiveStyleBuilder::new()
			.stroke_color(BinaryColor::On)
			.stroke_width(1)
			.build();
		let medium_stroke = PrimitiveStyleBuilder::new()
			.stroke_color(BinaryColor::On)
			.stroke_width(2)
			.build();
		let thick_stroke = PrimitiveStyleBuilder::new()
			.stroke_color(BinaryColor::On)
			.stroke_width(4)
			.build();

		let fill = PrimitiveStyleBuilder::new()
			.fill_color(BinaryColor::On)
			.build();

		// speedometer
		let speedo_x = HALF_WIDTH;
		let speed_y = HALF_HEIGHT + 10;
		let diameter: u32 = 250;
		let radius = diameter / 2;

		Circle::with_center(Point::new(speedo_x, speed_y), diameter)
			.into_styled(thick_stroke)
			.draw(&mut self.display)
			.unwrap();

		Arc::with_center(Point::new(speedo_x, speed_y), diameter - 10, 135.0.deg(), 270.0.deg())
			.into_styled(thin_stroke)
			.draw(&mut self.display)
			.unwrap();

		let angle = lerp(self.state.speed / MAX_SPEED, 0.0, 260.0) - 220.0;

		let x1: i32 = speedo_x  + (0.300 * angle.to_radians().cos() * radius as f32) as i32;
		let y1: i32 = speed_y + (0.300 * angle.to_radians().sin() * radius as f32) as i32;
		let x2: i32 = speedo_x  + (0.825 * angle.to_radians().cos() * radius as f32) as i32;
		let y2: i32 = speed_y + (0.825 * angle.to_radians().sin() * radius as f32) as i32;
		Line::new(Point::new(x1, y1), Point::new(x2, y2))
			.into_styled(thick_stroke)
			.draw(&mut self.display)
			.unwrap();

		for i in 0..=10 {
			let speed = i * 10;

			let angle = lerp(speed as f32 / MAX_SPEED, 0.0, 260.0) - 220.0;

			let x1: i32 = speedo_x  + (0.9 * angle.to_radians().cos() * radius as f32) as i32;
			let y1: i32 = speed_y + (0.9 * angle.to_radians().sin() * radius as f32) as i32;
			let x2: i32 = speedo_x  + (1.0 * angle.to_radians().cos() * radius as f32) as i32;
			let y2: i32 = speed_y + (1.0 * angle.to_radians().sin() * radius as f32) as i32;

			Line::new(Point::new(x1, y1), Point::new(x2, y2))
				.into_styled(thick_stroke)
				.draw(&mut self.display)
				.unwrap();

			let x: i32 = speedo_x  + (1.125 * angle.to_radians().cos() * radius as f32) as i32;
			let y: i32 = speed_y + (1.125 * angle.to_radians().sin() * radius as f32) as i32;
			let marking_text = format!("{}", speed);
			Text::with_alignment(&marking_text, Point::new(x, y), medium_text_style, Alignment::Center)
				.draw(&mut self.display)
				.unwrap();
		}

		for i in 0..10 {
			let speed = i * 10 + 5;

			let angle = lerp(speed as f32 / MAX_SPEED, 0.0, 260.0) - 220.0;

			let x1: i32 = speedo_x  + (0.91 * angle.to_radians().cos() * radius as f32) as i32;
			let y1: i32 = speed_y + (0.91 * angle.to_radians().sin() * radius as f32) as i32;
			let x2: i32 = speedo_x  + (1.0 * angle.to_radians().cos() * radius as f32) as i32;
			let y2: i32 = speed_y + (1.0 * angle.to_radians().sin() * radius as f32) as i32;

			Line::new(Point::new(x1, y1), Point::new(x2, y2))
				.into_styled(medium_stroke)
				.draw(&mut self.display)
				.unwrap();

			let x: i32 = speedo_x  + (1.12 * angle.to_radians().cos() * radius as f32) as i32;
			let y: i32 = speed_y + (1.12 * angle.to_radians().sin() * radius as f32) as i32;
			let marking_text = format!("{}", speed);
			Text::with_alignment(&marking_text, Point::new(x, y), small_text_style, Alignment::Center)
				.draw(&mut self.display)
				.unwrap();
		}

		let speed_text = format!("{:.1} km", self.state.speed);
		Text::with_alignment(&speed_text, Point::new(speedo_x, speed_y), large_text_style, Alignment::Center)
			.draw(&mut self.display)
			.unwrap();

		// time
		let time_text = format!("{:02}:{:02}:{:02}", self.state.time.hour, self.state.time.minute, self.state.time.second as u8);
		Text::new(&time_text, Point::new(5, 20), large_text_style)
			.draw(&mut self.display)
			.unwrap();

		// temperature
		let thermometer_x = WIDTH - 20;
		let thermometer_y = 5;
		Line::new(Point::new(thermometer_x, thermometer_y), Point::new(thermometer_x, thermometer_y + 12))
			.into_styled(thick_stroke)
			.draw(&mut self.display)
			.unwrap();

		Circle::with_center(Point::new(thermometer_x, thermometer_y + 14), 8)
			.into_styled(fill)
			.draw(&mut self.display)
			.unwrap();

		for i in 0..3 {
			let tick_y = thermometer_y + 2 + (i * 4);
			Line::new(Point::new(thermometer_x + 3, tick_y), Point::new(thermometer_x + 5, tick_y))
				.into_styled(thin_stroke)
				.draw(&mut self.display)
				.unwrap();
		}

		let temp_text = format!("{:.1}°C", self.state.temp);
		Text::with_alignment(&temp_text, Point::new(WIDTH - 30, 20), large_text_style, Alignment::Right)
			.draw(&mut self.display)
			.unwrap();

		// battery percentage
		let battery_text = format!("{:02}%", self.state.battery_percentage);
		Text::new(&battery_text, Point::new(5, HEIGHT - 5), large_text_style)
			.draw(&mut self.display)
			.unwrap();

		self.display.flush().unwrap();
	}
}

fn lerp(val: f32, min: f32, max: f32) -> f32 {
	val * (max - min) + min
}
