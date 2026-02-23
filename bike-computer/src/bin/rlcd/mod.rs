// NMEA-0183
// #![no_std] safe

use esp_hal::{
	gpio::{
		Output,
		Input
	},
	spi::master::Spi,
	Blocking
};

pub struct Display <'d> {
	spi: Spi<'d, Blocking>,
	cs: Output<'d>,
	dc: Output<'d>,
	rst: Output<'d>,
	busy: Input<'d>,

	buffer: [u8; 15000]
}

impl<'d> Display<'d> {
	pub fn new(spi: Spi<'d, Blocking>, cs: Output<'d>, dc: Output<'d>, rst: Output<'d>, busy: Input<'d>) -> Self {
		Self {
			spi,
			cs,
			dc,
			rst,
			busy,
			buffer: [0u8; 15000]
		}
	}

	fn write_command(&mut self, cmd: u8) {
		let _ = self.dc.set_low();
		let _ = self.cs.set_low();

		let _ = self.spi.write(&[cmd]);

		let _ = self.cs.set_high();
	}

	fn write_data(&mut self, data: u8) {
		let _ = self.dc.set_high();
		let _ = self.cs.set_low();

		let _ = self.spi.write(&[data]);

		let _ = self.cs.set_high();
	}
}
