use nalgebra::Vector4;
use crate::ray::Ray;


/// Ranga kuta step function and helper
// https://en.wikipedia.org/wiki/Runge%E2%80%93Kutta_methods#The_Runge%E2%80%93Kutta_method

fn geodesic_rhs_from_state(state: &Vector4<f64>, E: f64, r_s: f64) -> Vector4<f64> {
    let r = state[0];
    let dr = state[2];
    let dphi = state[3];

    let f = 1.0 - r_s / r;
    let dt_dλ = E / f;

    let d2r = -(r_s / (2.0 * r * r)) * f * (dt_dλ * dt_dλ)
        + (r_s / (2.0 * r * r * f)) * (dr * dr)
        + (r - r_s) * (dphi * dphi);

    let d2phi = -2.0 * dr * dphi / r;

    Vector4::new(dr, dphi, d2r, d2phi)
}

pub fn rk4_step(ray: &mut Ray, d_λ: f64, r_s: f64) {
    let y0 = Vector4::new(ray.r, ray.phi, ray.dr, ray.dphi);
    
    let k1 = geodesic_rhs_from_state(&y0, ray.E, r_s);
    let k2 = geodesic_rhs_from_state(&(y0 + k1 * (d_λ / 2.0)), ray.E, r_s);
    let k3 = geodesic_rhs_from_state(&(y0 + k2 * (d_λ / 2.0)), ray.E, r_s);
    let k4 = geodesic_rhs_from_state(&(y0 + k3 * d_λ), ray.E, r_s);

    ray.r += (d_λ / 6.0) * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]);
    ray.phi += (d_λ / 6.0) * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]);
    ray.dr += (d_λ / 6.0) * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]);
    ray.dphi += (d_λ / 6.0) * (k1[3] + 2.0 * k2[3] + 2.0 * k3[3] + k4[3]);
}
