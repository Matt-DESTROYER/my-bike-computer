// NMEA-0183
// #![no_std] safe

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TalkerID {
	GP, // GPS, SBAS, QZSS
	GL, // GLONASS
	GA, // GALILEO
	GB, // BEIDOU
	GN  // any combination of GNSS
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MessageType {
	Unknown,
	GGA,
	GLL,
	GSA,
	GSV,
	RMC,
	VTG
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Quality {
	NoFix,
	StandardGPS,
	DifferentialGPS,
	EstimatedFix
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NavMode {
	NotAvailable,
	Fix2D,
	Fix3D
}

#[derive(Debug, Copy, Clone)]
pub struct Time {
	hour: u16,
	minute: u16,
	second: f32
}

#[derive(Debug, Copy, Clone)]
pub struct Date {
	day: u8,
	month: u8,
	year: u8
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct GLL {
	pub lat: f64,     // latitude
	pub NS: char,     // North/South indicator
	pub long: f64,    // Longitude
	pub EW: char,     // East/West indicator
	pub time: Time,   // UTC time
	pub status: char, // V = Data invalid or receiver warning, A = Data valid
	pub posMode: char // positioning mode
}

#[derive(Debug)]
pub struct GSA {
	pub opMode: char,
	pub navMode: NavMode,
	pub sv: [u8; 12],
	pub PDOP: f64,
	pub HDOP: f64,
	pub VDOP: f64,
	pub systemId: u8
}

#[derive(Debug)]
pub struct GSV {
	pub numMsg: u8,
	pub msgNum: u8,
	pub numSV: u8,
	pub SV: [u8; 4],
	pub elv: [u8; 4],
	pub az: [u16; 4],
	pub cno: [u8; 4],
	pub signalId: u8
}

#[derive(Debug)]
pub struct RMC {
	pub time: Time,
	pub status: char,
	pub lat: f64,
	pub NS: char,
	pub long: f64,
	pub EW: char,
	pub spd: f64,
	pub cog: f64,
	pub date: Date,
	pub mv: f64,
	pub mvEW: char,
	pub posMode: char,
	pub navStatus: char
}

#[derive(Debug)]
pub struct VTG {
	pub cogt: f64,
	pub T: char,
	pub cogm: f64,
	pub M: char,
	pub knots: f64,
	pub N: char,
	pub kph: f64,
	pub K: char,
	pub posMode: char
}

#[derive(Debug)]
pub enum ParserResult {
	GGA(GGA),
	GLL(GLL),
	GSA(GSA),
	GSV(GSV),
	RMC(RMC),
	VTG(VTG)
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
			quality: Quality::NoFix,
			numSV: 0,
			HDOP: 0.0,
			alt: 0.0,
			uAlt: '\0',
			sep: 0.0,
			uSep: '\0',
			diffAge: None,
			diffStation: None
		})),
		MessageType::GLL => Some(ParserResult::GLL(GLL {
			lat: 0.0,
			NS: '\0',
			long: 0.0,
			EW: '\0',
			time: Time {
				hour: 0,
				minute: 0,
				second: 0.0
			},
			status: '\0',
			posMode: '\0'
		})),
		MessageType::GSA => Some(ParserResult::GSA(GSA {
			opMode: '\0',
			navMode: NavMode::NotAvailable,
			sv: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
			PDOP: 0.0,
			HDOP: 0.0,
			VDOP: 0.0,
			systemId: 0
		})),
		MessageType::GSV => Some(ParserResult::GSV(GSV {
			numMsg: 0,
			msgNum: 0,
			numSV: 0,
			SV: [0, 0, 0, 0],
			elv: [0, 0, 0, 0],
			az: [0, 0, 0, 0],
			cno: [0, 0, 0, 0],
			signalId: 0
		})),
		MessageType::RMC => Some(ParserResult::RMC(RMC {
			time: Time {
				hour: 0,
				minute: 0,
				second: 0.0
			},
			status: '\0',
			lat: 0.0,
			NS: '\0',
			long: 0.0,
			EW: '\0',
			spd: 0.0,
			cog: 0.0,
			date: Date {
				day: 0,
				month: 0,
				year: 0
			},
			mv: 0.0,
			mvEW: '\0',
			posMode: '\0',
			navStatus: '\0'
		})),
		MessageType::VTG => Some(ParserResult::VTG(VTG {
			cogt: 0.0,
			T: '\0',
			cogm: 0.0,
			M: '\0',
			knots: 0.0,
			N: '\0',
			kph: 0.0,
			K: '\0',
			posMode: '\0'
		})),
		_ => None
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ParserState {
	ParsingValue,
	ParsingFormat,
	CheckingChecksum,
	Finishing
}

#[derive(Debug)]
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
			result = (result << 4) + match hex[i] {
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
			let current_buffer = &self.buffer[0..self.index];
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
							if self.index >= 2 {
								let degrees: u16 = Parser::parse_u16_from_u8_buffer(&self.buffer[0..2]);
								let minutes: f64 = Parser::parse_f64_from_u8_buffer(&self.buffer[2..self.index]);
								gga.lat = degrees as f64 + minutes / 60.0;
							}
						},
						// NS char
						3 => {
							gga.NS = self.buffer[0] as char;

							if gga.NS == 'S' {
								gga.lat = -gga.lat;
							}
						}
						// long "dddmm.mmmmm"
						4 => {
							if self.index >= 3 {
								let degrees: u16 = Parser::parse_u16_from_u8_buffer(&self.buffer[0..3]);
								let minutes: f64 = Parser::parse_f64_from_u8_buffer(&self.buffer[3..self.index]);
								gga.long = degrees as f64 + minutes / 60.0;
							}
						},
						// EW char
						5 => {
							gga.EW = self.buffer[0] as char;

							if gga.EW == 'W' {
								gga.long = -gga.long;
							}
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
							gga.numSV = Parser::parse_u8_from_u8_buffer(current_buffer);
						},
						// HDOP numeric
						8 => {
							gga.HDOP = Parser::parse_f64_from_u8_buffer(current_buffer);
						},
						// alt numeric
						9 => {
							gga.alt = Parser::parse_f64_from_u8_buffer(current_buffer);
						},
						// uAlt char
						10 => {
							gga.uAlt = self.buffer[0] as char;
						},
						// sep numeric
						11 => {
							gga.sep = Parser::parse_f64_from_u8_buffer(current_buffer);
						},
						// uSep char
						12 => {
							gga.uSep = self.buffer[0] as char;
						},
						// diffAge numeric
						13 => {
							gga.diffAge = Some(Parser::parse_f64_from_u8_buffer(current_buffer));
						},
						// diffStation numeric
						14 => {
							gga.diffStation = Some(Parser::parse_f64_from_u8_buffer(current_buffer));
						},
						_ => {}
					}
				},
				ParserResult::GLL(gll) => {
					match self.value_index {
						// lat "ddmm.mmmmm"
						1 => {
							if self.index >= 2 {
								let degrees: u16 = Parser::parse_u16_from_u8_buffer(&self.buffer[0..2]);
								let minutes: f64 = Parser::parse_f64_from_u8_buffer(&self.buffer[2..self.index]);
								gll.lat = degrees as f64 + minutes / 60.0;
							}
						},
						// NS char
						2 => {
							gll.NS = self.buffer[0] as char;

							if gll.NS == 'S' {
								gll.lat = -gll.lat;
							}
						},
						// long "dddmm.mmmmm"
						3 => {
							if self.index >= 3 {
								let degrees: u16 = Parser::parse_u16_from_u8_buffer(&self.buffer[0..3]);
								let minutes: f64 = Parser::parse_f64_from_u8_buffer(&self.buffer[3..self.index]);
								gll.long = degrees as f64 + minutes / 60.0;
							}
						},
						// EW char
						4 => {
							gll.EW = self.buffer[0] as char;

							if gll.EW == 'W' {
								gll.long = -gll.long;
							}
						},
						// time "hhmmss.ss"
						5 => {
							if self.index >= 4 {
								let hour: u16   = Parser::parse_u16_from_u8_buffer(&self.buffer[0..2]);
								let minute: u16 = Parser::parse_u16_from_u8_buffer(&self.buffer[2..4]);
								let second: f32 = Parser::parse_f32_from_u8_buffer(&self.buffer[4..self.index]);

								gll.time = Time { hour, minute, second };
							}
						},
						// status char
						6 => {
							gll.status = self.buffer[0] as char
						},
						// posMode char
						7 => {
							gll.posMode = self.buffer[0] as char
						},
						_ => {}
					}
				},
				ParserResult::GSA(gsa) => {
					match self.value_index {
						1 => {
							gsa.opMode = self.buffer[0] as char;
						},
						2 => {
							gsa.navMode = match self.buffer[0] {
								1 => NavMode::NotAvailable,
								2 => NavMode::Fix2D,
								3 => NavMode::Fix3D,
								_ => NavMode::NotAvailable
							};
						},
						3..15 => {
							gsa.sv[self.value_index - 3] = Parser::parse_u8_from_u8_buffer(current_buffer);
						},
						15 => {
							gsa.PDOP = Parser::parse_f64_from_u8_buffer(current_buffer)
						},
						16 => {
							gsa.HDOP = Parser::parse_f64_from_u8_buffer(current_buffer)
						},
						17 => {
							gsa.VDOP = Parser::parse_f64_from_u8_buffer(current_buffer)
						},
						18 => {
							gsa.systemId = Parser::parse_u8_from_u8_buffer(current_buffer)
						},
						_ => {}
					}
				},
				ParserResult::GSV(gsv) => {
					match self.value_index {
						1 => {
							gsv.numMsg = Parser::parse_u8_from_u8_buffer(current_buffer);
						},
						2 => {
							gsv.msgNum = Parser::parse_u8_from_u8_buffer(current_buffer);
						},
						3 => {
							gsv.numSV = Parser::parse_u8_from_u8_buffer(current_buffer);
						},
						4..20 => {
							let idx = (self.value_index -  4) / 4;
							match (self.value_index - 4) % 4 {
								0 => {
									gsv.SV[idx] = Parser::parse_u8_from_u8_buffer(current_buffer);
								},
								1 => {
									gsv.elv[idx] = Parser::parse_u8_from_u8_buffer(current_buffer);
								},
								2 => {
									gsv.az[idx] = Parser::parse_u16_from_u8_buffer(current_buffer);
								},
								3 => {
									gsv.cno[idx] = Parser::parse_u8_from_u8_buffer(current_buffer);
								},
								_ => { /* not possible, rust compiler! */ }
							}
						},
						20 => {
							gsv.signalId = Parser::parse_u8_from_u8_buffer(current_buffer);
						},
						_ => {}
					}
				},
				ParserResult::RMC(rmc) => {
					match self.value_index {
						1 => {
							if self.index >= 4 {
								let hour: u16   = Parser::parse_u16_from_u8_buffer(&self.buffer[0..2]);
								let minute: u16 = Parser::parse_u16_from_u8_buffer(&self.buffer[2..4]);
								let second: f32 = Parser::parse_f32_from_u8_buffer(&self.buffer[4..self.index]);

								rmc.time = Time { hour, minute, second };
							}
						},
						2 => {
							rmc.status = self.buffer[0] as char;
						},
						// lat "ddmm.mmmmm"
						3 => {
							if self.index >= 2 {
								let degrees: u16 = Parser::parse_u16_from_u8_buffer(&self.buffer[0..2]);
								let minutes: f64 = Parser::parse_f64_from_u8_buffer(&self.buffer[2..self.index]);
								rmc.lat = degrees as f64 + minutes / 60.0;
							}
						},
						// NS char
						4 => {
							rmc.NS = self.buffer[0] as char;

							if rmc.NS == 'S' {
								rmc.lat = -rmc.lat;
							}
						}
						// long "dddmm.mmmmm"
						5 => {
							if self.index >= 3 {
								let degrees: u16 = Parser::parse_u16_from_u8_buffer(&self.buffer[0..3]);
								let minutes: f64 = Parser::parse_f64_from_u8_buffer(&self.buffer[3..self.index]);
								rmc.long = degrees as f64 + minutes / 60.0;
							}
						},
						// EW char
						6 => {
							rmc.EW = self.buffer[0] as char;

							if rmc.EW == 'W' {
								rmc.long = -rmc.long;
							}
						},
						7 => {
							rmc.spd = Parser::parse_f64_from_u8_buffer(current_buffer);
						},
						8 => {
							rmc.cog = Parser::parse_f64_from_u8_buffer(current_buffer);
						},
						// date "ddmmyy"
						9 => {
							if self.index >= 4 {
								let day: u8   = Parser::parse_u8_from_u8_buffer(&self.buffer[0..2]);
								let month: u8 = Parser::parse_u8_from_u8_buffer(&self.buffer[2..4]);
								let year: u8  = Parser::parse_u8_from_u8_buffer(&self.buffer[4..6]);

								rmc.date = Date { day, month, year };
							}
						},
						10 => {
							rmc.mv = Parser::parse_f64_from_u8_buffer(current_buffer);
						},
						11 => {
							rmc.mvEW = self.buffer[0] as char;
						},
						12 => {
							rmc.posMode = self.buffer[0] as char;
						},
						13 => {
							rmc.navStatus = self.buffer[0] as char;
						},
						_ => {}
					}
				},
				ParserResult::VTG(vtg) => {
					match self.value_index {
						1 => {
							vtg.cogt = Parser::parse_f64_from_u8_buffer(current_buffer);
						},
						2 => {
							vtg.T = self.buffer[0] as char;
						},
						3 => {
							vtg.cogm = Parser::parse_f64_from_u8_buffer(current_buffer);
						},
						4 => {
							vtg.M = self.buffer[0] as char;
						},
						5 => {
							vtg.knots = Parser::parse_f64_from_u8_buffer(current_buffer);
						},
						6 => {
							vtg.N = self.buffer[0] as char;
						},
						7 => {
							vtg.kph = Parser::parse_f64_from_u8_buffer(current_buffer);
						},
						8 => {
							vtg.K = self.buffer[0] as char;
						},
						9 => {
							vtg.posMode = self.buffer[0] as char;
						},
						_ => {}
					}
				},
				_ => {}
			}
		}
	}

	pub fn parse_byte(self: &mut Self, byte: u8) -> &Option<ParserResult> {
		if byte != b'*' &&
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
						return &self.result;
					},
					b'*' => {
						self.parse_value();
						self.index = 0;
						self.state = ParserState::CheckingChecksum;
						return &self.result;
					},
					b'$' => {
						self.checksum = 0;
						self.valid_checksum = false;
						self.state = ParserState::ParsingFormat;
						return &self.result;
					},
					b'\r' => {
						self.state = ParserState::Finishing;
						return &self.result;
					},
					_ => {}
				}
			},
			ParserState::ParsingFormat => {
				if byte == b',' {
					let format_slice = &self.buffer[0..self.index];
					if format_slice.ends_with(b"GGA") {
						self.format = MessageType::GGA;
					} else if format_slice.ends_with(b"GLL") {
						self.format = MessageType::GLL;
					} else if format_slice.ends_with(b"GSA") {
						self.format = MessageType::GSA;
					} else if format_slice.ends_with(b"GSV") {
						self.format = MessageType::GSV;
					} else if format_slice.ends_with(b"RMC") {
						self.format = MessageType::RMC;
					} else if format_slice.ends_with(b"VTG") {
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
				
				self.finished = false;
				self.state = ParserState::ParsingValue;
				self.index = 0;
				self.value_index = 0;
				self.result = None;
				self.parse_byte(byte);
				return &self.result;
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
