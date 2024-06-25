use glam::{vec3a, vec4, Mat3A, Mat4, Vec3, Vec3A, Vec4, Vec4Swizzles};

#[repr(usize)]
enum FrustumPlane {
    Left,
    Right,
    Bottom,
    Top,
    Near,
    Far,
}

const PLANE_COUNT: usize = 6;
const PLANE_COMBINATIONS: usize = PLANE_COUNT * (PLANE_COUNT - 1) / 2;
const POINT_COUNT: usize = 8;

#[derive(Default)]
pub struct Frustum {
    planes: [Vec4; PLANE_COUNT],
    points: [Vec3A; POINT_COUNT],
}
impl Frustum {
    pub fn compute(perspective_matrix: Mat4, view_matrix: Mat4) -> Self {
        let mat = (perspective_matrix * view_matrix).transpose();

        let mut planes = [Vec4::default(); PLANE_COUNT];
        planes[FrustumPlane::Left as usize] = mat.w_axis + mat.x_axis;
        planes[FrustumPlane::Right as usize] = mat.w_axis - mat.x_axis;
        planes[FrustumPlane::Bottom as usize] = mat.w_axis + mat.y_axis;
        planes[FrustumPlane::Top as usize] = mat.w_axis - mat.y_axis;
        planes[FrustumPlane::Near as usize] = mat.w_axis + mat.z_axis;
        planes[FrustumPlane::Far as usize] = mat.w_axis - mat.z_axis;

        let crosses = [
            Vec3A::from(planes[FrustumPlane::Left as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Right as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Left as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Bottom as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Left as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Top as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Left as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Near as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Left as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Far as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Right as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Bottom as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Right as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Top as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Right as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Near as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Right as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Far as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Bottom as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Top as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Bottom as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Near as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Bottom as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Far as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Top as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Near as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Top as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Far as usize].xyz())),
            Vec3A::from(planes[FrustumPlane::Near as usize].xyz())
                .cross(Vec3A::from(planes[FrustumPlane::Far as usize].xyz())),
        ];

        let points = [
            intersection::<
                { FrustumPlane::Left as usize },
                { FrustumPlane::Bottom as usize },
                { FrustumPlane::Near as usize },
            >(&planes, &crosses),
            intersection::<
                { FrustumPlane::Left as usize },
                { FrustumPlane::Top as usize },
                { FrustumPlane::Near as usize },
            >(&planes, &crosses),
            intersection::<
                { FrustumPlane::Right as usize },
                { FrustumPlane::Bottom as usize },
                { FrustumPlane::Near as usize },
            >(&planes, &crosses),
            intersection::<
                { FrustumPlane::Right as usize },
                { FrustumPlane::Top as usize },
                { FrustumPlane::Near as usize },
            >(&planes, &crosses),
            intersection::<
                { FrustumPlane::Left as usize },
                { FrustumPlane::Bottom as usize },
                { FrustumPlane::Far as usize },
            >(&planes, &crosses),
            intersection::<
                { FrustumPlane::Left as usize },
                { FrustumPlane::Top as usize },
                { FrustumPlane::Far as usize },
            >(&planes, &crosses),
            intersection::<
                { FrustumPlane::Right as usize },
                { FrustumPlane::Bottom as usize },
                { FrustumPlane::Far as usize },
            >(&planes, &crosses),
            intersection::<
                { FrustumPlane::Right as usize },
                { FrustumPlane::Top as usize },
                { FrustumPlane::Far as usize },
            >(&planes, &crosses),
        ];

        Self { planes, points }
    }

    //https://iquilezles.org/articles/frustumcorrect/
    //先测试包围盒的八个顶点是否在视锥体内，再测试视锥体的八个点是否有在包围盒内
    pub fn is_bounding_box_visible(&self, minp: Vec3, maxp: Vec3) -> bool {
        for plane in self.planes {
            if (plane.dot(vec4(minp.x, minp.y, minp.z, 1.)) < 0.)
                && (plane.dot(vec4(maxp.x, minp.y, minp.z, 1.)) < 0.)
                && (plane.dot(vec4(minp.x, maxp.y, minp.z, 1.)) < 0.)
                && (plane.dot(vec4(maxp.x, maxp.y, minp.z, 1.)) < 0.)
                && (plane.dot(vec4(minp.x, minp.y, maxp.z, 1.)) < 0.)
                && (plane.dot(vec4(maxp.x, minp.y, maxp.z, 1.)) < 0.)
                && (plane.dot(vec4(minp.x, maxp.y, maxp.z, 1.)) < 0.)
                && (plane.dot(vec4(maxp.x, maxp.y, maxp.z, 1.)) < 0.)
            {
                return false;
            }
        }

        if self.points.iter().all(|point| point.x > maxp.x) {
            return false;
        }
        if self.points.iter().all(|point| point.x < minp.x) {
            return false;
        }
        if self.points.iter().all(|point| point.y > maxp.y) {
            return false;
        }
        if self.points.iter().all(|point| point.y < minp.y) {
            return false;
        }
        if self.points.iter().all(|point| point.z > maxp.z) {
            return false;
        }
        if self.points.iter().all(|point| point.z < minp.z) {
            return false;
        }

        true
    }
}

const fn ij2k<const I: usize, const J: usize>() -> usize {
    I * (9 - I) / 2 + J - 1
}
fn intersection<const A: usize, const B: usize, const C: usize>(
    planes: &[Vec4; PLANE_COUNT],
    crosses: &[Vec3A; PLANE_COMBINATIONS],
) -> Vec3A {
    let d = Vec3A::from(planes[A].xyz()).dot(crosses[ij2k::<B, C>()]);
    let res = Mat3A::from_cols(
        crosses[ij2k::<B, C>()],
        -crosses[ij2k::<A, C>()],
        crosses[ij2k::<A, B>()],
    ) * vec3a(planes[A].w, planes[B].w, planes[C].w);
    res * (-1. / d)
}
