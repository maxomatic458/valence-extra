use ordered_float::NotNan;
use std::{cmp::Reverse, collections::BinaryHeap, fmt::Debug};
use valence::math::{Aabb, DVec3};
use vek::num_traits::Zero;

use crate::{
    utils::{AabbExt, NodeOrd},
    Bvh, Node,
};

impl<T: Debug> Bvh<T> {
    /// Returns the closest element to the target and the distance squared to it.
    pub fn get_closest(&self, target: DVec3, get_aabb: impl Fn(&T) -> Aabb) -> Option<(&T, f64)> {
        let mut min_dist2 = f64::INFINITY;
        let mut min_node = None;

        let on = self.root();

        let on = match on {
            Node::Internal(internal) => internal,
            Node::Leaf(leaf) => {
                return leaf
                    .iter()
                    .map(|elem| {
                        let aabb = get_aabb(elem);

                        let dist2 = aabb.dist2(target);
                        (elem, dist2)
                    })
                    .min_by_key(|(_, dist)| dist.to_bits());
            }
        };

        // let mut stack: SmallVec<&BvhNode, 64> = SmallVec::new();
        let mut heap: BinaryHeap<_> = std::iter::once(on)
            .map(|node| {
                Reverse(NodeOrd {
                    node,
                    by: NotNan::zero(),
                })
            })
            .collect();

        while let Some(on) = heap.pop() {
            let on = on.0.node;
            let dist2 = on.aabb.dist2(target);

            if dist2 > min_dist2 {
                break;
            }

            for child in on.children(self) {
                match child {
                    Node::Internal(internal) => {
                        let dist2 = internal.aabb.dist2(target);
                        let dist2 = NotNan::new(dist2).unwrap();

                        heap.push(Reverse(NodeOrd {
                            node: internal,
                            by: dist2,
                        }));
                    }
                    Node::Leaf(leaf) => {
                        let Some((elem, dist2)) = leaf
                            .iter()
                            .map(|elem| {
                                let aabb = get_aabb(elem);
                                let dist2 = aabb.dist2(target);
                                (elem, dist2)
                            })
                            .min_by_key(|(_, dist)| dist.to_bits())
                        else {
                            continue;
                        };

                        if dist2 < min_dist2 {
                            min_dist2 = dist2;
                            min_node = Some(elem);
                        }
                    }
                }
            }
        }

        min_node.map(|elem| (elem, min_dist2))
    }
}
