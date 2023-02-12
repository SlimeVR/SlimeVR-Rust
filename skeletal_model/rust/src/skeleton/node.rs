use crate::{newtypes::Global, Point};

/// `Node`s represent the connections between [`Edge`]s in the skeleton. `Node`s have
/// positions, but not rotations.
///
/// For more information, see the [`skeleton`](crate::skeleton) module.
///
/// [`Edge`]: crate::skeleton::Edge
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct Node {
	/// Input position in global space. If it is unconstrained, it is `None`.
	pub input_pos_g: Option<Global<Point>>,
	/// The output position of the `Node`. Solving the skeleton updates this.
	pub output_pos_g: Global<Point>,
}
impl Node {
	pub fn new() -> Self {
		Self {
			input_pos_g: None,
			output_pos_g: Default::default(),
		}
	}
}
