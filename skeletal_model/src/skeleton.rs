use crate::bone::{Bone, BoneKind};
use crate::joint::Joint;

use daggy::{Dag, EdgeIndex};

pub struct Skeleton {
    graph: Dag<Joint, Bone>,
}
impl Skeleton {
    pub fn new() -> Self {
        let mut g = Dag::new();

        // Option is used for resiliance against bugs while the map is being built
        // We index into the "map" with `BoneKind` to get `Option<EdgeIndex>`
        let bone_map: [Option<EdgeIndex>; BoneKind::NUM_TYPES] =
            [None; BoneKind::NUM_TYPES];

        // Adds all the children of `bone` to the graph
        let add_child_bones = |bone: BoneKind| {
            let edge = bone_map[bone].expect("Bone was not yet added to graph");
            todo!()
        };
    }
}
