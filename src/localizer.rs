//! Converts radial points into cartesian points

use crate::hardware_data_manager::Update;
use std::f64::consts::PI;

/// A simple x/y cartesian point
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[allow(missing_docs)]
impl Point {
    #[allow(dead_code)]
    pub fn abs_dist(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.3}, {:.3})", self.x, self.y)
    }
}

/// Given a list of `Update` structs containing the angular measurements between
/// points, computes the cartesian positions of the points.
pub fn localize_points(measurements: &[Update]) -> Vec<Point> {
    // For now, assume constant range
    let range = 5.0;

    // For now, generate points just based on angles FROM listener
    // no duplicate updates for the same src, dst pair
    // NEEDSWORK: not averaging measurements over multiple updates?
    measurements
        .iter()
        .filter(|m| m.src == 0)
        .map(|m| {
            // working in the 2D plan, elv is 0 for now
            let elv = PI / 2.0 - m.elv;
            let x = range * m.azm.cos() * elv.sin();
            let y = range * m.azm.sin() * elv.sin();
            Point { x, y }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let updates = [
            Update {
                src: 0,
                dst: 1,
                elv: 0.0,
                azm: 0.0,
            }, // block 1 is straight ahead of the listener
            Update {
                src: 0,
                dst: 2,
                elv: 0.0,
                azm: 1.57,
            }, // block 2 is to the right of the listener
        ];
        let points = localize_points(&updates);
        eprintln!("{:?}", points);
    }
}
