mod direction;
pub use direction::*;

mod spline;
pub use spline::*;

mod noise;
pub use noise::*;

pub mod l_system;

mod numerical_traits;
pub use numerical_traits::*;

pub mod plugin;

use bevy::prelude::Vec3;

use rand::prelude::*;
use rand_distr::StandardNormal;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

pub fn trilerp<const X: usize, const Y: usize, const Z: usize>(samples: &[[[f32; X]; Y]; Z], x: usize, y: usize, z: usize, sample_interval: usize) -> f32 {
    let index_x = x % sample_interval;
    let index_y = y % sample_interval;
    let index_z = z % sample_interval;

    let factor_x = index_x as f32 / sample_interval as f32;
    let factor_y = index_y as f32 / sample_interval as f32;
    let factor_z = index_z as f32 / sample_interval as f32;

    let point = Vec3::new(factor_x, factor_y, factor_z);

    let v000 = samples[x / sample_interval][y / sample_interval][z / sample_interval];
    let v001 = samples[x / sample_interval][y / sample_interval][z / sample_interval + 1];
    let v010 = samples[x / sample_interval][y / sample_interval + 1][z / sample_interval];
    let v011 = samples[x / sample_interval][y / sample_interval + 1][z / sample_interval + 1];
    let v100 = samples[x / sample_interval + 1][y / sample_interval][z / sample_interval];
    let v101 = samples[x / sample_interval + 1][y / sample_interval][z / sample_interval + 1];
    let v110 = samples[x / sample_interval + 1][y / sample_interval + 1][z / sample_interval];
    let v111 = samples[x / sample_interval + 1][y / sample_interval + 1][z / sample_interval + 1];
    trilinear_interpolation(point, v000, v001, v010, v011, v100, v101, v110, v111)
}

pub fn trilinear_interpolation(point: Vec3, v000: f32, v001: f32, v010: f32, v011: f32, v100: f32, v101: f32, v110: f32, v111: f32) -> f32 {
    let c00 = v000 * (1.0 - point.x) + v100 * point.x;
    let c01 = v001 * (1.0 - point.x) + v101 * point.x;
    let c10 = v010 * (1.0 - point.x) + v110 * point.x;
    let c11 = v011 * (1.0 - point.x) + v111 * point.x;
    let c0 = c00 * (1.0 - point.y) + c10 * point.y;
    let c1 = c01 * (1.0 - point.y) + c11 * point.y;
    c0 * (1.0 - point.z) + c1 * point.z
}

//if v has maximum element m, returns the vector with m set to sign(m) and all other elements 0.
pub fn max_component_norm(v: Vec3) -> Vec3 {
    let abs = v.abs();
    if abs.x > abs.y && abs.x > abs.z {
        Vec3::new(v.x.signum(), 0.0, 0.0)
    } else if abs.y > abs.z {
        Vec3::new(0.0, v.y.signum(), 0.0)
    } else {
        Vec3::new(0.0, 0.0, v.z.signum())
    }
}

//last method on https://mathworld.wolfram.com/SpherePointPicking.html (Muller 1959, Marsaglia 1972).
pub fn sample_sphere_surface(rng: &mut impl Rng) -> Vec3 {
    Vec3::new(
        rng.sample(StandardNormal),
        rng.sample(StandardNormal),
        rng.sample(StandardNormal),
    )
    .try_normalize()
    //should almost never fail, but provide a point in S^2 just in case
    .unwrap_or(Vec3::new(0.0, 1.0, 0.0))
}

//use in lerp(x,b,t) where x is current position, b is target dest
//lerps are exponential functions, so we have to correct the t 
//speed is proportion that we should travel in 1 second
//TODO: https://chicounity3d.wordpress.com/2014/05/23/how-to-lerp-like-a-pro/
pub fn lerp_delta_time(speed: f32, dt: f32) -> f32 {
    //0.5 is arbitrary
    1.0-((1.0-speed).powf(dt))
}