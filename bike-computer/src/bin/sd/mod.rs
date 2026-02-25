use esp_hal::sdmmc::SdMmc;
use embedded_sdmmc::{
	TimeSource,
	Timestamp,
	VolumeManager,
	VolumeIdx,
	Mode
};

pub struct DummyClock;

impl TimeSource for DummyClock {
	fn get_timestamp(&self) -> Timestamp {
		// fake date: Jan 1, 2026, 00:00:00
		Timestamp {
			year_since_1970: 56,
			zero_indexed_month: 0,
			zero_indexed_day: 0,
			hours: 0,
			minutes: 0,
			seconds: 0
		}
	}
}

pub struct SD<SDMMC, CLK, CMD, DATA> {
	sdmmc: SDMMC,
	clk: CLK,
	cmd: CMD,
	data: DATA,
}

impl<SDMMC, CLK, CMD, DATA> SD<SDMMC, CLK, CMD, DATA> {
	pub fn init(sdmmc: SDMMC, clk: CLK, cmd: CMD, data: DATA) -> Self {
		let sd_host_driver = SdMmc::new(
			sdmmc,
			clk,
			cmd,
			data
		);
		let mut volume_mgr = VolumeManager::new(sd_host_driver, DummyClock);
		Self {
			sdmmc,
			clk,
			cmd,
			data
		}
	}
}
