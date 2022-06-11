use crate::bone::{Bone, BoneKind, BoneMap};
use crate::joint::Joint;

use core::ops::Index;
use daggy::{Dag, EdgeIndex};

pub struct SkeletonConfig {
    bone_lengths: BoneMap<f32>,
}

pub struct Skeleton {
    bone_map: BoneMap<EdgeIndex>,
    graph: Dag<Joint, Bone>,
}
impl Skeleton {
    pub fn new(config: &SkeletonConfig) -> Self {
        let mut g = Dag::new();

        // Option is used for resiliance against bugs while the map is being built
        let mut bone_map: BoneMap<Option<EdgeIndex>> = BoneMap::default();

        // Create root bone (edge + joints)
        {
            let head = g.add_node(Joint::new());
            let tail = g.add_node(Joint::new());
            let bone = Bone::new(BoneKind::Neck, config.bone_lengths[BoneKind::Neck]);
            let edge = g.add_edge(head, tail, bone).unwrap();
            bone_map[BoneKind::Neck] = Some(edge);
        }

        // Adds all the immediate children of `parent_bone` to the graph
        let mut add_child_bones = |parent_bone: BoneKind| {
            let parent_edge =
                bone_map[parent_bone].expect("Bone was not yet added to graph");
            let head = g.edge_endpoints(parent_edge).unwrap().1;
            for child_kind in parent_bone.children() {
                let child_kind = *child_kind;
                let tail = g.add_node(Joint::new());
                let _child_edge = g.add_edge(
                    head,
                    tail,
                    Bone::new(child_kind, config.bone_lengths[child_kind]),
                );
            }
        };

        // Call `add_child_bones` in a depth-first traversal to build the actual graph.
        let mut bone_stack = vec![BoneKind::Neck];
        while !bone_stack.is_empty() {
            let parent_bone = bone_stack.pop().unwrap();
            add_child_bones(parent_bone);
            bone_stack.extend(parent_bone.children());
        }

        let bone_map: BoneMap<EdgeIndex> = bone_map.map(|_kind, bone| bone.unwrap());

        Self { graph: g, bone_map }
    }
}
impl Index<BoneKind> for Skeleton {
    type Output = Bone;

    fn index(&self, index: BoneKind) -> &Self::Output {
        let edge = self.bone_map[index];
        &self.graph[edge]
    }
}
