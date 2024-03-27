use cgmath::{BaseFloat, Matrix4, Vector3, Vector4};
use std::{cmp::Ordering, ops::Mul};

pub fn partial_min<I, S>(iter: I) -> Option<S>
where
    S: PartialOrd,
    I: Iterator<Item = S>,
{
    iter.min_by(|v1, v2| v1.partial_cmp(v2).unwrap_or(Ordering::Equal))
}

pub fn partial_max<I, S>(iter: I) -> Option<S>
where
    S: PartialOrd,
    I: Iterator<Item = S>,
{
    iter.max_by(|v1, v2| v1.partial_cmp(v2).unwrap_or(Ordering::Equal))
}

#[derive(Copy, Clone, Debug)]
pub struct Aabb<S> {
    min: Vector3<S>,
    max: Vector3<S>,
}

impl<S> Aabb<S> {
    pub fn new(min: Vector3<S>, max: Vector3<S>) -> Self {
        Aabb { min, max }
    }
}

impl<S: BaseFloat> Aabb<S> {
    pub fn union(aabbs: &[Aabb<S>]) -> Option<Self> {
        if aabbs.is_empty() {
            None
        } else if aabbs.len() == 1 {
            Some(aabbs[0])
        } else {
            let min_x = partial_min(aabbs.iter().map(|aabb| aabb.min.x)).unwrap();
            let min_y = partial_min(aabbs.iter().map(|aabb| aabb.min.y)).unwrap();
            let min_z = partial_min(aabbs.iter().map(|aabb| aabb.min.z)).unwrap();
            let min = Vector3::new(min_x, min_y, min_z);

            let max_x = partial_max(aabbs.iter().map(|aabb| aabb.max.x)).unwrap();
            let max_y = partial_max(aabbs.iter().map(|aabb| aabb.max.y)).unwrap();
            let max_z = partial_max(aabbs.iter().map(|aabb| aabb.max.z)).unwrap();
            let max = Vector3::new(max_x, max_y, max_z);

            Some(Aabb::new(min, max))
        }
    }

    pub fn larger_side_size(&self) -> S {
        let size = self.max - self.min;
        let x = size.x.abs();
        let y = size.y.abs();
        let z = size.z.abs();

        if x > y && x > z {
            x
        } else if y > z {
            y
        } else {
            z
        }
    }

    pub fn center(&self) -> Vector3<S> {
        let two = S::one() + S::one();
        self.min + (self.max - self.min) / two
    }
}

impl<S: BaseFloat> Mul<Matrix4<S>> for Aabb<S> {
    type Output = Aabb<S>;

    fn mul(self, rhs: Matrix4<S>) -> Self::Output {
        let min = self.min;
        let min = rhs * Vector4::new(min.x, min.y, min.z, S::one());

        let max = self.max;
        let max = rhs * Vector4::new(max.x, max.y, max.z, S::one());

        Aabb::new(min.truncate(), max.truncate())
    }
}

impl<S: BaseFloat> Mul<S> for Aabb<S> {
    type Output = Aabb<S>;

    fn mul(self, rhs: S) -> Self::Output {
        Aabb::new(self.min * rhs, self.max * rhs)
    }
}
