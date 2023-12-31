//! TODO

use crate::hardware_data_manager::Update;
use crate::Point;
use std::f64::consts::PI;

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
