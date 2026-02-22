#![no_std]

// NMEA-0183

#[derive(PartialEq)]
pub enum TalkerID {
	GP, // GPS, SBAS, QZSS
	GL, // GLONASS
	GA, // GALILEO
	GB, // BEIDOU
	GN  // any combination of GNSS
}

#[derive(PartialEq)]
pub enum MessageType {
	GGA,
	GLL,
	GSA,
	GSV,
	RMC,
	VTG
}

#[derive(PartialEq)]
pub enum Quality {
	NoFix,
	StandardGPS,
	DifferentialGPS,
	EstimatedFix
}

pub struct GGA {
	pub time: u64,               // UTC time
	pub lat: f32,                // latitude
	pub NS: char,                // North/South indicator
	pub long: f32,               // Longitude
	pub EW: char,                // East/West indicator
	pub quality: Quality,        // quality
	pub numSV: u8,               // number of satellites used
	pub HDOP: f32,               // horizontal dilution of precision
	pub alt: f32,                // altitude above mean sea level
	pub uAlt: char,              // altutude units: meters (fixed field)
	pub sep: f32,                // geoid separation: difference between
	pub uSep: char,              // separation units: meters (fixed field)
	pub diffAge: Option<f32>,    // age of differential corrections (blank when DGPS is not used)
	pub diffStation: Option<f32> // id of station providing differential corrections (blank when DGPS is not used)
}

pub struct GLL {}

pub struct GSA {}

pub struct GSV {}

pub struct RMC {}

pub struct VTG {}

pub enum ParserResult {
	GGA(GGA),
	GLL(GLL),
	GSA(GSA),
	GSV(GSV),
	RMC(RMC),
	VTG(VTG)
}

fn check_format_equals(format: &[u8; 5], cmp: &str) -> bool {
	let cmp_bytes = cmp.as_bytes();
	let iters = if cmp.len() > 5 {
		5
	} else {
		cmp.len()
	};
	for i in 0..iters {
		if format[i] != cmp_bytes[i] {
			return false;
		}
	}
	return true;
}
fn check_format_endswith(format: &[u8; 5], cmp: &str) -> bool {
	let cmp_bytes = cmp.as_bytes();
	let iters = if cmp_bytes.len() > 5 {
		5
	} else {
		cmp_bytes.len()
	};
	for i in 1..=iters {
		if format[5 - i] != cmp_bytes[cmp_bytes.len() - i] {
			return false;
		}
	}
	return true;
}
fn init_format(format: &[u8; 5]) -> Option<ParserResult> {
	if check_format_endswith(format, "GGA") {
		return Some(ParserResult::GGA(GGA {
			time: 0,
			lat: 0.0,
			NS: '\0',
			long: 0.0,
			EW: '\0',
			quality: Quality::StandardGPS,
			numSV: 0,
			HDOP: 0.0,
			alt: 0.0,
			uAlt: '\0',
			sep: 0.0,
			uSep: '\0',
			diffAge: None,
			diffStation: None
		}));
	} else if check_format_endswith(format, "GLL") {
		return Some(ParserResult::GLL(GLL {}));
	} else if check_format_endswith(format, "GSA") {
		return Some(ParserResult::GSA(GSA {}));
	} else if check_format_endswith(format, "GSV") {
		return Some(ParserResult::GSV(GSV {}));
	} else if check_format_endswith(format, "RMC") {
		return Some(ParserResult::RMC(RMC {}));
	} else if check_format_endswith(format, "VTG") {
		return Some(ParserResult::VTG(VTG {}));
	}
	return None;
}

#[derive(PartialEq)]
enum ParserState {
	ParsingValue,
	ParsingFormat,
	CheckingChecksum,
	Finishing
}

pub struct Parser {
	state: ParserState,

	checksum: u8,
	pub valid_checksum: bool,

	buffer: [u8; 16],
	index: usize,

	format: [u8; 5],

	result: Option<ParserResult>
}

fn hex_to_dec(hex: &[u8]) -> usize {
	let mut result: usize = 0;
	for i in 0..hex.len() {
		result += match hex[hex.len() - i - 1] as char {
			'0' => 0x0,
			'1' => 0x1,
			'2' => 0x2,
			'3' => 0x3,
			'4' => 0x4,
			'5' => 0x5,
			'6' => 0x6,
			'7' => 0x7,
			'8' => 0x8,
			'9' => 0x9,
			'a' | 'A' => 0xA,
			'b' | 'B' => 0xB,
			'c' | 'C' => 0xC,
			'd' | 'D' => 0xD,
			'e' | 'E' => 0xE,
			'f' | 'F' => 0xF,
			_ => return 0
			// 16^n = 2^4^n = 1 << (4 * n)
		} * 1usize.checked_shl(4 * i as u32).unwrap_or(0);
	}
	return result;
}

impl Parser {
	pub fn new() -> Parser {
		return Parser {
			state: ParserState::ParsingValue,
			checksum: 0,
			valid_checksum: false,
			buffer: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			index: 0,
			format: ['\0', '\0', '\0', '\0', '\0'],
			result: None
		};
	}

	fn parse_value(self: &mut Self) {}

	pub fn parse_byte(self: &mut Self, byte: u8) -> &Option<ParserResult> {
		if self.state != ParserState::ParsingValue &&
				self.state != ParserState::CheckingChecksum &&
				self.state != ParserState::Finishing {
			self.checksum ^= byte;
		}

		match &self.state {
			ParserState::ParsingValue => {
				match byte as char {
					',' => {
						self.parse_value();
						self.index = 0;
						self.state = ParserState::ParsingValue;
					},
					'*' => {
						self.parse_value();
						self.index = 0;
						self.state = ParserState::CheckingChecksum;
					},
					'$' => {
						self.checksum = 0;
						self.valid_checksum = false;
						self.state = ParserState::ParsingFormat;
					},
					'\r' => self.state = ParserState::Finishing,
					_ => {}
				}
			},
			ParserState::ParsingFormat => {
				if byte as char == ',' {
					for i in 0..5 {
						self.format[i] = self.buffer[i];
					}

					self.result = init_format(&self.format);

					self.state = ParserState::ParsingValue;
					self.index = 0;
					return &self.result;
				}
			},
			ParserState::CheckingChecksum => {}
			ParserState::Finishing => {
				if byte as char == '\n' {
					return &self.result;
				}

				// erm... how did we end up here.. smthn's wrong...
			},
		}

		if self.index < self.buffer.len() {
			self.buffer[self.index] = byte;
		}
		self.index += 1;

		if self.state == ParserState::CheckingChecksum && self.index > 1 {
			let received_checksum: u8 = hex_to_dec(&self.buffer[0..2]) as u8;
			self.valid_checksum = received_checksum == self.checksum;
			self.state = ParserState::ParsingValue;
			self.index = 0;
		}

		return &self.result;
	}
}
