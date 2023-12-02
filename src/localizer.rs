use crate::hardware_data_manager::Update;
use crate::Point;

/** localize_points()
 * @brief   Given a list of `Update` structs, computes the positions of the blocks
 * @param   Array of Updates, which describe a block's rough azimuth relative to another
 * @returns A vector of Points
 */
pub fn localize_points(measurements: &[Update]) -> Vec<Point> {
    // Debug printing code!!
    // TODO: remove
    fn print_update(m: Update) {
        eprintln!("src={:?}, dst={:?}, elv={:?}, azm={:?}", m.src, m.dst, m.elv, m.azm);
    }
    eprintln!("ALL UPDATES");
    measurements.iter().for_each(print_update);
    // end debug code

    // For now, assume constant range
    let range = 1;

    // For now, generate points just based on angles FROM listener
    // no duplicate updates for the same src, dst pair
    // NEEDSWORK: not averaging measurements over multiple updates?
    measurements.iter().filter(|m| m.src == 0).map(|m| {
        print_update(m); // TODO: remove
        // working in the 2D plan, elv is 0 for now
        let x = range * m.azm.cos() * m.elv.sin();
        let y = range * m.azm.sin() * m.elv.sin();
        Point(x, y)
    }).collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let updates: [Update] = [
            Update(0, 1, 0, 0),   // block 1 is straight ahead of the listener
            Update(0, 2, 0, 1.57) // block 2 is to the right of the listener
        ];
        let points = localize_points(&updates);
        eprintln!("{:?}", points);
    }
}
