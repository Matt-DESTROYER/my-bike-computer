#![no_std]

// NMEA-0183

#[derive(Copy, Clone, PartialEq)]
pub enum TalkerID {
	GP, // GPS, SBAS, QZSS
	GL, // GLONASS
	GA, // GALILEO
	GB, // BEIDOU
	GN  // any combination of GNSS
}

#[derive(Copy, Clone, PartialEq)]
pub enum MessageType {
	Unknown,
	GGA,
	GLL,
	GSA,
	GSV,
	RMC,
	VTG
}

#[derive(Copy, Clone, PartialEq)]
pub enum Quality {
	NoFix,
	StandardGPS,
	DifferentialGPS,
	EstimatedFix
}

pub struct Time {
	hour: u16,
	minute: u16,
	second: f32
}

pub struct GGA {
	pub time: Time,              // UTC time
	pub lat: f64,                // latitude
	pub NS: char,                // North/South indicator
	pub long: f64,               // Longitude
	pub EW: char,                // East/West indicator
	pub quality: Quality,        // quality
	pub numSV: u8,               // number of satellites used
	pub HDOP: f64,               // horizontal dilution of precision
	pub alt: f64,                // altitude above mean sea level
	pub uAlt: char,              // altutude units: meters (fixed field)
	pub sep: f64,                // geoid separation: difference between
	pub uSep: char,              // separation units: meters (fixed field)
	pub diffAge: Option<f64>,    // age of differential corrections (blank when DGPS is not used)
	pub diffStation: Option<f64> // id of station providing differential corrections (blank when DGPS is not used)
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

fn check_format_equals(format: &[u8], cmp: &str) -> bool {
	let cmp_bytes = cmp.as_bytes();
	let iters = if cmp.len() > format.len() {
		format.len()
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
fn check_format_endswith(format: &[u8], cmp: &str) -> bool {
	let cmp_bytes = cmp.as_bytes();
	let iters = if cmp_bytes.len() > format.len() {
		format.len()
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
fn init_format(format: MessageType) -> Option<ParserResult> {
	match format {
		MessageType::GGA => Some(ParserResult::GGA(GGA {
			time: Time {
				hour: 0,
				minute: 0,
				second: 0.0
			},
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
		})),
		MessageType::GLL => Some(ParserResult::GLL(GLL {})),
		MessageType::GSA => Some(ParserResult::GSA(GSA {})),
		MessageType::GSV => Some(ParserResult::GSV(GSV {})),
		MessageType::RMC => Some(ParserResult::RMC(RMC {})),
		MessageType::VTG => Some(ParserResult::VTG(VTG {})),
		_ => None
	}
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
	value_index: usize,

	pub format: MessageType,

	pub result: Option<ParserResult>,
	pub finished: bool
}

impl Parser {
	pub fn new() -> Parser {
		return Parser {
			state: ParserState::ParsingValue,
			checksum: 0,
			valid_checksum: false,
			buffer: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			index: 0,
			value_index: 0,
			format: MessageType::Unknown,
			result: None,
			finished: false
		};
	}

	fn hex_to_dec(hex: &[u8]) -> usize {
		let mut result: usize = 0;
		for i in 0..hex.len() {
			result = (result << 4) + match hex[hex.len() - i - 1] {
				b'0' => 0x0,
				b'1' => 0x1,
				b'2' => 0x2,
				b'3' => 0x3,
				b'4' => 0x4,
				b'5' => 0x5,
				b'6' => 0x6,
				b'7' => 0x7,
				b'8' => 0x8,
				b'9' => 0x9,
				b'a' | b'A' => 0xA,
				b'b' | b'B' => 0xB,
				b'c' | b'C' => 0xC,
				b'd' | b'D' => 0xD,
				b'e' | b'E' => 0xE,
				b'f' | b'F' => 0xF,
				_ => return 0
			};
		}
		return result;
	}

	fn parse_u8_from_u8_buffer(buff: &[u8]) -> u8 {
		if buff.len() < 1 {
			return 0;
		}

		let mut result: u8 = 0;
		for i in 0..buff.len() {
			if buff[i] < b'0' || buff[i] > b'9' {
				continue;
			}
			result = result * 10 + (buff[i] - b'0') as u8;
		}

		return result;
	}
	fn parse_u16_from_u8_buffer(buff: &[u8]) -> u16 {
		if buff.len() < 1 {
			return 0;
		}

		let mut result: u16 = 0;
		for i in 0..buff.len() {
			if buff[i] < b'0' || buff[i] > b'9' {
				continue;
			}
			result = result * 10 + (buff[i] - b'0') as u16;
		}

		return result;
	}
	fn parse_u32_from_u8_buffer(buff: &[u8]) -> u32 {
		if buff.len() < 1 {
			return 0;
		}
		
		let mut result: u32 = 0;
		for i in 0..buff.len() {
			if buff[i] < b'0' || buff[i] > b'9' {
				continue;
			}
			result = result * 10 + (buff[i] - b'0') as u32;
		}

		return result;
	}
	fn parse_f32_from_u8_buffer(buff: &[u8]) -> f32 {
		if buff.len() < 1 {
			return 0.0;
		}

		// find '.'
		let mut decimal_point_idx = buff.len();
		for i in 0..buff.len() {
			if buff[i] == b'.' {
				decimal_point_idx = i;
				break;
			}
		}

		let mut whole_part: f32 = 0.0;
		for i in 0..decimal_point_idx {
			if buff[i] < b'0' || buff[i] > b'9' {
				continue;
			}
			whole_part = whole_part * 10.0 + (buff[i] - b'0') as f32;
		}

		if decimal_point_idx == buff.len() {
			return whole_part;
		}

		let mut decimal_part: f32 = 0.0;
		for i in 0..buff.len() - decimal_point_idx - 1 {
			let idx = buff.len() - i - 1;
			if buff[idx] < b'0' || buff[idx] > b'9' {
				continue;
			}
			decimal_part = decimal_part / 10.0 + (buff[idx] - b'0') as f32;
		}

		return whole_part + decimal_part / 10.0;
	}
	fn parse_f64_from_u8_buffer(buff: &[u8]) -> f64 {
		if buff.len() < 1 {
			return 0.0;
		}

		// find '.'
		let mut decimal_point_idx = buff.len();
		for i in 0..buff.len() {
			if buff[i] == b'.' {
				decimal_point_idx = i;
				break;
			}
		}

		let mut whole_part: f64 = 0.0;
		for i in 0..decimal_point_idx {
			if buff[i] < b'0' || buff[i] > b'9' {
				continue;
			}
			whole_part = whole_part * 10.0 + (buff[i] - b'0') as f64;
		}

		if decimal_point_idx == buff.len() {
			return whole_part;
		}

		let mut decimal_part: f64 = 0.0;
		for i in 0..buff.len() - decimal_point_idx - 1 {
			let idx = buff.len() - i - 1;
			if buff[idx] < b'0' || buff[idx] > b'9' {
				continue;
			}
			decimal_part = decimal_part / 10.0 + (buff[idx] - b'0') as f64;
		}

		return whole_part + decimal_part / 10.0;
	}

	fn parse_value(self: &mut Self) {
		self.value_index += 1;

		if self.index == 0 {
			return;
		}

		if let Some(ref mut result) = self.result {
			match result {
				ParserResult::GGA(gga) => {
					match self.value_index {
						// time "hhmmss.ss"
						1 => {
							if self.index >= 4 {
								let hour: u16   = Parser::parse_u16_from_u8_buffer(&self.buffer[0..2]);
								let minute: u16 = Parser::parse_u16_from_u8_buffer(&self.buffer[2..4]);
								let second: f32 = Parser::parse_f32_from_u8_buffer(&self.buffer[4..self.index]);

								gga.time = Time { hour, minute, second };
							}
						},
						// lat "ddmm.mmmmm"
						2 => {
							let degrees: u16 = Parser::parse_u16_from_u8_buffer(&self.buffer[0..2]);
							let minutes: f64 = Parser::parse_f64_from_u8_buffer(&self.buffer[2..self.index]);
							gga.lat = degrees as f64 + minutes / 60.0;
						},
						// NS char
						3 => {
							gga.NS = self.buffer[0] as char;
						}
						// long "dddmm.mmmmm"
						4 => {
							let degrees: u16 = Parser::parse_u16_from_u8_buffer(&self.buffer[0..3]);
							let minutes: f64 = Parser::parse_f64_from_u8_buffer(&self.buffer[3..self.index]);
							gga.long = degrees as f64 + minutes / 60.0;
						},
						// EW char
						5 => {
							gga.EW = self.buffer[0] as char;
						},
						// quality digit
						6 => {
							gga.quality = match self.buffer[0] {
								b'0' => Quality::NoFix,
								b'1' => Quality::StandardGPS,
								b'2' => Quality::DifferentialGPS,
								b'6' => Quality::EstimatedFix,
								_ => Quality::NoFix
							}
						},
						// numSV numeric
						7 => {
							gga.numSV = Parser::parse_u8_from_u8_buffer(&self.buffer[0..self.index]);
						},
						// HDOP numeric
						8 => {
							gga.HDOP = Parser::parse_f64_from_u8_buffer(&self.buffer[0..self.index]);
						},
						// alt numeric
						9 => {
							gga.alt = Parser::parse_f64_from_u8_buffer(&self.buffer[0..self.index]);
						},
						// uAlt char
						10 => {
							gga.uAlt = self.buffer[0] as char;
						},
						// sep numeric
						11 => {
							gga.sep = Parser::parse_f64_from_u8_buffer(&self.buffer[0..self.index]);
						},
						// uSep char
						12 => {
							gga.uSep = self.buffer[0] as char;
						},
						// diffAge numeric
						13 => {
							gga.diffAge = Some(Parser::parse_f64_from_u8_buffer(&self.buffer[0..self.index]));
						},
						// diffStation numeric
						14 => {
							gga.diffStation = Some(Parser::parse_f64_from_u8_buffer(&self.buffer[0..self.index]));
						},
						_ => {}
					}
				}
				_ => {}
			}
		}
	}

	pub fn parse_byte(self: &mut Self, byte: u8) -> &Option<ParserResult> {
		if self.state != ParserState::ParsingValue &&
				self.state != ParserState::CheckingChecksum &&
				self.state != ParserState::Finishing {
			self.checksum ^= byte;
		}

		match &self.state {
			ParserState::ParsingValue => {
				match byte {
					b',' => {
						self.parse_value();
						self.index = 0;
						self.state = ParserState::ParsingValue;
					},
					b'*' => {
						self.parse_value();
						self.index = 0;
						self.state = ParserState::CheckingChecksum;
					},
					b'$' => {
						self.checksum = 0;
						self.valid_checksum = false;
						self.state = ParserState::ParsingFormat;
					},
					b'\r' => self.state = ParserState::Finishing,
					_ => {}
				}
			},
			ParserState::ParsingFormat => {
				if byte == b',' {
					if check_format_endswith(&self.buffer[0..self.index], "GGA") {
						self.format = MessageType::GGA;
					} else if check_format_endswith(&self.buffer[0..self.index], "GLL") {
						self.format = MessageType::GLL;
					} else if check_format_endswith(&self.buffer[0..self.index], "GSA") {
						self.format = MessageType::GSA;
					} else if check_format_endswith(&self.buffer[0..self.index], "GSV") {
						self.format = MessageType::GSV;
					} else if check_format_endswith(&self.buffer[0..self.index], "RMC") {
						self.format = MessageType::RMC;
					} else if check_format_endswith(&self.buffer[0..self.index], "VTG") {
						self.format = MessageType::VTG;
					} else {
						self.format = MessageType::Unknown;
					}

					self.result = init_format(self.format);

					self.finished = false;
					self.state = ParserState::ParsingValue;
					self.index = 0;
					self.value_index = 0;
					return &self.result;
				}
			},
			ParserState::CheckingChecksum => {}
			ParserState::Finishing => {
				if byte == b'\n' {
					self.finished = true;
					return &self.result;
				}

				// erm... how did we end up here.. smthn's wrong...
			},
		}

		// prevent potential overflows if a comma is missed due to invalid data
		if self.index < self.buffer.len() {
			self.buffer[self.index] = byte;
			self.index += 1;
		}

		// check the checksum is correct
		if self.state == ParserState::CheckingChecksum && self.index > 1 {
			let received_checksum: u8 = Parser::hex_to_dec(&self.buffer[0..2]) as u8;
			self.valid_checksum = received_checksum == self.checksum;
			self.state = ParserState::ParsingValue;
			self.index = 0;
		}

		return &self.result;
	}
}
