use crate::prelude::*;

/// Joints represent the pivot points of [`Edge`]s in the skeleton.
/// `Joint`s have positions, but not rotations.
///
/// For more information, see the [`skeleton`](crate::skeleton) module.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Joint {
    /// Input position in global space. If it is unconstrained, it is `None`.
    input_pos_g: Option<Global<Point>>,
    /// The output position of the Joint. Solving the skeleton updates this.
    output_pos_g: Global<Point>,
}
#[allow(dead_code)]
impl Joint {
    pub fn new() -> Self {
        Self {
            input_pos_g: None,
            output_pos_g: Default::default(),
        }
    }

    pub fn input_position(&self) -> Option<&Global<Point>> {
        self.input_pos_g.as_ref()
    }

    pub fn input_position_mut(&mut self) -> Option<&mut Global<Point>> {
        self.input_pos_g.as_mut()
    }

    pub fn output_position(&self) -> &Global<Point> {
        &self.output_pos_g
    }

    pub fn output_position_mut(&mut self) -> &mut Global<Point> {
        &mut self.output_pos_g
    }
}
