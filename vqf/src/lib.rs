//! This crate reimplements most of the relevant parts of the VQF algorithm from
//! <https://github.com/dlaidig/vqf/blob/f2a63375604e0b025048d181ba6a204e96ce2559/vqf/pyvqf.py>
//! Currently this is just copy-pasted from the python code, and some of the cpp code but it should
//! be made more idiomatic before actually using it. I have marked areas most likely to contain bugs
//! with ඞ
//!
//! The original code is licensed under the MIT license, so this crate is also licensed under the MIT license.
#![no_std]
#![allow(non_snake_case)] //The vqf given names are used
#![warn(missing_docs)]

use nalgebra::{ArrayStorage, U2, U9};
use num_traits::float::Float;

#[cfg(feature = "single-precision")]
use core::f32::consts::PI;
#[cfg(feature = "single-precision")]
/// Calculating resolution, use feature 'f64' for f64 calculations
#[allow(non_camel_case_types)]
pub type VQF_Real_T = f32;

#[cfg(not(feature = "single-precision"))]
use core::f64::consts::PI;
#[cfg(not(feature = "single-precision"))]
/// Calculating resolution, using feature 'f64' for f64 calculations
#[allow(non_camel_case_types)]
pub type VQF_Real_T = f64;

use core::f64::consts::PI as PI64;

/// A Quaternion type with capacity VQF_Real_T
pub type Quat = nalgebra::UnitQuaternion<VQF_Real_T>;
/// A 2 variable vector with type VQF_Real_T
pub type Vec2 = nalgebra::Vector2<VQF_Real_T>;
/// A 3 variable vector with type VQF_Real_T
pub type Vec3 = nalgebra::Vector3<VQF_Real_T>;
type Vec2Double = nalgebra::Vector2<f64>;
type Vec3Double = nalgebra::Vector3<f64>;
type Mat2x3Double = nalgebra::Matrix2x3<f64>;
type Mat2x9Double = nalgebra::Matrix<f64, U2, U9, ArrayStorage<f64, 2, 9>>;
type Mat2x2Double = nalgebra::Matrix2<f64>;
type Mat3x3 = nalgebra::Matrix3<VQF_Real_T>;

const EPS: VQF_Real_T = VQF_Real_T::EPSILON;

/// Constructor that initializes the struct with the default parameters.
#[derive(Clone)]
pub struct VqfParameters {
	/// Time constant for accelerometer low-pass filtering in seconds.
	/// Small values for imply trust on the accelerometer measurements and while large values of
	/// imply trust on the gyroscope measurements.
	///	Default value: 3.0 s
	pub tauAcc: VQF_Real_T,
	/// Time constant for magnetometer update in seconds.
	/// Small values for imply trust on the magnetometer measurements and while large values of
	/// imply trust on the gyroscope measurements.
	/// Default value: 9.0 s
	pub tauMag: VQF_Real_T,
	/// Enables gyroscope bias estimation during motion phases.
	/// If set to true (default), gyroscope bias is estimated based on the inclination correction only,
	/// i.e. without using magnetometer measurements.
	pub motionBiasEstEnabled: bool,
	/// Enables rest detection and gyroscope bias estimation during rest phases.
	/// If set to true (default), phases in which the IMU is at rest are detected. During rest,
	/// the gyroscope bias is estimated from the low-pass filtered gyroscope readings.
	pub restBiasEstEnabled: bool,
	/// Enables magnetic disturbance detection and magnetic disturbance rejection.
	/// If set to true (default), the magnetic field is analyzed. For short disturbed phases, the
	/// magnetometer-based correction is disabled totally. If the magnetic field is always regarded
	/// as disturbed or if the duration of the disturbances exceeds magMaxRejectionTime,
	/// magnetometer-based updates are performed,
	/// but with an increased time constant.
	pub magDistRejectionEnabled: bool,
	/// Standard deviation of the initial bias estimation uncertainty (in degrees per second).
	/// Default value: 0.5 °/s
	pub biasSigmaInit: VQF_Real_T,
	/// Time in which the bias estimation uncertainty increases from 0 °/s to 0.1 °/s (in seconds).
	/// This value determines the system noise assumed by the Kalman filter.
	/// Default value: 100.0 s
	pub biasForgettingTime: VQF_Real_T,
	/// Maximum expected gyroscope bias (in degrees per second).
	/// This value is used to clip the bias estimate and the measurement error in the bias estimationupdate step.
	/// It is further used by the rest detection algorithm in order to not regard measurements with a large but
	/// constant angular rate as rest.
	/// Default value: 2.0 °/s
	pub biasClip: VQF_Real_T,
	/// Standard deviation of the converged bias estimation uncertainty during motion (in degrees per second).
	/// This value determines the trust on motion bias estimation updates. A small value leads to fast convergence.
	/// Default value: 0.1 °/s
	pub biasSigmaMotion: VQF_Real_T,
	/// Forgetting factor for unobservable bias in vertical direction during motion.
	/// As magnetometer measurements are deliberately not used during motion bias estimation, gyroscope bias
	/// is not observable in vertical direction. This value is the relative weight of an artificial zero
	/// measurement that ensures that the bias estimate in the unobservable direction will eventually decay to zero.
	/// Default value: 0.0001
	pub biasVerticalForgettingFactor: VQF_Real_T,
	/// Standard deviation of the converged bias estimation uncertainty during rest (in degrees per second).
	/// This value determines the trust on rest bias estimation updates. A small value leads to fast convergence.
	/// Default value: 0.03 °
	pub biasSigmaRest: VQF_Real_T,
	/// Time threshold for rest detection (in seconds).
	/// Rest is detected when the measurements have been close to the low-pass filtered reference for the given time.
	/// Default value: 1.5 s
	pub restMinT: VQF_Real_T,
	/// Time constant for the low-pass filter used in rest detection (in seconds).
	/// This time constant characterizes a second-order Butterworth low-pass filter used to obtain the reference for rest detection.
	/// Default value: 0.5 s
	pub restFilterTau: VQF_Real_T,
	/// Angular velocity threshold for rest detection (in °/s).
	/// For rest to be detected, the norm of the deviation between measurement and reference must be below the given threshold.
	/// (Furthermore, the absolute value of each component must be below biasClip).
	/// Default value: 2.0 °/s
	pub restThGyr: VQF_Real_T,
	/// Acceleration threshold for rest detection (in m/s²).
	/// For rest to be detected, the norm of the deviation between measurement and reference must be below the given threshold.
	/// Default value: 0.5 m/s²
	pub restThAcc: VQF_Real_T,
	/// Time constant for current norm/dip value in magnetic disturbance detection (in seconds).
	/// This (very fast) low-pass filter is intended to provide additional robustness when the magnetometer measurements
	/// are noisy or not sampled perfectly in sync with the gyroscope measurements. Set to -1 to disable the low-pass filter
	/// and directly use the magnetometer measurements.
	/// Default value: 0.05 s
	pub magCurrentTau: VQF_Real_T,
	/// Time constant for the adjustment of the magnetic field reference (in seconds).
	/// This adjustment allows the reference estimate to converge to the observed undisturbed field.
	/// Default value: 20.0 s
	pub magRefTau: VQF_Real_T,
	/// Relative threshold for the magnetic field strength for magnetic disturbance detection.
	/// This value is relative to the reference norm.
	/// Default value: 0.1 (10%)
	pub magNormTh: VQF_Real_T,
	/// Threshold for the magnetic field dip angle for magnetic disturbance detection (in degrees).
	/// Default vaule: 10 °
	pub magDipTh: VQF_Real_T,
	/// Duration after which to accept a different homogeneous magnetic field (in seconds).
	/// A different magnetic field reference is accepted as the new field when the measurements are
	/// within the thresholds magNormTh and magDipTh for the given time. Additionally, only phases with
	/// sufficient movement, specified by magNewMinGyr, count.
	/// Default value: 20.0
	pub magNewTime: VQF_Real_T,
	/// Duration after which to accept a homogeneous magnetic field for the first time (in seconds).
	/// This value is used instead of magNewTime when there is no current estimate in order to allow
	/// for the initial magnetic field reference to be obtained faster.
	/// Default value: 5.0
	pub magNewFirstTime: VQF_Real_T,
	/// Minimum angular velocity needed in order to count time for new magnetic field acceptance (in °/s).
	/// Durations for which the angular velocity norm is below this threshold do not count towards reaching magNewTime.
	/// Default value: 20.0 °/s
	pub magNewMinGyr: VQF_Real_T,
	/// Minimum duration within thresholds after which to regard the field as undisturbed again (in seconds).
	/// Default value: 0.5 s
	pub magMinUndisturbedTime: VQF_Real_T,
	/// Maximum duration of full magnetic disturbance rejection (in seconds).
	/// For magnetic disturbances up to this duration, heading correction is fully disabled and heading
	/// changes are tracked by gyroscope only. After this duration (or for many small disturbed phases
	/// without sufficient time in the undisturbed field in between), the heading correction is performed
	/// with an increased time constant (see magRejectionFactor).
	/// Default value: 60.0 s
	pub magMaxRejectionTime: VQF_Real_T,
	/// Factor by which to slow the heading correction during long disturbed phases.
	/// After magMaxRejectionTime of full magnetic disturbance rejection, heading correction is performed
	/// with an increased time constant. This parameter (approximately) specifies the factor of the increase.
	/// Furthermore, after spending magMaxRejectionTime/magRejectionFactor seconds in an undisturbed magnetic
	/// field, the time is reset and full magnetic disturbance rejection will be performed for up to magMaxRejectionTime again.
	/// Default value: 2.0
	pub magRejectionFactor: VQF_Real_T,
}

impl Default for VqfParameters {
	fn default() -> Self {
		VqfParameters {
			tauAcc: 3.0,
			tauMag: 9.0,
			motionBiasEstEnabled: true,
			restBiasEstEnabled: true,
			magDistRejectionEnabled: true,
			biasSigmaInit: 0.5,
			biasForgettingTime: 100.0,
			biasClip: 2.0,
			biasSigmaMotion: 0.1,
			biasVerticalForgettingFactor: 0.0001,
			biasSigmaRest: 0.03,
			restMinT: 1.5,
			restFilterTau: 0.5,
			restThGyr: 2.0,
			restThAcc: 0.5,
			magCurrentTau: 0.05,
			magRefTau: 20.0,
			magNormTh: 0.1,
			magDipTh: 10.0,
			magNewTime: 20.0,
			magNewFirstTime: 5.0,
			magNewMinGyr: 20.0,
			magMinUndisturbedTime: 0.5,
			magMaxRejectionTime: 60.0,
			magRejectionFactor: 2.0,
		}
	}
}

/// Struct containing the filter state of the VQF class.
/// The relevant parts of the state can be accessed via functions of the VQF class, e.g.
/// VQF::getQuat6D(), VQF::getQuat9D(), VQF::getGyrBiasEstimate(), VQF::setGyrBiasEstimate(),
/// VQF::getRestDetected() and VQF::getMagDistDetected().
/// To reset the state to the initial values, use VQF::resetState().

/// Direct access to the full state is typically not needed but can be useful in some cases,
/// e.g. for debugging. For this purpose, the state can be accessed by VQF::getState() and set by VQF::setState().
#[derive(Clone)]
pub struct VqfState {
	/// Angular velocity strapdown integration quaternion
	pub gyrQuat: Quat,
	/// Inclination correction quaternion
	pub accQuat: Quat,
	/// Heading difference δ between ϵi and ϵ
	pub delta: VQF_Real_T,
	/// True if it has been detected that the IMU is currently at rest.
	/// Used to switch between rest and motion gyroscope bias estimation
	pub restDetected: bool,
	/// True if magnetic disturbances have been detected.
	pub magDistDetected: bool,
	/// Last low-pass filtered acceleration in the Ii frame
	pub lastAccLp: Vec3,
	/// Internal low-pass filter state for lastAccLp
	pub accLpState: Mat2x3Double,
	/// Last inclination correction angular rate.
	/// Change to inclination correction quaternion
	/// performed in the last accelerometer update, expressed as an angular rate (in rad/s).
	pub lastAccCorrAngularRate: VQF_Real_T,
	/// Gain used for heading correction to ensure fast initial convergence.
	/// This value is used as the gain for heading correction in the beginning if it is
	/// larger than the normal filter gain. It is initialized to 1 and then updated to
	/// 0.5, 0.33, 0.25, … After VQFParams::tauMag seconds, it is set to zero.
	pub kMagInit: VQF_Real_T,
	/// Last heading disagreement angle. Disagreement between the heading δ^
	/// estimated from the last magnetometer sample and the state δ (in rad)
	pub lastMagDisAngle: VQF_Real_T,
	/// Last heading correction angular rate. Change to heading
	/// performed in the last magnetometer update, expressed as an angular rate δ (in rad/s).
	pub lastMagCorrAngularRate: VQF_Real_T,
	/// Current gyroscope bias estimate (in rad/s).
	pub bias: Vec3,
	/// Covariance matrix of the gyroscope bias estimate. The 3x3 matrix is stored in row-major order.
	/// Note that for numeric reasons the internal unit used is 0.01 °/s, i.e.
	/// to get the standard deviation in degrees per second use (equation in vqf docs)
	pub biasP: Mat3x3,
	/// Internal state of the Butterworth low-pass filter for the rotation matrix
	/// coefficients used in motion bias estimation.
	pub motionBiasEstRLpState: Mat2x9Double,
	/// Internal low-pass filter state for the rotated bias estimate used in motion bias estimation.
	pub motionBiasEstBiasLpState: Mat2x2Double,
	/// Last (squared) deviations from the reference of the last sample used in rest detection.
	/// Looking at those values can be useful to understand how rest detection is working and which
	/// thresholds are suitable. The array contains the last values for gyroscope and accelerometer
	/// in the respective units. Note that the values are squared.
	/// The method VQF::getRelativeRestDeviations() provides an easier way to obtain and interpret those values.
	pub restLastSquaredDeviations: Vec2,
	/// The current duration for which all sensor readings are within the rest detection thresholds.
	/// Rest is detected if this value is larger or equal to VQFParams::restMinT
	pub restT: VQF_Real_T,
	/// Last low-pass filtered gyroscope measurement used as the reference for rest detection.
	/// Note that this value is also used for gyroscope bias estimation when rest is detected.
	pub restLastGyrLp: Vec3,
	/// Internal low-pass filter state for restLastGyrLp.
	pub restGyrLpState: Mat2x3Double,
	/// Last low-pass filtered accelerometer measurement used as the reference for rest detection.
	pub restLastAccLp: Vec3,
	/// Internal low-pass filter state for restLastAccLp.
	pub restAccLpState: Mat2x3Double,
	/// Norm of the currently accepted magnetic field reference.
	/// A value of -1 indicates that no homogeneous field is found yet.
	pub magRefNorm: VQF_Real_T,
	/// Dip angle of the currently accepted magnetic field reference.
	pub magRefDip: VQF_Real_T,
	/// The current duration for which the current norm and dip are close to the reference.
	/// The magnetic field is regarded as undisturbed when this value reaches VQFParams::magMinUndisturbedTime.
	pub magUndisturbedT: VQF_Real_T,
	/// The current duration for which the magnetic field was rejected.
	/// If the magnetic field is disturbed and this value is smaller than VQFParams::magMaxRejectionTime, heading correction updates are fully disabled.
	pub magRejectT: VQF_Real_T,
	/// Norm of the alternative magnetic field reference currently being evaluated.
	pub magCandidateNorm: VQF_Real_T,
	/// Dip angle of the alternative magnetic field reference currently being evaluated.
	pub magCandidateDip: VQF_Real_T,
	/// The current duration for which the norm and dip are close to the candidate.
	/// If this value exceeds VQFParams::magNewTime (or VQFParams::magNewFirstTime if magRefNorm < 0),
	/// the current candidate is accepted as the new reference.
	pub magCandidateT: VQF_Real_T,
	/// Norm and dip angle of the current magnetometer measurements.
	/// Slightly low-pass filtered, see VQFParams::magCurrentTau.
	pub magNormDip: Vec2,
	/// Internal low-pass filter state for the current norm and dip angle.
	pub magNormDipLpState: Mat2x2Double,
}

impl Default for VqfState {
	fn default() -> VqfState {
		VqfState {
			gyrQuat: Quat::identity(),
			accQuat: Quat::identity(),
			delta: 0.0,
			restDetected: false,
			magDistDetected: true,
			lastAccLp: Vec3::zeros(),
			accLpState: Mat2x3Double::repeat(f64::NAN),
			lastAccCorrAngularRate: 0.0,
			kMagInit: 1.0,
			lastMagDisAngle: 0.0,
			lastMagCorrAngularRate: 0.0,
			bias: Vec3::zeros(),
			biasP: Mat3x3::repeat(VQF_Real_T::NAN),
			motionBiasEstRLpState: Mat2x9Double::repeat(f64::NAN),
			motionBiasEstBiasLpState: Mat2x2Double::repeat(f64::NAN),
			restLastSquaredDeviations: Vec2::zeros(),
			restT: 0.0,
			restLastGyrLp: Vec3::zeros(),
			restGyrLpState: Mat2x3Double::repeat(f64::NAN),
			restLastAccLp: Vec3::zeros(),
			restAccLpState: Mat2x3Double::repeat(f64::NAN),
			magRefNorm: 0.0,
			magRefDip: 0.0,
			magUndisturbedT: 0.0,
			magRejectT: -1.0,
			magCandidateNorm: -1.0,
			magCandidateDip: 0.0,
			magCandidateT: 0.0,
			magNormDip: Vec2::zeros(),
			magNormDipLpState: Mat2x2Double::repeat(f64::NAN),
		}
	}
}

/// Coefficients are values that depend on the parameters and the sampling times,
/// but do not change during update steps. They are calculated in VQF::setup().
#[derive(Clone)]
pub struct VQFCoefficients {
	/// Sampling time of the gyroscope measurements (in seconds).
	pub gyrTs: VQF_Real_T,
	/// Sampling time of the accelerometer measurements (in seconds).
	pub accTs: VQF_Real_T,
	/// Sampling time of the magnetometer measurements (in seconds).
	pub magTs: VQF_Real_T,
	/// Numerator coefficients of the acceleration low-pass filter.
	/// The array contains [b0, b1, b2]
	pub accLpB: Vec3Double,
	/// Denominator coefficients of the acceleration low-pass filter.
	/// The array contains [a1, a2] and a0 = 1
	pub accLpA: Vec2Double,
	/// Gain of the first-order filter used for heading correction.
	pub kMag: VQF_Real_T,
	/// Variance of the initial gyroscope bias estimate.
	pub biasP0: VQF_Real_T,
	/// System noise variance used in gyroscope bias estimation.
	pub biasV: VQF_Real_T,
	/// Measurement noise variance for the motion gyroscope bias estimation update.
	pub biasMotionW: VQF_Real_T,
	/// Measurement noise variance for the motion gyroscope bias estimation update in vertical direction.
	pub biasVerticalW: VQF_Real_T,
	/// Measurement noise variance for the rest gyroscope bias estimation update.
	pub biasRestW: VQF_Real_T,
	/// Numerator coefficients of the gyroscope measurement low-pass filter for rest detection.
	pub restGyrLpB: Vec3Double,
	/// Denominator coefficients of the gyroscope measurement low-pass filter for rest detection.
	pub restGyrLpA: Vec2Double,
	/// Numerator coefficients of the accelerometer measurement low-pass filter for rest detection.
	pub restAccLpB: Vec3Double,
	/// Denominator coefficients of the accelerometer measurement low-pass filter for rest detection.
	pub restAccLpA: Vec2Double,
	/// Gain of the first-order filter used for to update the magnetic field reference and candidate.
	pub kMagRef: VQF_Real_T,
	/// Numerator coefficients of the low-pass filter for the current magnetic norm and dip.
	pub magNormDipLpB: Vec3Double,
	/// Denominator coefficients of the low-pass filter for the current magnetic norm and dip.
	pub magNormDipLpA: Vec2Double,
}

impl Default for VQFCoefficients {
	fn default() -> Self {
		Self {
			gyrTs: 0.0,
			accTs: 0.0,
			magTs: 0.0,
			accLpB: Vec3Double::repeat(f64::NAN),
			accLpA: Vec2Double::repeat(f64::NAN),
			kMag: -1.0,
			biasP0: -1.0,
			biasV: -1.0,
			biasMotionW: -1.0,
			biasVerticalW: -1.0,
			biasRestW: -1.0,
			restGyrLpB: Vec3Double::repeat(f64::NAN),
			restGyrLpA: Vec2Double::repeat(f64::NAN),
			restAccLpB: Vec3Double::repeat(f64::NAN),
			restAccLpA: Vec2Double::repeat(f64::NAN),
			kMagRef: -1.0,
			magNormDipLpB: Vec3Double::repeat(f64::NAN),
			magNormDipLpA: Vec2Double::repeat(f64::NAN),
		}
	}
}

/// Internally used struct to manage vqf
pub struct Vqf {
	_params: VqfParameters,
	_state: VqfState,
	_coeffs: VQFCoefficients,
}

impl Vqf {
	/// Used to create a new instance of VQF
	pub fn new(
		gyrTs: VQF_Real_T,
		accTs: VQF_Real_T,
		magTs: VQF_Real_T,
		params: VqfParameters,
	) -> Vqf {
		let mut coeffs: VQFCoefficients = Default::default();
		assert!(gyrTs > 0.0);
		assert!(accTs > 0.0);
		assert!(magTs > 0.0);

		coeffs.gyrTs = gyrTs;
		coeffs.accTs = accTs;
		coeffs.magTs = magTs;

		(coeffs.accLpB, coeffs.accLpA) = filterCoeffs(params.tauAcc, coeffs.accTs);
		coeffs.kMag = gainFromTau(params.tauMag, coeffs.magTs);

		coeffs.biasP0 = (params.biasSigmaInit * 100.0).powi(2);

		// the system noise increases the variance from 0 to (0.1 °/s)^2 in biasForgettingTime seconds
		coeffs.biasV = (0.1 * 100.0).powi(2) * coeffs.accTs / params.biasForgettingTime;

		let pMotion = (params.biasSigmaMotion * 100.0).powi(2);
		coeffs.biasMotionW = (pMotion).powi(2) / coeffs.biasV + pMotion;
		coeffs.biasVerticalW =
			coeffs.biasMotionW / params.biasVerticalForgettingFactor.max(1e-10);

		let pRest = (params.biasSigmaRest * 100.0).powi(2);
		coeffs.biasRestW = (pRest).powi(2) / coeffs.biasV + pRest;

		(coeffs.restGyrLpB, coeffs.restGyrLpA) =
			filterCoeffs(params.restFilterTau, coeffs.gyrTs);
		(coeffs.restAccLpB, coeffs.restAccLpA) =
			filterCoeffs(params.restFilterTau, coeffs.accTs);

		coeffs.kMagRef = gainFromTau(params.magRefTau, coeffs.magTs);
		if params.magCurrentTau > 0.0 {
			(coeffs.magNormDipLpB, coeffs.magNormDipLpA) =
				filterCoeffs(params.magCurrentTau, coeffs.magTs);
		}

		let mut vqf = Vqf {
			_params: params,
			_state: Default::default(),
			_coeffs: coeffs,
		};
		vqf.resetState();
		vqf
	}

	/// Preforms gyroscope update step
	/// It is only necessary to call this function directly if gyroscope, accelerometers
	/// and magnetometers have different smpling rates. Otherwise simply use update()
	/// Parameters: gyr - gyroscope measument in rad/s
	pub fn updateGyr(&mut self, gyr: Vec3) {
		// rest detection
		if self._params.restBiasEstEnabled || self._params.magDistRejectionEnabled {
			let gyrLp = filterVec(
				gyr,
				self._params.restFilterTau,
				self._coeffs.gyrTs,
				self._coeffs.restGyrLpB,
				self._coeffs.restGyrLpA,
				&mut self._state.restGyrLpState,
			);

			let deviation = gyr - gyrLp;
			let squaredDeviation = deviation.dot(&deviation);

			let biasClip = self._params.biasClip * PI / 180.0;
			if squaredDeviation >= (self._params.restThGyr * PI / 180.0).powf(2.0)
				|| gyrLp.abs().max() > biasClip
			{
				self._state.restT = 0.0;
				self._state.restDetected = false;
			}
			self._state.restLastGyrLp = gyrLp;
			self._state.restLastSquaredDeviations[0] = squaredDeviation;
		}

		// remove estimated gyro bias
		let gyrNoBias = gyr - self._state.bias;

		// gyroscope prediction step
		let gyrNorm = gyrNoBias.dot(&gyrNoBias).sqrt();
		let angle = gyrNorm * self._coeffs.gyrTs;
		if gyrNorm > EPS {
			let c = (angle / 2.0).cos();
			let s = (angle / 2.0).sin() / gyrNorm;
			let gyrStepQuat = Quat::from_quaternion(
				[s * gyrNoBias[0], s * gyrNoBias[1], s * gyrNoBias[2], c].into(),
			);
			self._state.gyrQuat = self._state.gyrQuat * gyrStepQuat;
			self._state.gyrQuat.renormalize_fast();
		}
	}

	/// Preforms accelerometer update step
	/// It is only necessary to call this function directly if gyroscope, accelerometers
	/// and magnetometers have different smpling rates. Otherwise simply use update()
	/// Paramters: acc - accelerometer measurement in m/s²
	pub fn updateAcc(&mut self, acc: Vec3) {
		if acc == Vec3::zeros() {
			return;
		}

		let accTs = self._coeffs.accTs;

		// Rest detection
		if self._params.restBiasEstEnabled {
			let accLp = filterVec(
				acc,
				self._params.restFilterTau,
				accTs,
				self._coeffs.restAccLpB,
				self._coeffs.restAccLpA,
				&mut self._state.restAccLpState,
			);

			let deviation = acc - accLp;
			let squaredDeviation = deviation.dot(&deviation);

			if squaredDeviation >= self._params.restThAcc.powf(2.0) {
				self._state.restT = 0.0;
				self._state.restDetected = false;
			} else {
				self._state.restT += accTs;
				if self._state.restT >= self._params.restMinT {
					self._state.restDetected = true
				}
			}

			self._state.restLastAccLp = accLp;
			self._state.restLastSquaredDeviations[1] = squaredDeviation
		}

		// filter acc in inertial frame
		let accEarth = self._state.gyrQuat * acc;
		self._state.lastAccLp = filterVec(
			accEarth,
			self._params.tauAcc,
			accTs,
			self._coeffs.accLpB,
			self._coeffs.accLpA,
			&mut self._state.accLpState,
		);

		// transform to 6D earth frame and normalize
		let accEarth = (self._state.accQuat * self._state.lastAccLp).normalize();

		// inclination correction
		let q_w = ((accEarth[2] + 1.0) / 2.0).sqrt();
		let accCorrQuat;
		if q_w > 1e-6 {
			accCorrQuat = Quat::from_quaternion(
				[0.5 * accEarth[1] / q_w, -0.5 * accEarth[0] / q_w, 0.0, q_w].into(),
			);
		} else {
			accCorrQuat = Quat::from_quaternion([1.0, 0.0, 0.0, 0.0].into());
		}
		self._state.accQuat = accCorrQuat * self._state.accQuat;
		self._state.accQuat.renormalize_fast();

		// calculate correction angular rate to facilitate debugging
		self._state.lastAccCorrAngularRate = (accEarth[2]).acos() / self._coeffs.accTs;

		// bias estimation
		if self._params.motionBiasEstEnabled || self._params.restBiasEstEnabled {
			let biasClip = self._params.biasClip * PI / 180.0;
			let mut bias = self._state.bias;

			// get rotation matrix corresponding to accGyrQuat
			let accGyrQuat = self.getQuat6D();
			// ඞ but works
			let accGyrQuat = Quat::from_quaternion(
				[
					-accGyrQuat.coords.x,
					-accGyrQuat.coords.y,
					accGyrQuat.coords.z,
					accGyrQuat.coords.w,
				]
				.into(),
			);
			let R = accGyrQuat.to_rotation_matrix().into_inner();

			// calculate R*b_hat (only the x and y component, as z is not needed)
			let biasLp = (R * bias).xy();

			// low-pass filter R and R*b_hat
			let mut R = filterVec(
				R.reshape_generic(nalgebra::Const::<9>, nalgebra::Const::<1>),
				self._params.tauAcc,
				accTs,
				self._coeffs.accLpB,
				self._coeffs.accLpA,
				&mut self._state.motionBiasEstRLpState,
			)
			.reshape_generic(nalgebra::Const::<3>, nalgebra::Const::<3>);
			let biasLp = filterVec(
				biasLp,
				self._params.tauAcc,
				accTs,
				self._coeffs.accLpB,
				self._coeffs.accLpA,
				&mut self._state.motionBiasEstBiasLpState,
			);

			// set measurement error and covariance for the respective Kalman filter update
			let e;
			let w;
			if self._state.restDetected && self._params.restBiasEstEnabled {
				e = Some(self._state.restLastGyrLp - bias);
				R = Mat3x3::identity();
				w = Some(Vec3::repeat(self._coeffs.biasRestW));
			} else if self._params.motionBiasEstEnabled {
				// ඞ
				e = Some(Vec3::new(
					-accEarth[1] / accTs + biasLp[0]
						- R[0] * bias[0] - R[1] * bias[1]
						- R[2] * bias[2],
					accEarth[0] / accTs + biasLp[1]
						- R[3] * bias[0] - R[4] * bias[1]
						- R[5] * bias[2],
					-R[6] * bias[0] - R[7] * bias[1] - R[8] * bias[2],
				));
				w = Some(Vec3::new(
					self._coeffs.biasMotionW,
					self._coeffs.biasMotionW,
					self._coeffs.biasVerticalW,
				));
			} else {
				w = None;
				e = None;
			}

			// Kalman filter update
			// step 1: P = P + V (also increase covariance if there is no measurement update!)
			if self._state.biasP[(0, 0)] < self._coeffs.biasP0 {
				self._state.biasP[(0, 0)] += self._coeffs.biasV;
			}
			if self._state.biasP[(1, 1)] < self._coeffs.biasP0 {
				self._state.biasP[(1, 1)] += self._coeffs.biasV;
			}
			if self._state.biasP[(2, 2)] < self._coeffs.biasP0 {
				self._state.biasP[(2, 2)] += self._coeffs.biasV;
			}

			if let Some(w) = w {
				// clip disagreement to -2..2 °/s
				// (this also effectively limits the harm done by the first inclination correction step)
				let e = e.unwrap();
				let e = Vec3::from_fn(|x, y| e[(x, y)].clamp(-biasClip, biasClip));

				// step 2: K = P R^T inv(W + R P R^T)
				let mut K = self._state.biasP
					* R.transpose() * (Mat3x3::from_diagonal(&w)
					+ R * self._state.biasP * R.transpose())
				.pseudo_inverse(EPS)
				.unwrap();

				// ඞ why are the signs messed up also K is transposed but works for the math
				K[(0, 2)] *= -1.0;
				K[(1, 2)] *= -1.0;
				K[(2, 0)] *= -1.0;
				K[(2, 1)] *= -1.0;

				// step 3: bias = bias + K (y - R bias) = bias + K e
				bias += K * e;

				// step 4: P = P - K R P
				let mut biasP_change = K * R * self._state.biasP;
				// ඞ why are the signs messed up again
				biasP_change[(0, 2)] *= -1.0;
				biasP_change[(1, 2)] *= -1.0;
				biasP_change[(2, 2)] *= -1.0;

				self._state.biasP -= biasP_change;

				// clip bias estimate to -2..2 °/s
				bias = Vec3::from_fn(|x, y| bias[(x, y)].clamp(-biasClip, biasClip));
			}

			// ඞ
			self._state.bias = bias;
		}
	}

	/// Preforms magnetometer update step
	/// It is only necessary to call this function directly if gyroscope, accelerometers
	/// and magnetometers have different smpling rates. Otherwise simply use update()
	/// Parameters: mag - magnetometer measurement in arbitrary units
	pub fn updateMag(&mut self, mag: Vec3) {
		if mag == Vec3::zeros() {
			return;
		}

		let magTs = self._coeffs.magTs;

		// bring magnetometer measurement into 6D earth frame
		let magEarth = self.getQuat6D() * mag;

		if self._params.magDistRejectionEnabled {
			let mut magNormDip = self._state.magNormDip;
			magNormDip[0] = (magEarth.dot(&magEarth)).sqrt();
			magNormDip[1] = -((magEarth[2] / magNormDip[0]).asin());

			if self._params.magCurrentTau > 0.0 {
				magNormDip = filterVec(
					magNormDip,
					self._params.magCurrentTau,
					magTs,
					self._coeffs.magNormDipLpB,
					self._coeffs.magNormDipLpA,
					&mut self._state.magNormDipLpState,
				);
			}

			// magnetic disturbance detection
			if (magNormDip[0] - self._state.magRefNorm).abs()
				< self._params.magNormTh * self._state.magRefNorm
				&& (magNormDip[1] - self._state.magRefDip).abs()
					< self._params.magDipTh * PI / 180.0
			{
				self._state.magUndisturbedT += magTs;

				if self._state.magUndisturbedT >= self._params.magMinUndisturbedTime {
					self._state.magDistDetected = false;
					self._state.magRefNorm +=
						self._coeffs.kMagRef * (magNormDip[0] - self._state.magRefNorm);
					self._state.magRefDip +=
						self._coeffs.kMagRef * (magNormDip[1] - self._state.magRefDip);
				}
			} else {
				self._state.magUndisturbedT = 0.0;
				self._state.magDistDetected = true;
			}

			// new magnetic field acceptance
			if (magNormDip[0] - self._state.magCandidateNorm).abs()
				< self._params.magNormTh * self._state.magCandidateNorm
				&& (magNormDip[1] - self._state.magCandidateDip).abs()
					< self._params.magDipTh * PI / 180.0
			{
				let gyrNorm =
					(self._state.restLastGyrLp.dot(&self._state.restLastGyrLp)).sqrt();
				if gyrNorm >= self._params.magNewMinGyr * PI / 180.0 {
					self._state.magCandidateT += magTs;
				}

				self._state.magCandidateNorm += self._coeffs.kMagRef
					* (magNormDip[0] - self._state.magCandidateNorm);
				self._state.magCandidateDip += self._coeffs.kMagRef
					* (magNormDip[1] - self._state.magCandidateDip);

				if self._state.magDistDetected
					&& (self._state.magCandidateT >= self._params.magNewTime
						|| (self._state.magRefNorm == 0.0
							&& self._state.magCandidateT
								>= self._params.magNewFirstTime))
				{
					self._state.magRefNorm = self._state.magCandidateNorm;
					self._state.magRefDip = self._state.magCandidateDip;
					self._state.magDistDetected = false;
					self._state.magUndisturbedT = self._params.magMinUndisturbedTime;
				}
			} else {
				self._state.magCandidateT = 0.0;
				self._state.magCandidateNorm = magNormDip[0];
				self._state.magCandidateDip = magNormDip[1];
			}
		}

		// calculate disagreement angle based on current magnetometer measurement
		self._state.lastMagDisAngle =
			magEarth[0].atan2(magEarth[1]) - self._state.delta;

		// make sure the disagreement angle is in the range [-pi, pi]
		if self._state.lastMagDisAngle > PI {
			self._state.lastMagDisAngle -= 2.0 * PI;
		} else if self._state.lastMagDisAngle < -PI {
			self._state.lastMagDisAngle += 2.0 * PI;
		}

		let mut k = self._coeffs.kMag;

		if self._params.magDistRejectionEnabled {
			// magnetic disturbance rejection
			if self._state.magDistDetected {
				if self._state.magRejectT <= self._params.magMaxRejectionTime {
					self._state.magRejectT += magTs;
					k = 0.0;
				} else {
					k /= self._params.magRejectionFactor;
				}
			} else {
				self._state.magRejectT = (self._state.magRejectT
					- self._params.magRejectionFactor * magTs)
					.max(0.0);
			}
		}

		// ensure fast initial convergence
		if self._state.kMagInit != 0.0 {
			// make sure that the gain k is at least 1/N, N=1,2,3,... in the first few samples
			if k < self._state.kMagInit {
				k = self._state.kMagInit;
			}

			// iterative expression to calculate 1/N
			self._state.kMagInit = self._state.kMagInit / (self._state.kMagInit + 1.0);

			// disable if t > tauMag
			if self._state.kMagInit * self._params.tauMag < self._coeffs.magTs {
				self._state.kMagInit = 0.0;
			}
		}

		// first-order filter step
		self._state.delta += k * self._state.lastMagDisAngle;
		// calculate correction angular rate to facilitate debugging
		self._state.lastMagCorrAngularRate =
			k * self._state.lastMagDisAngle / self._coeffs.magTs;

		// make sure delta is in the range [-pi, pi]
		if self._state.delta > PI {
			self._state.delta -= 2.0 * PI;
		} else if self._state.delta < -PI {
			self._state.delta += 2.0 * PI;
		}
	}

	/// Bulk update the gyro, accel and optionally the mag
	pub fn update(&mut self, gyr: Vec3, acc: Vec3, mag: Option<Vec3>) {
		self.updateGyr(gyr);
		self.updateAcc(acc);
		if let Some(mag) = mag {
			self.updateMag(mag);
		}
	}

	/// Returns the angular velocity strapdown integration quaternion
	pub fn getQuat3D(&self) -> Quat {
		self._state.gyrQuat
	}

	/// Returns the 6D (magnetometer-free) orientation quaternion
	pub fn getQuat6D(&self) -> Quat {
		self._state.accQuat * self.getQuat3D()
	}

	/// Returns the 9D (with magnetometers) orientation quaternion
	pub fn getQuat9D(&self) -> Quat {
		quatApplyDelta(self.getQuat6D(), self._state.delta)
	}

	/// Returns the heading difference δ between ϵi and ϵ in rad
	pub fn getDelta(&self) -> VQF_Real_T {
		self._state.delta
	}

	/// Returns the current gyroscope bias estimate and the uncetainty
	/// The returned standard deviation sigma represents the estimation uncertainty
	/// in the worst direction and is based on an upper bound of the largest
	/// eigenvalue of the covariance matrix
	/// VQF_Real_T - standard deviation sigma of the estimation uncertainty (rad/s)
	/// Vec3 - output array for the gyroscope bias estimate (rad/s)
	pub fn getBiasEstimate(&self) -> (VQF_Real_T, Vec3) {
		let sum1 = self._state.biasP[(0, 0)].abs()
			+ self._state.biasP[(0, 1)].abs()
			+ self._state.biasP[(0, 2)].abs();
		let sum2 = self._state.biasP[(1, 0)].abs()
			+ self._state.biasP[(1, 1)].abs()
			+ self._state.biasP[(1, 2)].abs();
		let sum3 = self._state.biasP[(2, 0)].abs()
			+ self._state.biasP[(2, 1)].abs()
			+ self._state.biasP[(2, 2)].abs();

		let max: Vec3 = [sum1, sum2, sum3].into();
		let p = max.max().min(self._coeffs.biasP0);

		let std_dev = p.sqrt() * (PI / 100.0 / 180.0);
		(std_dev, self._state.bias)
	}

	/// Sets the current gyroscope bias estimate and the uncertainty
	/// If a value for the uncertainty sigma is given, the covariance
	/// matrix is set to a corresponding scale identity matrix
	/// bias - gyroscope bias estimate (rad/s)
	/// sigma - standard deviation of the estimation uncertainty (rad/s)
	///       - set to -1 (default) in order to not change the estimation
	///       - covariance matrix
	pub fn setBiasEstimate(&self, _bias: Vec3, _sigma: Option<VQF_Real_T>) {
		//if sigma none then = -1.0;
		todo!();
	}

	/// Returns true if rest was detected
	pub fn getRestDetected(&self) -> bool {
		self._state.restDetected
	}

	/// Returns true if a disturbed magnetic field was detected
	pub fn getMagDistDetected(&self) -> bool {
		self._state.magDistDetected
	}

	/// Returns the relative deviations used in rest detection
	/// Looking at those values can be useful to understand how rest detection is
	/// working and which thresholds are suitable. The output array is fulled with
	/// the last values for gyroscope and accelerometer, relative to the threshold.
	/// In order for rest to be detected, both values must stay below 1
	/// Returns - relative rest deviations
	pub fn getRelativeRestDeviations(&self) -> Vec2 {
		todo!();
	}

	/// Returns the norm of the currently accepted magnetic field reference
	pub fn getMagRefNorm(&self) -> VQF_Real_T {
		self._state.magRefNorm
	}

	/// Returns the dip angle of the currently accepted magnetic field reference
	pub fn getMagRefDip(&self) -> VQF_Real_T {
		self._state.magRefDip
	}

	/// Overwrites the current magnetic field reference
	pub fn setMagRef(mut self, norm: VQF_Real_T, dip: VQF_Real_T) {
		self._state.magRefNorm = norm;
		self._state.magRefDip = dip;
	}

	/// Sets the time constant for accelerometer low-pass filtering (in seconds)
	pub fn setTauAcc(mut self, tauAcc: VQF_Real_T) {
		self._params.tauAcc = tauAcc;
		todo!();
	}

	/// Sets the time constant for the magnetometer update (in seconds)
	pub fn setTauMag(mut self, tauMag: VQF_Real_T) {
		self._params.tauMag = tauMag;
		todo!();
	}

	/// Enables/disables gyroscope bias estimation during motion
	pub fn setMotionBiasEstEnabled(mut self, enabled: bool) {
		if self._params.motionBiasEstEnabled == enabled {
			return;
		}
		self._params.motionBiasEstEnabled = enabled;
		self._state.motionBiasEstRLpState = Mat2x9Double::repeat(f64::NAN);
		self._state.motionBiasEstBiasLpState = Mat2x2Double::repeat(f64::NAN);
	}

	/// Enables/disables rest detection and bias estimation during rest
	pub fn setRestBiasEstEnabled(mut self, enabled: bool) {
		if self._params.restBiasEstEnabled == enabled {
			return;
		}
		self._params.restBiasEstEnabled = enabled;
		self._state.restDetected = false;
		self._state.restLastSquaredDeviations = Vec2::zeros();
		self._state.restT = 0.0;
		self._state.restLastGyrLp = Vec3::zeros();
		self._state.restGyrLpState = Mat2x3Double::repeat(f64::NAN);
		self._state.restLastAccLp = Vec3::zeros();
		self._state.restAccLpState = Mat2x3Double::repeat(f64::NAN);
	}

	/// Enables/disables magnetic disturbance detection and rejection
	pub fn setMagDistRejectionEnabled(mut self, enabled: bool) {
		if self._params.magDistRejectionEnabled == enabled {
			return;
		}
		self._params.magDistRejectionEnabled = enabled;
		self._state.magDistDetected = true;
		self._state.magRefNorm = 0.0;
		self._state.magRefDip = 0.0;
		self._state.magUndisturbedT = 0.0;
		self._state.magRejectT = self._params.magMaxRejectionTime;
		self._state.magCandidateNorm = -1.0;
		self._state.magCandidateDip = 0.0;
		self._state.magCandidateT = 0.0;
		self._state.magNormDipLpState = Mat2x2Double::repeat(f64::NAN);
	}

	/// Sets the current thresholds for rest detection
	pub fn setRestDetectionThresholds(mut self, thGyr: VQF_Real_T, thAcc: VQF_Real_T) {
		self._params.restThGyr = thGyr;
		self._params.restThAcc = thAcc;
	}

	/// Returns the current parameters (read only)
	pub fn getParams(&self) -> VqfParameters {
		self._params.clone()
	}

	/// Returns the coefficients used by the algorithm (read only)
	#[cfg(feature = "debug-state")]
	pub fn getCoeffs(&self) -> VQFCoefficients {
		self._coeffs.clone()
	}

	/// Returns the current state (read only)
	#[cfg(feature = "debug-state")]
	pub fn getState(&self) -> VqfState {
		self._state.clone()
	}

	/// Overwrites the current state
	/// This method allows to set a completely arbitrary filter state and
	/// is intended for debugging purposes. In combination with getState()
	/// individual elements of the state can be modified
	#[cfg(feature = "debug-state")]
	pub fn setState(mut self, state: VqfState) {
		self._state = state;
	}

	/// Resets the state to the defualt values at initialization
	/// Resetting the state is equivalent to creating a new instance of this class
	pub fn resetState(&mut self) {
		self._state.gyrQuat = Quat::identity();
		self._state.accQuat = Quat::identity();
		self._state.delta = 0.0;

		self._state.restDetected = false;
		self._state.magDistDetected = true;

		self._state.lastAccLp = Vec3::zeros();
		self._state.accLpState = Mat2x3Double::repeat(f64::NAN);
		self._state.lastAccCorrAngularRate = 0.0;

		self._state.kMagInit = 1.0;
		self._state.lastMagDisAngle = 0.0;
		self._state.lastMagCorrAngularRate = 0.0;

		self._state.bias = Vec3::zeros();
		self._state.biasP = Mat3x3::identity() * self._coeffs.biasP0;
		self._state.motionBiasEstRLpState = Mat2x9Double::repeat(f64::NAN);
		self._state.motionBiasEstBiasLpState = Mat2x2Double::repeat(f64::NAN);

		self._state.restLastSquaredDeviations = Vec2::zeros();
		self._state.restT = 0.0;
		self._state.restLastGyrLp = Vec3::zeros();
		self._state.restGyrLpState = Mat2x3Double::repeat(f64::NAN);
		self._state.restLastAccLp = Vec3::zeros();
		self._state.restAccLpState = Mat2x3Double::repeat(f64::NAN);

		self._state.magRefNorm = 0.0;
		self._state.magRefDip = 0.0;
		self._state.magUndisturbedT = 0.0;
		self._state.magRejectT = self._params.magMaxRejectionTime;
		self._state.magCandidateNorm = -1.0;
		self._state.magCandidateDip = 0.0;
		self._state.magCandidateT = 0.0;
		self._state.magNormDip = Vec2::zeros();
		self._state.magNormDipLpState = Mat2x2Double::repeat(f64::NAN);
	}
}

fn quatApplyDelta(q: Quat, delta: VQF_Real_T) -> Quat {
	let c = (delta / 2.0).cos();
	let s = (delta / 2.0).sin();
	let w = c * q.coords.w - s * q.coords.z;
	let x = c * q.coords.x - s * q.coords.y;
	let y = c * q.coords.y + s * q.coords.x;
	let z = c * q.coords.z + s * q.coords.w;
	Quat::from_quaternion([x, y, z, w].into())
}

fn filterCoeffs(tau: VQF_Real_T, Ts: VQF_Real_T) -> (Vec3Double, Vec2Double) {
	assert!(tau > 0.0);
	assert!(Ts > 0.0);
	// second order Butterworth filter based on https://stackoverflow.com/a/52764064
	let fc: f64 = ((2.0 as f64).sqrt() / (2.0 * PI64)) / tau as f64; // time constant of dampened, non-oscillating part of step response
	let C = (PI64 * fc * Ts as f64).tan();
	let D = C.powi(2) + (2.0 as f64).sqrt() * C + 1.0;
	let b0 = C.powi(2) / D;
	let outB: Vec3Double = [b0, 2.0 * b0, b0].into();

	// a0 = 1.0
	let a1 = 2.0 * (C.powi(2) - 1.0) / D;
	let a2 = (1.0 - (2.0 as f64).sqrt() * C + C.powi(2)) / D;
	let outA: Vec2Double = [a1, a2].into();
	return (outB, outA);
}

fn gainFromTau(tau: VQF_Real_T, Ts: VQF_Real_T) -> VQF_Real_T {
	assert!(Ts > 0.0);
	if tau < 0.0 {
		return 0.0; // k=0 for negative tau (disable update)
	} else if tau == 0.0 {
		return 1.0; //k=1 for tau=0
	} else {
		return 1.0 - (-Ts / tau).exp(); // fc = 1/(2*pi*tau)
	}
}

fn filterVec<const N: usize, const M: usize>(
	x: nalgebra::SVector<VQF_Real_T, N>,
	tau: VQF_Real_T,
	Ts: VQF_Real_T,
	b: Vec3Double,
	a: Vec2Double,
	state: &mut nalgebra::Matrix<
		f64,
		nalgebra::Const<M>,
		nalgebra::Const<N>,
		ArrayStorage<f64, M, N>,
	>,
) -> nalgebra::Matrix<
	VQF_Real_T,
	nalgebra::Const<N>,
	nalgebra::Const<1>,
	ArrayStorage<VQF_Real_T, N, 1>,
> {
	assert!(N >= M);
	// to avoid depending on a single sample, average the first samples (for duration tau)
	// and then use this average to calculate the filter initial state
	if state[(0, 0)].is_nan() {
		// initialization phase
		if state[(0, 1)].is_nan() {
			// first sample
			state[(0, 1)] = 0.0; // state[0, 1] is used to store the sample count
					 // ඞ
			state.get_mut((1, ..)).unwrap().fill(0.0); // state[1, :] is used to store the sum
		}

		state[(0, 1)] += 1.0;
		let mut out = nalgebra::SMatrix::zeros();
		// ඞ
		for (i, x) in x.iter().enumerate() {
			state[(1, i)] += *x as f64;
			out[i] = (state[(1, i)] / state[(0, 1)]) as VQF_Real_T;
		}

		if state[(0, 1)] as VQF_Real_T * Ts >= tau {
			for i in 0..N {
				let init = filterInitialState(out[i], b, a);
				// ඞ
				for j in 0..M {
					state[(j, i)] = init[j];
				}
			}
		}
		return out;
	}

	filterStep(x, b, a, state)
}

fn filterInitialState(x0: VQF_Real_T, b: Vec3Double, a: Vec2Double) -> Vec2Double {
	Vec2Double::new(x0 as f64 * (1.0 - b[0]), x0 as f64 * (b[2] - a[1]))
}

fn filterStep<const N: usize, const M: usize>(
	x: nalgebra::SVector<VQF_Real_T, N>,
	b: Vec3Double,
	a: Vec2Double,
	state: &mut nalgebra::Matrix<
		f64,
		nalgebra::Const<M>,
		nalgebra::Const<N>,
		ArrayStorage<f64, M, N>,
	>,
) -> nalgebra::Matrix<
	VQF_Real_T,
	nalgebra::Const<N>,
	nalgebra::Const<1>,
	ArrayStorage<VQF_Real_T, N, 1>,
> {
	// difference equations based on scipy.signal.lfilter documentation
	// assumes that a0 == 1.0
	let x_64 = x.map(|x| x as f64);
	let y_64: nalgebra::Matrix<
		f64,
		nalgebra::Const<N>,
		nalgebra::Const<1>,
		ArrayStorage<f64, N, 1>,
	> = b[0] * x_64 + nalgebra::SMatrix::repeat(state[0]);
	let y: nalgebra::Matrix<
		VQF_Real_T,
		nalgebra::Const<N>,
		nalgebra::Const<1>,
		ArrayStorage<VQF_Real_T, N, 1>,
	> = y_64.map(|x| x as VQF_Real_T);

	// ඞ
	for i in 0..N {
		state[(0, i)] = b[1] * x[i] as f64 - a[0] * y_64[i] + state[(1, i)];
	}
	// ඞ
	for i in 0..N {
		state[(1, i)] = b[2] * x[i] as f64 - a[1] * y_64[i];
	}
	return y;
}

#[cfg(test)]
mod test {
	use crate::Quat;
	use crate::Vec3;
	use crate::Vqf;
	use crate::VqfParameters;

	#[test]
	fn quat_ordering() {
		let quat = Quat::from_quaternion([1.0, 2.0, 3.0, 4.0].into());
		assert!(quat.coords.x == quat[0]);
		assert!((quat.coords.x - 0.182574183).abs() < 1e-7);
		assert!(quat.coords.y == quat[1]);
		assert!((quat.coords.y - 0.365148365).abs() < 1e-7);
		assert!(quat.coords.z == quat[2]);
		assert!((quat.coords.z - 0.547722518).abs() < 1e-7);
		assert!(quat.coords.w == quat[3]);
		assert!((quat.coords.w - 0.730296731).abs() < 1e-7);
	}

	#[test]
	fn single_same_3D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		vqf.updateGyr(gyr);

		let quat = vqf.getQuat3D();
		assert!((quat.coords.w - 1.0).abs() < 1e-6);
		assert!((quat.coords.x - 0.000105).abs() < 1e-6);
		assert!((quat.coords.y - 0.000105).abs() < 1e-6);
		assert!((quat.coords.z - 0.000105).abs() < 1e-6);
	}

	#[test]
	fn single_x_3D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.25, 0.0, 0.0].into();
		vqf.updateGyr(gyr);

		let quat = vqf.getQuat3D();
		assert!((quat.coords.w - 0.9999999).abs() < 1e-6);
		assert!((quat.coords.x - 0.00125).abs() < 1e-6);
		assert!((quat.coords.y - 0.0).abs() < 1e-6);
		assert!((quat.coords.z - 0.0).abs() < 1e-6);
	}

	#[test]
	fn single_y_3D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.0, 0.25, 0.0].into();
		vqf.updateGyr(gyr);

		let quat = vqf.getQuat3D();
		assert!((quat.coords.w - 0.9999999).abs() < 1e-6);
		assert!((quat.coords.x - 0.0).abs() < 1e-6);
		assert!((quat.coords.y - 0.00125).abs() < 1e-6);
		assert!((quat.coords.z - 0.0).abs() < 1e-6);
	}

	#[test]
	fn single_z_3D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.0, 0.0, 0.25].into();
		vqf.updateGyr(gyr);

		let quat = vqf.getQuat3D();
		assert!((quat.coords.w - 0.9999999).abs() < 1e-6);
		assert!((quat.coords.x - 0.0).abs() < 1e-6);
		assert!((quat.coords.y - 0.0).abs() < 1e-6);
		assert!((quat.coords.z - 0.00125).abs() < 1e-6);
	}

	#[test]
	fn single_different_3D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.054, 0.012, -0.9].into();
		vqf.updateGyr(gyr);

		let quat = vqf.getQuat3D();
		assert!((quat.coords.w - 0.99999).abs() < 1e-6);
		assert!((quat.coords.x - 0.000269999).abs() < 1e-6);
		assert!((quat.coords.y - 5.99998e-5).abs() < 1e-6);
		assert!((quat.coords.z - -0.00449998).abs() < 1e-6);
	}

	#[test]
	fn many_same_3D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		for _ in 0..10_000 {
			vqf.updateGyr(gyr);
		}

		let quat = vqf.getQuat3D();
		assert!((quat.coords.w - -0.245327).abs() < 1e-6); //slightly different results
		assert!((quat.coords.x - 0.559707).abs() < 1e-6);
		assert!((quat.coords.y - 0.559707).abs() < 1e-6);
		assert!((quat.coords.z - 0.559707).abs() < 1e-6);
	}

	#[test]
	fn many_different_3D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.054, 0.012, -0.09].into();
		for _ in 0..10_000 {
			vqf.updateGyr(gyr);
		}

		let quat = vqf.getQuat3D();
		assert!((quat.coords.w - 0.539342).abs() < 1e-6); //slightly different results
		assert!((quat.coords.x - -0.430446).abs() < 1e-6);
		assert!((quat.coords.y - -0.0956546).abs() < 1e-6);
		assert!((quat.coords.z - 0.71741).abs() < 1e-6);
	}

	#[test]
	fn single_same_6D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [5.663806, 5.663806, 5.663806].into();
		vqf.updateGyr(gyr);
		vqf.updateAcc(acc);

		let quat = vqf.getQuat6D();
		assert!((quat.coords.w - 0.888074).abs() < 1e-6);
		assert!((quat.coords.x - 0.325117).abs() < 1e-6);
		assert!((quat.coords.y - -0.324998).abs() < 1e-6);
		assert!((quat.coords.z - 0.00016151).abs() < 1e-6);
	}

	#[test]
	fn single_x_6D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [9.81, 0.0, 0.0].into();
		vqf.update(gyr, acc, None);

		let quat = vqf.getQuat6D();
		assert!((quat.coords.w - 0.707107).abs() < 1e-6);
		assert!((quat.coords.x - 0.000148508).abs() < 1e-6);
		assert!((quat.coords.y - -0.707107).abs() < 1e-6);
		assert!((quat.coords.z - 0.000148508).abs() < 1e-6);
	}

	#[test]
	fn single_y_6D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [0.0, 9.81, 0.0].into();
		vqf.update(gyr, acc, None);

		let quat = vqf.getQuat6D();
		assert!((quat.coords.w - 0.707107).abs() < 1e-6);
		assert!((quat.coords.x - 0.707107).abs() < 1e-6);
		assert!((quat.coords.y - 0.000148477).abs() < 1e-6);
		assert!((quat.coords.z - 0.000148477).abs() < 1e-6);
	}

	#[test]
	fn single_z_6D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [0.0, 0.0, 9.81].into();
		vqf.update(gyr, acc, None);

		let quat = vqf.getQuat6D();
		assert!((quat.coords.w - 1.0).abs() < 1e-6);
		assert!((quat.coords.x - -1.72732e-20).abs() < 1e-6);
		assert!((quat.coords.y - -4.06576e-20).abs() < 1e-6);
		assert!((quat.coords.z - 0.000105).abs() < 1e-6);
	}

	#[test]
	fn single_different_6D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [4.5, 6.7, 3.2].into();
		vqf.update(gyr, acc, None);

		let quat = vqf.getQuat6D();
		assert!((quat.coords.w - 0.827216).abs() < 1e-6);
		assert!((quat.coords.x - 0.466506).abs() < 1e-6);
		assert!((quat.coords.y - -0.313187).abs() < 1e-6);
		assert!((quat.coords.z - 0.000168725).abs() < 1e-6);
	}

	#[test]
	fn many_same_6D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [5.663806, 5.663806, 5.663806].into();

		for _ in 0..10_000 {
			vqf.update(gyr, acc, None);
		}

		let quat = vqf.getQuat6D();
		assert!((quat.coords.w - 0.887649).abs() < 1e-6); //Look into why there is so
		assert!((quat.coords.x - 0.334951).abs() < 1e-6); // much difference between them
		assert!((quat.coords.y - -0.314853).abs() < 1e-6); // we use f32 math, they use mostly double math
		assert!((quat.coords.z - 0.0274545).abs() < 1e-6);
	}

	#[test]
	fn many_different_6D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [4.5, 6.7, 3.2].into();

		for _ in 0..10_000 {
			vqf.update(gyr, acc, None);
		}

		let quat = vqf.getQuat6D();
		assert!((quat.coords.w - 0.826852).abs() < 1e-6); //Look into why there is so
		assert!((quat.coords.x - 0.475521).abs() < 1e-6); // much difference between them
		assert!((quat.coords.y - -0.299322).abs() < 1e-6); // we use f32 math, they use mostly double math
		assert!((quat.coords.z - 0.0245133).abs() < 1e-6);
	}

	#[test]
	fn single_same_9D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [5.663806, 5.663806, 5.663806].into();
		let mag: Vec3 = [10.0, 10.0, 10.0].into();
		vqf.updateGyr(gyr);
		vqf.updateAcc(acc);
		vqf.updateMag(mag);

		let quat = vqf.getQuat9D();
		assert!((quat.coords.w - 0.86428).abs() < 1e-6);
		assert!((quat.coords.x - 0.391089).abs() < 1e-6);
		assert!((quat.coords.y - -0.241608).abs() < 1e-6);
		assert!((quat.coords.z - 0.204195).abs() < 1e-6);
	}

	#[test]
	fn single_x_9D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [5.663806, 5.663806, 5.663806].into();
		let mag: Vec3 = [10.0, 0.0, 0.0].into();
		vqf.update(gyr, acc, Some(mag));

		let quat = vqf.getQuat9D();
		assert!((quat.coords.w - 0.540625).abs() < 1e-6);
		assert!((quat.coords.x - 0.455768).abs() < 1e-6);
		assert!((quat.coords.y - 0.060003).abs() < 1e-6);
		assert!((quat.coords.z - 0.704556).abs() < 1e-6);
	}

	#[test]
	fn single_y_9D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [5.663806, 5.663806, 5.663806].into();
		let mag: Vec3 = [0.0, 10.0, 0.0].into();
		vqf.update(gyr, acc, Some(mag));

		let quat = vqf.getQuat9D();
		assert!((quat.coords.w - 0.880476).abs() < 1e-6);
		assert!((quat.coords.x - 0.279848).abs() < 1e-6);
		assert!((quat.coords.y - -0.364705).abs() < 1e-6);
		assert!((quat.coords.z - -0.115917).abs() < 1e-6);
	}

	#[test]
	fn single_z_9D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [5.663806, 5.663806, 5.663806].into();
		let mag: Vec3 = [0.0, 0.0, 10.0].into();
		vqf.update(gyr, acc, Some(mag));

		let quat = vqf.getQuat9D();
		assert!((quat.coords.w - 0.339851).abs() < 1e-6);
		assert!((quat.coords.x - -0.17592).abs() < 1e-6);
		assert!((quat.coords.y - -0.424708).abs() < 1e-6);
		assert!((quat.coords.z - -0.820473).abs() < 1e-6);
	}

	#[test]
	fn single_different_9D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [5.663806, 5.663806, 5.663806].into();
		let mag: Vec3 = [3.54, 6.32, -2.34].into();
		vqf.update(gyr, acc, Some(mag));

		let quat = vqf.getQuat9D();
		assert!((quat.coords.w - 0.864117).abs() < 1e-6);
		assert!((quat.coords.x - 0.391281).abs() < 1e-6);
		assert!((quat.coords.y - -0.241297).abs() < 1e-6);
		assert!((quat.coords.z - 0.204882).abs() < 1e-6);
	}

	#[test]
	fn many_same_9D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [5.663806, 5.663806, 5.663806].into();
		let mag: Vec3 = [10.0, 10.0, 10.0].into();

		for _ in 0..10_000 {
			vqf.update(gyr, acc, Some(mag));
		}

		let quat = vqf.getQuat9D();
		assert!((quat.coords.w - 0.338005).abs() < 1e-6); //Look into why there is so
		assert!((quat.coords.x - -0.176875).abs() < 1e-6); // much difference between them
		assert!((quat.coords.y - -0.424311).abs() < 1e-6); // we use f32 math, they use mostly double math
		assert!((quat.coords.z - -0.821236).abs() < 1e-6);
	}

	#[test]
	fn many_different_9D_quat() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = [0.021, 0.021, 0.021].into();
		let acc: Vec3 = [5.663806, 5.663806, 5.663806].into();
		let mag: Vec3 = [3.54, 6.32, -2.34].into();

		for _ in 0..10_000 {
			vqf.update(gyr, acc, Some(mag));
		}

		let quat = vqf.getQuat9D();
		assert!((quat.coords.w - 0.864111).abs() < 1e-6); //Look into why there is so
		assert!((quat.coords.x - 0.391288).abs() < 1e-6); // much difference between them
		assert!((quat.coords.y - -0.241286).abs() < 1e-6); // we use f32 math, they use mostly double math
		assert!((quat.coords.z - 0.204906).abs() < 1e-6);
	}

	#[test]
	fn run_vqf_cpp_example() {
		let param = VqfParameters::default();
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = Vec3::repeat(0.01745329);
		let acc: Vec3 = Vec3::repeat(5.663806);

		for _ in 0..6000 {
			vqf.update(gyr, acc, None);
		}

		let quat = vqf.getQuat6D();
		assert!((quat.coords.w - 0.887781).abs() < 1e-6);
		assert!((quat.coords.x - 0.333302).abs() < 1e-6);
		assert!((quat.coords.y - -0.316598).abs() < 1e-6);
		assert!((quat.coords.z - 0.0228175).abs() < 1e-6);
	}

	#[test]
	fn run_vqf_cpp_example_basic() {
		let mut param = VqfParameters::default();
		param.restBiasEstEnabled = false;
		param.motionBiasEstEnabled = false;
		param.magDistRejectionEnabled = false;
		let mut vqf = Vqf::new(0.01, 0.01, 0.01, param);

		let gyr: Vec3 = Vec3::repeat(0.01745329);
		let acc: Vec3 = Vec3::repeat(5.663806);

		for _ in 0..6000 {
			vqf.update(gyr, acc, None);
		}

		let quat = vqf.getQuat6D();
		assert!((quat.coords.w - 0.547223).abs() < 1e-6);
		assert!((quat.coords.x - 0.456312).abs() < 1e-6);
		assert!((quat.coords.y - 0.055717).abs() < 1e-6);
		assert!((quat.coords.z - 0.699444).abs() < 1e-6);
	}
}
