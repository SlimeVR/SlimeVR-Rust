use crate::bone::{Bone, BoneKind};
use crate::joint::Joint;

use daggy::Dag;

pub struct Skeleton {
    graph: Dag<Joint, Bone>,
}
impl Skeleton {
    pub fn new() -> Self {
        let mut g = Dag::new();

        let head = g.add_node(Joint::default());
        // for bkind in BoneKind::iter() {
        //     use BoneKind::*;
        //     match bkind {
        //         Neck => {
        //             todo!()
        //         }
        //         Chest | Waist | Hip => todo!(),
        //         ThighL | => todo!(),
        //         ThighR => todo!(),
        //         AnkleL => todo!(),
        //         AnkleR => todo!(),
        //         FootL => todo!(),
        //         FootR => todo!(),
        //         UpperArmL => todo!(),
        //         UpperArmR => todo!(),
        //         ForearmL => todo!(),
        //         ForearmR => todo!(),
        //         WristL => todo!(),
        //         WristR => todo!(),
        //     }
        //     let tail = g.add_node(Joint::default());
        //     let bone = g.add_edge(tail, head, Bone::new(bkind));
        //     head = tail;
        // }
        /// Adds all the children of `bone` to the graph
        fn add_child_bones(
            bone: BoneKind,
            tail: NodeIndex<Joint>,
        ) -> Vec<NodeIndex<Joint>> {
            match bone {}
        }
    }
}
