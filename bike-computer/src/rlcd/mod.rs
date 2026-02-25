// #![no_std] safe

use esp_hal::{
	gpio::Output,
	spi::master::Spi,
	Blocking
};
use embassy_time::{Duration, Timer};
use embedded_graphics_core::{
	draw_target::DrawTarget,
	pixelcolor::BinaryColor,
	prelude::*
};

#[allow(non_snake_case, non_upper_case_globals)]
pub mod BinaryColour {
	pub const Black: u8 = 0x00;
	pub const White: u8 = 0xFF;
}

pub struct Display <'d> {
	spi: Spi<'d, Blocking>,
	cs: Output<'d>,
	dc: Output<'d>,
	rst: Output<'d>,

	buffer: [u8; 15000]
}

impl<'d> Display<'d> {
	pub fn new(spi: Spi<'d, Blocking>, cs: Output<'d>, dc: Output<'d>, rst: Output<'d>) -> Self {
		Self {
			spi,
			cs,
			dc,
			rst,
			buffer: [0u8; 15000]
		}
	}

	fn write_command(&mut self, cmd: u8) {
		let _ = self.dc.set_low();
		let _ = self.cs.set_low();

		let _ = self.spi.write(&[cmd]);

		let _ = self.cs.set_high();
	}

	fn write_data_byte(&mut self, data: u8) {
		let _ = self.dc.set_high();
		let _ = self.cs.set_low();

		let _ = self.spi.write(&[data]);

		let _ = self.cs.set_high();
	}
	fn write_data(&mut self, data: &[u8]) {
		let _ = self.dc.set_high();
		let _ = self.cs.set_low();

		let _ = self.spi.write(data);

		let _ = self.cs.set_high();
	}

	pub async fn init(&mut self) {
		// wait for LCD's power circuitry to stabilise on a cold battery boot
		Timer::after(Duration::from_millis(500)).await;

		self.reset().await;

		// NVM Load Control
		self.write_command(0xD6);
		self.write_data(&[0x17, 0x02]);

		// Booster Enable
		self.write_command(0xD1);
		self.write_data_byte(0x01);

		// Gate Voltage Control
		self.write_command(0xC0);
		self.write_data(&[0x11, 0x04]);

		// VSHP Setting
		self.write_command(0xC1);
		self.write_data(&[0x69, 0x69, 0x69, 0x69]);

		self.write_command(0xC2);
		self.write_data(&[0x19, 0x19, 0x19, 0x19]);
		
		self.write_command(0xC4);
		self.write_data(&[0x4B, 0x4B, 0x4B, 0x4B]);

		self.write_command(0xC5);
		self.write_data(&[0x19, 0x19, 0x19, 0x19]);

		self.write_command(0xD8);
		self.write_data(&[0x80, 0xE9]);

		self.write_command(0xB2);
		self.write_data_byte(0x02);

		self.write_command(0xB3);
		self.write_data(&[0xE5, 0xF6, 0x05, 0x46, 0x77, 0x77, 0x77, 0x77, 0x76, 0x45]);

		self.write_command(0xB4);
		self.write_data(&[0x05, 0x46, 0x77, 0x77, 0x77, 0x77, 0x76, 0x45]);

		self.write_command(0x62);
		self.write_data(&[0x32, 0x03, 0x1F]);

		self.write_command(0xB7);
		self.write_data_byte(0x13);

		self.write_command(0xB0);
		self.write_data_byte(0x64);

		self.write_command(0x11);
		Timer::after(Duration::from_millis(200)).await;
		self.write_command(0xC9);
		self.write_data_byte(0x00);

		self.write_command(0x36);
		self.write_data_byte(0x48);

		self.write_command(0x3A);
		self.write_data_byte(0x11);
		
		self.write_command(0xB9);
		self.write_data_byte(0x20);

		self.write_command(0xB8);
		self.write_data_byte(0x29);

		self.write_command(0x21);
		
		self.write_command(0x2A);
		self.write_data(&[0x12, 0x2A]);

		self.write_command(0x2B);
		self.write_data(&[0x00, 0xC7]);

		self.write_command(0x35);
		self.write_data_byte(0x00);

		self.write_command(0xD0);
		self.write_data_byte(0xFF);

		self.write_command(0x38);
		self.write_command(0x29);

		self.colour_clear(BinaryColour::White);
	}

	pub fn colour_clear(&mut self, colour: u8) {
		self.buffer.fill(colour);
	}

	pub fn flush(&mut self) {
		// Column Address Set
		self.write_command(0x2A);
		self.write_data(&[0x12, 0x2A]);

		// Page Address Set
		self.write_command(0x2B);
		self.write_data(&[0x00, 0xC7]);

		// Page Address set
		self.write_command(0x2C);

		// inlined send command to satisfy borrow checker
		let _ = self.dc.set_high();
		let _ = self.cs.set_low();

		for chunk in self.buffer.chunks(64) {
			let _ = self.spi.write(chunk);
		}

		let _ = self.cs.set_high();
	}

	pub async fn reset(&mut self) {
		let _ = self.rst.set_high();
		Timer::after(Duration::from_millis(50)).await;

		let _ = self.rst.set_low();
		Timer::after(Duration::from_millis(20)).await;

		let _ = self.rst.set_high();
		Timer::after(Duration::from_millis(50)).await;
	}
}

impl<'d> OriginDimensions for Display<'d> {
	fn size(&self) -> Size {
		Size::new(400, 300)
	}
}

impl<'d> DrawTarget for Display<'d> {
	type Color = BinaryColor;
	type Error = core::convert::Infallible;

	fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
	where
		I: IntoIterator<Item = Pixel<Self::Color>> {
		
		for Pixel(point, colour) in pixels {
			let x = point.x as u16;
			let y = point.y as u16;

			if x >= 400 || y >= 300 {
				continue;
			}

			// Waveshare's Landscape bit-math translation
			let inv_y = 300 - 1 - y;
			let byte_x = x / 2;
			let block_y = inv_y / 4;

			let index = (byte_x * (300 / 4) + block_y) as usize;

			let local_x = x % 2;
			let local_y = inv_y % 4;
			let bit = 7 - (local_y * 2 + local_x);

			if colour.is_on() {
				self.buffer[index] |= 1 << bit;    // Turn pixel White
			} else {
				self.buffer[index] &= !(1 << bit); // Turn pixel Black
			}
		}
		Ok(())
	}
}
