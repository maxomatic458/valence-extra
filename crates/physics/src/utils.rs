use valence::math::{Aabb, Vec3};

/// The normals of a collision with a block
pub struct CollisionNormals {
    /// The normal of the x-axis
    ///
    /// - `None` if there is no collision.
    /// - `true` if the collision is in the `+x` direction.
    pub x: Option<bool>,
    /// The normal of the y-axis
    ///
    /// - `None` if there is no collision.
    /// - `true` if the collision is in the `+y` direction.
    pub y: Option<bool>,
    /// The normal of the z-axis
    ///
    /// - `None` if there is no collision.
    /// - `true` if the collision is in the `+z` direction.
    pub z: Option<bool>,
}

pub struct CollisionResult {
    pub entry_time: f64,
    /// The normals of the
    pub face_direction: CollisionNormals,
}

/// Performs a swept AABB collision
///
/// # Arguments
///
/// * `hb1` - The hitbox of the entity that is moving
/// * `velocity` - The velocity of `hb1`
/// * `hb2` - The hitbox that `hb1` is colliding with
///
/// # Returns
///
/// * `CollisionResult` - The result of the collision
pub fn swept_aabb_collide(hb1: &Aabb, velocity: &Vec3, hb2: &Aabb) -> Option<CollisionResult> {
    let (vx, vy, vz) = (velocity.x, velocity.y, velocity.z);

    fn time(x: f64, y: f32) -> f64 {
        if y != 0.0 {
            x / y as f64
        } else if x > 0.0 {
            f64::INFINITY
        } else {
            f64::NEG_INFINITY
        }
    }

    // TODO: maybe refactor this
    let x_entry = if vx != 0.0 {
        time(
            if vx > 0.0 {
                hb2.min().x - hb1.max().x
            } else {
                hb2.max().x - hb1.min().x
            },
            vx,
        )
    } else if hb1.max().x < hb2.min().x || hb1.min().x > hb2.max().x {
        return None;
    } else {
        f64::NEG_INFINITY
    };

    let x_exit = if vx != 0.0 {
        time(
            if vx > 0.0 {
                hb2.max().x - hb1.min().x
            } else {
                hb2.min().x - hb1.max().x
            },
            vx,
        )
    } else {
        f64::INFINITY
    };

    let y_entry = if vy != 0.0 {
        time(
            if vy > 0.0 {
                hb2.min().y - hb1.max().y
            } else {
                hb2.max().y - hb1.min().y
            },
            vy,
        )
    } else if hb1.max().y < hb2.min().y || hb1.min().y > hb2.max().y {
        return None;
    } else {
        f64::NEG_INFINITY
    };

    let y_exit = if vy != 0.0 {
        time(
            if vy > 0.0 {
                hb2.max().y - hb1.min().y
            } else {
                hb2.min().y - hb1.max().y
            },
            vy,
        )
    } else {
        f64::INFINITY
    };

    let z_entry = if vz != 0.0 {
        time(
            if vz > 0.0 {
                hb2.min().z - hb1.max().z
            } else {
                hb2.max().z - hb1.min().z
            },
            vz,
        )
    } else if hb1.max().z < hb2.min().z || hb1.min().z > hb2.max().z {
        return None;
    } else {
        f64::NEG_INFINITY
    };

    let z_exit = if vz != 0.0 {
        time(
            if vz > 0.0 {
                hb2.max().z - hb1.min().z
            } else {
                hb2.min().z - hb1.max().z
            },
            vz,
        )
    } else {
        f64::INFINITY
    };

    if x_entry < 0.0 && y_entry < 0.0 && z_entry < 0.0 {
        return None;
    }

    if x_entry > 1.0 || y_entry > 1.0 || z_entry > 1.0 {
        return None;
    }

    let entry = x_entry.max(y_entry).max(z_entry);
    let exit = x_exit.min(y_exit).min(z_exit);

    if entry > exit {
        return None;
    }

    let nx = if entry == x_entry {
        Some(vx <= 0.0)
    } else {
        None
    };

    let ny = if entry == y_entry {
        Some(vy <= 0.0)
    } else {
        None
    };

    let nz = if entry == z_entry {
        Some(vz <= 0.0)
    } else {
        None
    };

    Some(CollisionResult {
        entry_time: entry,
        face_direction: CollisionNormals {
            x: nx,
            y: ny,
            z: nz,
        },
    })
}
