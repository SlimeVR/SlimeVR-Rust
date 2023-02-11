use crate::bone::BoneMap;
use crate::newtypes::Global;
use crate::Point;

pub struct CalibrationData {
	pub positions: BoneMap<Option<Global<Point>>>,
}
