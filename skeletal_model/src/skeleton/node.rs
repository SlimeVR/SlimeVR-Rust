use crate::prelude::*;

/// `Node`s represent the connections between [`Edge`]s in the skeleton. `Node`s have
/// positions, but not rotations.
///
/// For more information, see the [`skeleton`](crate::skeleton) module.
///
/// [`Edge`]: crate::skeleton::Edge
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Node {
    /// Input position in global space. If it is unconstrained, it is `None`.
    input_pos_g: Option<Global<Point>>,
    /// The output position of the `Node`. Solving the skeleton updates this.
    output_pos_g: Global<Point>,
}
#[allow(dead_code)]
impl Node {
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
