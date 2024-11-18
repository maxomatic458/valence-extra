use valence::math::{Aabb, DVec3};

pub trait AabbExt {
    fn width_x(&self) -> f64;
    fn width_y(&self) -> f64;
    fn width_z(&self) -> f64;
    fn translate(&self, translation: DVec3) -> Aabb;
}

impl AabbExt for Aabb {
    fn width_x(&self) -> f64 {
        self.max().x - self.min().x
    }

    fn width_y(&self) -> f64 {
        self.max().y - self.min().y
    }

    fn width_z(&self) -> f64 {
        self.max().z - self.min().z
    }

    fn translate(&self, translation: DVec3) -> Aabb {
        Aabb::new(self.min() + translation, self.max() + translation)
    }
}
