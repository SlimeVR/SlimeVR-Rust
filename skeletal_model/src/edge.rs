use crate::prelude::*;

use derive_more::From;

/// The different kinds of edges.
#[derive(Debug, Eq, PartialEq, Hash, From, Copy, Clone)]
pub enum EdgeKind {
    /// Represents a regular bone in the skeleton.
    Bone(BoneKind),
    /// Represents a tracker that is providing pose information as an input to the
    /// skeleton.
    InputTracker,
    /// Represents a computed/synthetic tracker that will act as a
    OutputTracker,
}

/// `Edge`s represent the connections between the [`Node`]s of the
/// skeleton.
///
/// Edges have a global rotation, represented as a unit quaternion. To
/// get an edge's position, you can get the position of either of its two attached
/// `Node`s.
///
/// Note that by convention, the directionality of edges points towards the top of the
/// skeleton. So the head of the edge would also be the tail of an edge closer to the
/// top of the skeleton. This is simply to give the parent and child of an edge a
/// consistent meaning.
///
/// For more information, see the [`skeleton`](crate::skeleton) module.
pub struct Edge {
    kind: EdgeKind,
    /// Input rotation in global space. If it is unconstrained, it is `None`.
    input_rot_g: Option<Global<UnitQuat>>,
    /// Local rotation of the edge with respect to the parent edge at calibration time.
    /// Maps from parent frame to child frame.
    calib_rot_l: Local<UnitQuat>,
    /// Length of the edge. May be set by the user, or may be computed at calibration.
    length: f32,
    /// The output rotation of the edge. Solving the skeleton updates this.
    output_rot_g: Global<UnitQuat>,
}
impl Edge {
    pub fn new(kind: impl Into<EdgeKind>, length: f32) -> Self {
        let kind = kind.into();
        let calib_rot_l = match kind {
            EdgeKind::Bone(k) => k.calibration_rotation_local(),
            _ => UnitQuat::identity().into(),
        };
        Self {
            kind,
            input_rot_g: None,
            calib_rot_l,
            length,
            output_rot_g: Default::default(),
        }
    }

    pub fn input_rotation_mut(&mut self) -> Option<&mut Global<UnitQuat>> {
        self.input_rot_g.as_mut()
    }

    pub fn output_rotation(&self) -> &Global<UnitQuat> {
        &self.output_rot_g
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn length_mut(&mut self) -> &mut f32 {
        &mut self.length
    }

    pub fn calibration_rotation(&self) -> &Local<UnitQuat> {
        &self.calib_rot_l
    }

    pub fn kind(&self) -> EdgeKind {
        self.kind
    }
}
