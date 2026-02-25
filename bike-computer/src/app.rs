use embedded_graphics::{
	mono_font::{
		MonoTextStyle,
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

use crate::rlcd;

const WIDTH: i32 = 400;
const HEIGHT: i32 = 300;
const HALF_WIDTH: i32 = WIDTH / 2;
const HALF_HEIGHT: i32 = HEIGHT / 2;

const MAX_SPEED: f32 = 100.0;

pub struct State {
	pub lat: f32,
	pub long: f32,
	pub speed: f32,

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

	pub fn update_state(&mut self, lat: f32, long: f32, speed: f32, temp: f32) {
		self.state.lat = lat;
		self.state.long = long;
		self.state.speed = speed;

		self.state.temp = temp;
	}

	pub fn render(&mut self) {
		self.display.colour_clear(rlcd::BinaryColour::Black);

		// styles
		let large_text_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
		let small_text_style = MonoTextStyle::new(&FONT_8X13, BinaryColor::On);

		let thin_stroke = PrimitiveStyleBuilder::new()
			.stroke_color(BinaryColor::On)
			.stroke_width(1)
			.build();

		let thick_stroke = PrimitiveStyleBuilder::new()
			.stroke_color(BinaryColor::On)
			.stroke_width(4)
			.build();

		// speedometer
		let diameter: u32 = 250;
		let radius = diameter / 2;

		Circle::with_center(Point::new(HALF_WIDTH, HALF_HEIGHT), diameter)
			.into_styled(thick_stroke)
			.draw(&mut self.display)
			.unwrap();

		Arc::with_center(Point::new(HALF_WIDTH, HALF_HEIGHT), diameter - 10, 135.0.deg(), 270.0.deg())
			.into_styled(thin_stroke)
			.draw(&mut self.display)
			.unwrap();

		let angle = lerp(self.state.speed / MAX_SPEED, 0.0, 180.0) - 180.0;

		let x1: i32 = HALF_WIDTH  + (0.300 * angle.to_radians().cos() * radius as f32) as i32;
		let y1: i32 = HALF_HEIGHT + (0.300 * angle.to_radians().sin() * radius as f32) as i32;
		let x2: i32 = HALF_WIDTH  + (0.825 * angle.to_radians().cos() * radius as f32) as i32;
		let y2: i32 = HALF_HEIGHT + (0.825 * angle.to_radians().sin() * radius as f32) as i32;
		Line::new(Point::new(x1, y1), Point::new(x2, y2))
			.into_styled(thick_stroke)
			.draw(&mut self.display)
			.unwrap();

		for i in 0..=10 {
			let speed = i * 10;

			let angle = lerp(speed as f32 / MAX_SPEED, 0.0, 180.0) - 180.0;

			let x1: i32 = HALF_WIDTH  + (0.9 * angle.to_radians().cos() * radius as f32) as i32;
			let y1: i32 = HALF_HEIGHT + (0.9 * angle.to_radians().sin() * radius as f32) as i32;
			let x2: i32 = HALF_WIDTH  + (1.0 * angle.to_radians().cos() * radius as f32) as i32;
			let y2: i32 = HALF_HEIGHT + (1.0 * angle.to_radians().sin() * radius as f32) as i32;
			
			Line::new(Point::new(x1, y1), Point::new(x2, y2))
				.into_styled(thick_stroke)
				.draw(&mut self.display)
				.unwrap();
			
			let x: i32 = HALF_WIDTH  + (1.125 * angle.to_radians().cos() * radius as f32) as i32;
			let y: i32 = HALF_HEIGHT + (1.125 * angle.to_radians().sin() * radius as f32) as i32;
			let marking_text = format!("{}", speed);
			Text::with_alignment(&marking_text, Point::new(x, y), small_text_style, Alignment::Center)
				.draw(&mut self.display)
				.unwrap();
		}

		let speed_text = format!("{:.1} km", self.state.speed);
		Text::with_alignment(&speed_text, Point::new(HALF_WIDTH, HALF_HEIGHT), large_text_style, Alignment::Center)
			.draw(&mut self.display)
			.unwrap();

		let temp_text = format!("{:.1}°C", self.state.temp);
		Text::with_alignment(&temp_text, Point::new(WIDTH - 5, 20), large_text_style, Alignment::Right)
			.draw(&mut self.display)
			.unwrap();

		self.display.flush();
	}
}

fn lerp(val: f32, min: f32, max: f32) -> f32 {
	val * (max - min) + min
}

