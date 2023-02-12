use crate::newtypes::Global;
use crate::{Point, UnitQuat};

#[allow(dead_code)]
pub enum CalibrationData {
	SixDof {
		pos: Global<Point>,
		rot: Global<UnitQuat>,
	},
	ThreeDof {
		rot: Global<UnitQuat>,
	},
}
