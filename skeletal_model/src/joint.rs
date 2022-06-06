use crate::Translation;

/// A `PositionConstraint` represents position as a global translation vector.
/// It is a constraint because the skeleton will have some [`Joint`]s what have a fixed
/// position and some joints that have an unknown position, that may or may not have
/// been solved for.
///
/// - It can be `Fixed`, which means the position is constrained to a particular
///   position.
/// - It can be `Unsolved` which means the position has been unconstrained, but it has
///   not yet been solved for.
/// - It can also be `Solved` which means the position has been solved for based on the
///   other `Fixed` positions in the skeleton.
#[derive(Debug, PartialEq)]
pub enum PositionConstraint {
    /// Means the position is constrained to a particular position.
    Fixed(Translation),
    /// The position has been unconstrained, but it has not yet been solved for.
    Solved(Translation),
    /// The position has been solved for based on the other `Fixed` positions in the skeleton.
    Unsolved,
}
impl Default for PositionConstraint {
    fn default() -> Self {
        PositionConstraint::Unsolved
    }
}

/// Joints represent the pivot points of [`Bone`]s in the skeleton. Pivot points have
/// positions, but not rotations,
#[derive(Debug, Default)]
pub struct Joint {
    global_pos: PositionConstraint,
}
