//! The skeleton, and its representation as a graph data structure.
//!
//! The bread and butter of the skeletal model is the [`Skeleton`], which models the
//! pose of a human. The skeleton is comprised of various bones and joints, and there
//! are trackers that get "attached" to the skeleton that represent the trackers worn
//! when using FBT.
//!
//!
//! # Inputs and Outputs of the Skeleton
//! The input trackers are what provide position and/or rotation to the skeleton.
//!
//! The output trackers are for providing 6DOF simulated "trackers" for use in apps that
//! expect 6DOF trackers. These can be used similarly to how vive trackers are used.
//! The pose of the various bones in the skeleton can also be read directly, instead of
//! using the output trackers.
//!
//!
//! # Calibration
//! Before the skeleton can solve for the pose of the various bones and output trackers,
//! it needs the user to stand in an initial calibration pose. This pose is
//! pre-determined, and is necessary to compute the relative transforms of the input
//! trackers to the bones they are attached to.
//!
//! Calibration may be necessary any time the local transform of the tracker with
//! respect to its attached bone changes. This may occur in the following scenarios:
//! * The physical tracker was not securely mounted to the body and shifted during use.
//! * It is a 3DOF tracker experiencing IMU drift, such that the orientation
//!   has diverged from the true value.
//! * The lengths of the bones in the skeleton have been adjusted.
//!
//!
//! # Skeleton Structure
//! Fundamentally, the skeleton is a graph data structure. The graph is a "tree" with
//! the root of the tree at the head of the skeleton. This graph consists of edges
//! and nodes. Each edge has a "head" and a "tail" node. By convention, the side of the
//! edge closer to the root of the tree is the head, and the side further is the tail.
//! This gives a consistent directionality for the tree and makes it easier to describe
//! parent/child relationships.
//!
//!
//! ## The Different Kinds of Edges
//! While all edges hold the same data internally, they play slightly different
//! roles depending on what the edge represents. There are really two types of edges:
//! * A regular bone in the human skeleton.
//! * A an offset between a tracker (either input or output) and the bone it is
//!   attached to.
//!
//! The only difference between these two is that a tracker edge never has any children,
//! and always has a bone as its parent. Also, its local transform with respect to its
//! parent never changes between calibrations because unlike bones, trackers are assumed
//! not to move once they are calibrated, whereas bones bend at their joints.
//!
//! ## Data in the Skeleton
//! The data associated with edges in the skeleton is different from the data associated
//! with nodes.
//!
//! #### Edge
//! * [Rotation](UnitQuat), both the local rotation from calibration, as well as the
//!   latest global rotation. If the latest global rotation is not directly provided via
//!   an input tracker, this will be solved for.
//! * Type of edge (either [`BoneKind`], input tracker, or output tracker)
//! * The length of the edge. This is defined by the user for bones, and computed at
//!   calibration time for tracker edges.
//!
//! #### Node
//! * The latest global position. If not directly provided via an input tracker, this
//!   will be solved for.
//!
//! # The Skeletal Solver
//! TODO: We will document this soon

mod edge;
mod node;

pub(crate) use edge::Edge;
pub(crate) use node::Node;

use crate::prelude::*;

use core::ops::Index;
use daggy::{Dag, EdgeIndex};

/// Used to initialize the [`Skeleton`] with its initial parameters
pub struct SkeletonConfig {
    bone_lengths: BoneMap<f32>,
}
impl SkeletonConfig {
    pub fn new(bone_lengths: BoneMap<f32>) -> Self {
        SkeletonConfig { bone_lengths }
    }
}

/// The `Skeleton` provides a way of reading, writing, and solving for the pose of
/// a human wearing FBT.
///
/// See the [`crate::skeleton`] module for more information.
pub struct Skeleton {
    bone_map: BoneMap<EdgeIndex>,
    graph: Dag<Node, Edge>,
}
impl Skeleton {
    /// Creates a new `Skeleton` from [`SkeletonConfig`]. Initially, the skeleton will
    /// not have any input trackers or output trackers.
    pub fn new(config: &SkeletonConfig) -> Self {
        let mut g = Dag::new();

        // Option is used for resiliance against bugs while the map is being built
        let mut bone_map: BoneMap<Option<EdgeIndex>> = BoneMap::default();

        // Create root skeletal bone: edge (bone) connects to nodes (joints)
        {
            let head = g.add_node(Node::new());
            let (edge, _tail) = g.add_child(
                head,
                Edge::new(BoneKind::Neck, config.bone_lengths[BoneKind::Neck]),
                Node::new(),
            );
            bone_map[BoneKind::Neck] = Some(edge);
        }

        // This closure adds all the immediate children of `parent_bone` to the graph
        let mut add_child_bones = |parent_bone: BoneKind| {
            let parent_edge =
                bone_map[parent_bone].expect("Bone was not yet added to graph");
            let head = g.edge_endpoints(parent_edge).unwrap().1; // Get child node of edge
            for child_kind in parent_bone.children() {
                // No need to work with a ref, `child_kind` is `Copy`
                let child_kind = *child_kind;

                let (edge, _tail) = g.add_child(
                    head,
                    Edge::new(child_kind, config.bone_lengths[child_kind]),
                    Node::new(),
                );

                bone_map[child_kind] = Some(edge);
            }
        };

        // Call `add_child_bones` in a depth-first traversal to build the actual graph.
        let mut bone_stack = vec![BoneKind::Neck];
        while !bone_stack.is_empty() {
            let parent_bone = bone_stack.pop().unwrap();
            add_child_bones(parent_bone);
            bone_stack.extend(parent_bone.children());
        }

        // Map is populated, get rid of the `Optional`
        let bone_map: BoneMap<EdgeIndex> = bone_map.map(|_kind, bone| bone.unwrap());

        Self { graph: g, bone_map }
    }
}
impl Index<BoneKind> for Skeleton {
    type Output = Edge;

    fn index(&self, index: BoneKind) -> &Self::Output {
        let edge = self.bone_map[index];
        &self.graph[edge]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Tests that all lengths of the skeleton are properly initialized based on `SkeletonConfig`
    #[test]
    fn test_lengths() {
        let mut bone_lengths = BoneMap::new([0.; BoneKind::num_types()]);

        bone_lengths[BoneKind::FootL] = 4.0;

        let config = SkeletonConfig::new(bone_lengths);

        let skeleton = Skeleton::new(&config);

        for (bone, length) in bone_lengths.iter() {
            assert_eq!(&skeleton[bone].length(), length);
        }
    }
}
