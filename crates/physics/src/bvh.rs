use valence::prelude::*;
use valence_spatial::bvh::Bvh;
use vek::Aabb;

#[derive(Resource)]
pub(super) struct BvhResource {
    pub _bvh: Bvh<Aabb<f64>>,
}

impl BvhResource {
    pub fn new() -> Self {
        Self { _bvh: Bvh::new() }
    }
}
