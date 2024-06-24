use std::mem;

use glam::{Mat4, Vec3};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FrustumCuller {
    nx_x: f32,
    nx_y: f32,
    nx_z: f32,
    nx_w: f32,
    px_x: f32,
    px_y: f32,
    px_z: f32,
    px_w: f32,
    ny_x: f32,
    ny_y: f32,
    ny_z: f32,
    ny_w: f32,
    py_x: f32,
    py_y: f32,
    py_z: f32,
    py_w: f32,
    nz_x: f32,
    nz_y: f32,
    nz_z: f32,
    nz_w: f32,
    pz_x: f32,
    pz_y: f32,
    pz_z: f32,
    pz_w: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    #[inline]
    pub fn from_params(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    #[inline]
    pub fn new() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: f32::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Intersection {
    Inside,
    Partial,
    Outside,
}

impl BoundingBox {
    #[inline]
    pub fn from_params(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn new() -> Self {
        Self::from_params(Vec3::ZERO, Vec3::ZERO)
    }
}

impl FrustumCuller {
    pub fn new() -> Self {
        Self::from_matrix(Mat4::IDENTITY)
    }

    #[inline]
    pub fn from_perspective_mat(perspective_mat: Mat4) -> Self {
        Self::from_matrix(perspective_mat)
    }

    pub fn from_matrix(m: Mat4) -> Self {
        let mut culler: Self = unsafe { mem::zeroed() };

        culler.nx_x = m.x_axis.w + m.x_axis.x;
        culler.nx_y = m.y_axis.w + m.y_axis.x;
        culler.nx_z = m.z_axis.w + m.z_axis.x;
        culler.nx_w = m.w_axis.w + m.w_axis.x;

        let invl =
            (culler.nx_x * culler.nx_x + culler.nx_y * culler.nx_y + culler.nx_z * culler.nx_z)
                .sqrt()
                .recip();
        culler.nx_x *= invl;
        culler.nx_y *= invl;
        culler.nx_z *= invl;
        culler.nx_w *= invl;
        culler.px_x = m.x_axis.w + m.x_axis.x;
        culler.px_y = m.y_axis.w + m.y_axis.x;
        culler.px_z = m.z_axis.w + m.z_axis.x;
        culler.px_w = m.w_axis.w + m.w_axis.x;
        let invl =
            (culler.px_x * culler.px_x + culler.px_y * culler.px_y + culler.px_z * culler.px_z)
                .sqrt()
                .recip();
        culler.px_x *= invl;
        culler.px_y *= invl;
        culler.px_z *= invl;
        culler.px_w *= invl;
        culler.ny_x = m.x_axis.w + m.x_axis.y;
        culler.ny_y = m.y_axis.w + m.y_axis.y;
        culler.ny_z = m.z_axis.w + m.z_axis.y;
        culler.ny_w = m.w_axis.w + m.w_axis.y;
        let invl =
            (culler.ny_x * culler.ny_x + culler.ny_y * culler.ny_y + culler.ny_z * culler.ny_z)
                .sqrt()
                .recip();
        culler.ny_x *= invl;
        culler.ny_y *= invl;
        culler.ny_z *= invl;
        culler.ny_w *= invl;
        culler.py_x = m.x_axis.w + m.x_axis.y;
        culler.py_y = m.y_axis.w + m.y_axis.y;
        culler.py_z = m.z_axis.w + m.z_axis.y;
        culler.py_w = m.w_axis.w + m.w_axis.y;
        let invl =
            (culler.py_x * culler.py_x + culler.py_y * culler.py_y + culler.py_z * culler.py_z)
                .sqrt()
                .recip();
        culler.py_x *= invl;
        culler.py_y *= invl;
        culler.py_z *= invl;
        culler.py_w *= invl;
        culler.nz_x = m.x_axis.w + m.x_axis.z;
        culler.nz_y = m.y_axis.w + m.y_axis.z;
        culler.nz_z = m.z_axis.w + m.z_axis.z;
        culler.nz_w = m.w_axis.w + m.w_axis.z;
        let invl =
            (culler.nz_x * culler.nz_x + culler.nz_y * culler.nz_y + culler.nz_z * culler.nz_z)
                .sqrt()
                .recip();
        culler.nz_x *= invl;
        culler.nz_y *= invl;
        culler.nz_z *= invl;
        culler.nz_w *= invl;
        culler.pz_x = m.x_axis.w + m.x_axis.z;
        culler.pz_y = m.y_axis.w + m.y_axis.z;
        culler.pz_z = m.z_axis.w + m.z_axis.z;
        culler.pz_w = m.w_axis.w + m.w_axis.z;
        let invl =
            (culler.pz_x * culler.pz_x + culler.pz_y * culler.pz_y + culler.pz_z * culler.pz_z)
                .sqrt()
                .recip();
        culler.pz_x *= invl;
        culler.pz_y *= invl;
        culler.pz_z *= invl;
        culler.pz_w *= invl;

        culler
    }

    /// Test wether a 3D point lies inside of the frustum
    pub fn test_point(&self, point: Vec3) -> Intersection {
        if self.nx_x * point.x + self.nx_y * point.y + self.nx_z * point.z + self.nx_w
            >= f32::default()
            && self.px_x * point.x + self.px_y * point.y + self.px_z * point.z + self.px_w
                >= f32::default()
            && self.ny_x * point.x + self.ny_y * point.y + self.ny_z * point.z + self.ny_w
                >= f32::default()
            && self.py_x * point.x + self.py_y * point.y + self.py_z * point.z + self.py_w
                >= f32::default()
            && self.nz_x * point.x + self.nz_y * point.y + self.nz_z * point.z + self.nz_w
                >= f32::default()
            && self.pz_x * point.x + self.pz_y * point.y + self.pz_z * point.z + self.pz_w
                >= f32::default()
        {
            Intersection::Inside
        } else {
            Intersection::Outside
        }
    }

    pub fn test_sphere<T>(&self, sphere: T) -> Intersection
    where
        T: Into<Sphere>,
    {
        let sphere = sphere.into();

        let mut inside = true;
        let mut dist;
        dist = self.nx_x * sphere.center.x
            + self.nx_y * sphere.center.y
            + self.nx_z * sphere.center.z
            + self.nx_w;
        if dist >= -sphere.radius {
            inside &= dist >= sphere.radius;
            dist = self.px_x * sphere.center.x
                + self.px_y * sphere.center.y
                + self.px_z * sphere.center.z
                + self.px_w;
            if dist >= -sphere.radius {
                inside &= dist >= sphere.radius;
                dist = self.ny_x * sphere.center.x
                    + self.ny_y * sphere.center.y
                    + self.ny_z * sphere.center.z
                    + self.ny_w;
                if dist >= -sphere.radius {
                    inside &= dist >= sphere.radius;
                    dist = self.py_x * sphere.center.x
                        + self.py_y * sphere.center.y
                        + self.py_z * sphere.center.z
                        + self.py_w;
                    if dist >= -sphere.radius {
                        inside &= dist >= sphere.radius;
                        dist = self.nz_x * sphere.center.x
                            + self.nz_y * sphere.center.y
                            + self.nz_z * sphere.center.z
                            + self.nz_w;
                        if dist >= -sphere.radius {
                            inside &= dist >= sphere.radius;
                            dist = self.pz_x * sphere.center.x
                                + self.pz_y * sphere.center.y
                                + self.pz_z * sphere.center.z
                                + self.pz_w;
                            if dist >= -sphere.radius {
                                inside &= dist >= sphere.radius;
                                return if inside {
                                    Intersection::Inside
                                } else {
                                    Intersection::Partial
                                };
                            }
                        }
                    }
                }
            }
        }

        Intersection::Outside
    }

    pub fn test_bounding_box<T>(&self, aab: T) -> Intersection
    where
        T: Into<BoundingBox>,
    {
        let aab = aab.into();
        let mut inside = true;
        if self.nx_x
            * if self.nx_x < f32::default() {
                aab.min.x
            } else {
                aab.max.x
            }
            + self.nx_y
                * if self.nx_y < f32::default() {
                    aab.min.y
                } else {
                    aab.max.y
                }
            + self.nx_z
                * if self.nx_z < f32::default() {
                    aab.min.z
                } else {
                    aab.max.z
                }
            >= -self.nx_w
        {
            inside &= self.nx_x
                * if self.nx_x < f32::default() {
                    aab.max.x
                } else {
                    aab.min.x
                }
                + self.nx_y
                    * if self.nx_y < f32::default() {
                        aab.max.y
                    } else {
                        aab.min.y
                    }
                + self.nx_z
                    * if self.nx_z < f32::default() {
                        aab.max.z
                    } else {
                        aab.min.z
                    }
                >= -self.nx_w;
            if self.px_x
                * if self.px_x < f32::default() {
                    aab.min.x
                } else {
                    aab.max.x
                }
                + self.px_y
                    * if self.px_y < f32::default() {
                        aab.min.y
                    } else {
                        aab.max.y
                    }
                + self.px_z
                    * if self.px_z < f32::default() {
                        aab.min.z
                    } else {
                        aab.max.z
                    }
                >= -self.px_w
            {
                inside &= self.px_x
                    * if self.px_x < f32::default() {
                        aab.max.x
                    } else {
                        aab.min.x
                    }
                    + self.px_y
                        * if self.px_y < f32::default() {
                            aab.max.y
                        } else {
                            aab.min.y
                        }
                    + self.px_z
                        * if self.px_z < f32::default() {
                            aab.max.z
                        } else {
                            aab.min.z
                        }
                    >= -self.px_w;
                if self.ny_x
                    * if self.ny_x < f32::default() {
                        aab.min.x
                    } else {
                        aab.max.x
                    }
                    + self.ny_y
                        * if self.ny_y < f32::default() {
                            aab.min.y
                        } else {
                            aab.max.y
                        }
                    + self.ny_z
                        * if self.ny_z < f32::default() {
                            aab.min.z
                        } else {
                            aab.max.z
                        }
                    >= -self.ny_w
                {
                    inside &= self.ny_x
                        * if self.ny_x < f32::default() {
                            aab.max.x
                        } else {
                            aab.min.x
                        }
                        + self.ny_y
                            * if self.ny_y < f32::default() {
                                aab.max.y
                            } else {
                                aab.min.y
                            }
                        + self.ny_z
                            * if self.ny_z < f32::default() {
                                aab.max.z
                            } else {
                                aab.min.z
                            }
                        >= -self.ny_w;
                    if self.py_x
                        * if self.py_x < f32::default() {
                            aab.min.x
                        } else {
                            aab.max.x
                        }
                        + self.py_y
                            * if self.py_y < f32::default() {
                                aab.min.y
                            } else {
                                aab.max.y
                            }
                        + self.py_z
                            * if self.py_z < f32::default() {
                                aab.min.z
                            } else {
                                aab.max.z
                            }
                        >= -self.py_w
                    {
                        inside &= self.py_x
                            * if self.py_x < f32::default() {
                                aab.max.x
                            } else {
                                aab.min.x
                            }
                            + self.py_y
                                * if self.py_y < f32::default() {
                                    aab.max.y
                                } else {
                                    aab.min.y
                                }
                            + self.py_z
                                * if self.py_z < f32::default() {
                                    aab.max.z
                                } else {
                                    aab.min.z
                                }
                            >= -self.py_w;
                        if self.nz_x
                            * if self.nz_x < f32::default() {
                                aab.min.x
                            } else {
                                aab.max.x
                            }
                            + self.nz_y
                                * if self.nz_y < f32::default() {
                                    aab.min.y
                                } else {
                                    aab.max.y
                                }
                            + self.nz_z
                                * if self.nz_z < f32::default() {
                                    aab.min.z
                                } else {
                                    aab.max.z
                                }
                            >= -self.nz_w
                        {
                            inside &= self.nz_x
                                * if self.nz_x < f32::default() {
                                    aab.max.x
                                } else {
                                    aab.min.x
                                }
                                + self.nz_y
                                    * if self.nz_y < f32::default() {
                                        aab.max.y
                                    } else {
                                        aab.min.y
                                    }
                                + self.nz_z
                                    * if self.nz_z < f32::default() {
                                        aab.max.z
                                    } else {
                                        aab.min.z
                                    }
                                >= -self.nz_w;
                            if self.pz_x
                                * if self.pz_x < f32::default() {
                                    aab.min.x
                                } else {
                                    aab.max.x
                                }
                                + self.pz_y
                                    * if self.pz_y < f32::default() {
                                        aab.min.y
                                    } else {
                                        aab.max.y
                                    }
                                + self.pz_z
                                    * if self.pz_z < f32::default() {
                                        aab.min.z
                                    } else {
                                        aab.max.z
                                    }
                                >= -self.pz_w
                            {
                                inside &= self.pz_x
                                    * if self.pz_x < f32::default() {
                                        aab.max.x
                                    } else {
                                        aab.min.x
                                    }
                                    + self.pz_y
                                        * if self.pz_y < f32::default() {
                                            aab.max.y
                                        } else {
                                            aab.min.y
                                        }
                                    + self.pz_z
                                        * if self.pz_z < f32::default() {
                                            aab.max.z
                                        } else {
                                            aab.min.z
                                        }
                                    >= -self.pz_w;
                                return if inside {
                                    Intersection::Inside
                                } else {
                                    Intersection::Partial
                                };
                            }
                        }
                    }
                }
            }
        }

        Intersection::Outside
    }
}

impl From<(Vec3, Vec3)> for BoundingBox {
    #[inline]
    fn from((min, max): (Vec3, Vec3)) -> Self {
        Self { min, max }
    }
}

impl From<(Vec3, f32)> for Sphere {
    #[inline]
    fn from((center, radius): (Vec3, f32)) -> Self {
        Self { center, radius }
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::culling::{FrustumCuller, Intersection, Sphere};

    #[test]
    fn sphere_in_frustum_perspective() {
        let frustum_culling = FrustumCuller::from_matrix(glam::Mat4::perspective_rh(
            Rad(3.14159265 / 2.0),
            1.0,
            0.1,
            100.0,
        ));

        assert_eq!(
            Intersection::Inside,
            frustum_culling.test_sphere(Sphere::from_params(Vec3::new(1.0, 0.0, -2.0), 0.1))
        );
        assert_eq!(
            Intersection::Outside,
            frustum_culling.test_sphere(Sphere::from_params(Vec3::new(4.0, 0.0, -2.0), 0.1))
        );
    }

    #[test]
    fn test_point_in_perspective() {
        let frustum_culling = FrustumCuller::from_matrix(
            PerspectiveFov {
                fovy: Rad(3.14159265 / 2.0),
                aspect: 1.0,
                near: 0.1,
                far: 100.0,
            }
            .into(),
        );

        assert_eq!(
            Intersection::Inside,
            frustum_culling.test_point(Vec3::new(0.0, 0.0, -5.0))
        );
        assert_eq!(
            Intersection::Outside,
            frustum_culling.test_point(Vec3::new(0.0, 6.0, -5.0))
        );
    }

    #[test]
    fn test_aab_in_perspective() {
        let c = FrustumCuller::from_perspective_fov(PerspectiveFov {
            fovy: Rad(3.14159265 / 2.0),
            aspect: 1.0,
            near: 0.1,
            far: 100.0,
        });

        assert_eq!(
            Intersection::Inside,
            c.test_bounding_box(BoundingBox::from_params(
                Vec3::new(0.0, 0.0, -7.0),
                Vec3::new(1.0, 1.0, -5.0)
            ))
        );
        assert_eq!(
            Intersection::Outside,
            c.test_bounding_box(BoundingBox::from_params(
                Vec3::new(1.1, 0.0, 0.0),
                Vec3::new(2.0, 2.0, 2.0)
            ))
        );
        assert_eq!(
            Intersection::Outside,
            c.test_bounding_box(BoundingBox::from_params(
                Vec3::new(4.0, 4.0, -3.0),
                Vec3::new(5.0, 5.0, -5.0)
            ))
        );
        assert_eq!(
            Intersection::Outside,
            c.test_bounding_box(BoundingBox::from_params(
                Vec3::new(-6.0, -6.0, -2.0),
                Vec3::new(-1.0, -4.0, -4.0)
            ))
        );
    }
}
