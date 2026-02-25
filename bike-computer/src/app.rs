use embedded_graphics::{
	mono_font::{
		MonoTextStyle,
		iso_8859_1::FONT_10X20
	},
	text::Text,
	pixelcolor::BinaryColor,
	prelude::*
};

use alloc::format;

use crate::rlcd;

pub struct State {
	pub lat: f64,
	pub long: f64,
	pub speed: f64,

	pub temp: f32
}

pub struct App<'a> {
	pub state: State,

	pub display: rlcd::Display<'a>
}

impl<'a> App<'a> {
	pub fn new(display: rlcd::Display<'a>) -> Self {
		App {
			state: State {
				lat: 0.0,
				long: 0.0,
				speed: 0.0,

				temp: 0.0
			},

			display
		}
	}

	pub async fn init(&mut self) {
		self.display.init().await;
		self.display.clear(BinaryColor::Off).unwrap();
		self.display.flush();
	}

	pub fn update_gps(&mut self, lat: f64, long: f64, speed: f64, temp: f32) {
		self.state.lat = lat;
		self.state.long = long;
		self.state.speed = speed;

		self.state.temp = temp;
	}

	pub fn render(&mut self) {
		self.display.colour_clear(rlcd::BinaryColour::Black);

		let text_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);

		Text::new("My Bike Computer!", Point::new(10, 30), text_style)
			.draw(&mut self.display)
			.unwrap();

		let position_text = format!("Current coordinates: {:.4}, {:.4}", self.state.lat, self.state.long);
		Text::new(&position_text, Point::new(10, 60), text_style)
			.draw(&mut self.display)
			.unwrap();

		let speed_text = format!("Current speed: {:.2}", self.state.speed);
		Text::new(&speed_text, Point::new(10, 90), text_style)
			.draw(&mut self.display)
			.unwrap();

		let temperature_text = format!("Current temp: {:.1}°C", self.state.temp);
		Text::new(&temperature_text, Point::new(10, 120), text_style)
			.draw(&mut self.display)
			.unwrap();

		self.display.flush();
	}
}
