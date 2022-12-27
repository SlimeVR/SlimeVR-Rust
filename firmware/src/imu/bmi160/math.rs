const RAD_PER_DEG: f32 = 2. * core::f32::consts::PI / 360.;
const DEG_PER_RAD: f32 = 1. / RAD_PER_DEG;

// TODO: This whole module needs unit tests

/// The Full Scale Range of the gyroscope. For example, D2000 means +/- 2000 degrees/sec
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum GyroFsr {
	D2000,
	D1000,
	D500,
	D250,
	D125,
}
#[allow(dead_code)]
impl GyroFsr {
	/// The default FSR when the IMU is reset
	pub const DEFAULT: Self = Self::D2000;
	pub const fn from_reg(v: u8) -> Result<Self, InvalidBitPattern> {
		Ok(match v {
			0b000 => Self::D2000,
			0b001 => Self::D1000,
			0b010 => Self::D500,
			0b011 => Self::D250,
			0b100 => Self::D125,
			_ => return Err(InvalidBitPattern),
		})
	}

	pub const fn to_reg(self) -> u8 {
		match self {
			Self::D2000 => 0b000,
			Self::D1000 => 0b001,
			Self::D500 => 0b010,
			Self::D250 => 0b011,
			Self::D125 => 0b100,
		}
	}

	pub const fn as_u16(self) -> u16 {
		match self {
			Self::D2000 => 2000,
			Self::D1000 => 1000,
			Self::D500 => 500,
			Self::D250 => 250,
			Self::D125 => 125,
		}
	}

	pub const fn from_u16(v: u16) -> Result<Self, InvalidNum> {
		// TODO: I'm not confident this is performant
		Ok(match v {
			v if v == Self::D2000.as_u16() => Self::D2000,
			v if v == Self::D1000.as_u16() => Self::D1000,
			v if v == Self::D500.as_u16() => Self::D500,
			v if v == Self::D250.as_u16() => Self::D250,
			v if v == Self::D125.as_u16() => Self::D125,
			_ => return Err(InvalidNum),
		})
	}

	/// least signficant bits per deg/s
	pub const fn lsb_per_dps(self) -> f32 {
		let range: f32 = self.as_u16() as _;
		// Add 1 because there is MAX+1 numbers due to `0`
		const TMP: f32 = i16::MAX as f32 + 1.;
		TMP / range
	}

	/// deg/s per least significant bit
	pub const fn dps_per_lsb(self) -> f32 {
		let range: f32 = self.as_u16() as _;
		// Add 1 because there is MAX+1 numbers due to `0`
		const TMP: f32 = 1. / (i16::MAX as f32 + 1.);
		range * TMP
	}

	/// least significant bits per rad/s
	pub const fn lsb_per_rad(self) -> f32 {
		self.lsb_per_dps() * DEG_PER_RAD
	}

	/// rad/s per least significant bit
	pub const fn rad_per_lsb(self) -> f32 {
		self.dps_per_lsb() * RAD_PER_DEG
	}
}

#[derive(Debug)]
pub struct InvalidBitPattern;

#[derive(Debug)]
pub struct InvalidNum;

/// The bmi160 returns the data from the gyro as an `i16`, we must use the Full Scale Range to
/// convert to a float.
pub const fn discrete_to_radians(fsr: GyroFsr, discrete: i16) -> f32 {
	discrete as f32 * fsr.rad_per_lsb()
}
