#![no_std]
#![no_main]

use embedded_graphics::{
	mono_font::{
		ascii::FONT_10X20,
		MonoTextStyle
	},
	pixelcolor::BinaryColor,
	text::Text,
	prelude::*
};
use embedded_hal::{
	spi::SpiDevice,
	digital::OutputPin
};

use alloc::format;

use crate::sd::SD;

mod rlcd;
mod sd;

pub struct State {
	pub lat: f64,
	pub long: f64,
	pub speed: f64
}

pub struct App<'a, SPI, RLCD_CS, RLCD_DC, RLCD_RST, SDMMC, SD_CLK, SD_CMD, SD_DATA> {
	display: rlcd::Display<'a>,
	sd: sd::SD<'a, SDMMC, SD_CLK, SD_CMD, SD_DATA>,
	pub state: State
}

impl<'a> App<'a> {
	pub fn init(display: rlcd::Display<'a>, sd: sd::SD<'a>) -> Self {
		display.init().await;
		display.clear(BinaryColor::Off).unwrap();
		display.flush();

		Self {
			display,
			sd,
			state: State {
				lat: 0.0,
				long: 0.0,
				speed: 0.0
			}
		}
	}

	pub fn update_gps(&mut self, lat: f64, long: f64, speed: f64) {
		self.state.lat = lat;
		self.state.long = long;
		self.state.speed = speed;
	}
	
	pub fn render(&mut self) {
		self.display.ColourClear(rlcd::BinaryColour::Black);
		
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
		
		self.display.flush();
	}
}
