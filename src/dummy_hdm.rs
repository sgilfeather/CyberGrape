//! TODO

use crate::hardware_data_manager::*;
use crate::Point;
use rand::prelude::*;
use std::collections::VecDeque;
use std::f64::consts::PI;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

/// A dummy implementation of a `HardwareDataManager` that currently pretends
/// that there is a static circle of sound sources around the listener
pub struct DummyHdm {
    handle: Option<thread::JoinHandle<()>>,
    tx: mpsc::Sender<Signal>,
    msgs: Arc<Mutex<VecDeque<Update>>>,
    debug_coordinates: Vec<Point>,
}

/// A utility struct that enables configuration of the `DummyHdm`
pub struct DummyHdmBuilder {
    num_points: usize,
    noise: f64,
    range: f64,
    delay: f64,
}

impl DummyHdmBuilder {
    /// Instantiates a `DummyHdmBuilder` with default values
    fn new() -> Self {
        Self {
            num_points: 1,
            noise: f64::MIN_POSITIVE,
            range: 1.0,
            delay: 0.25,
        }
    }

    /// Sets the number of sound sources around the listener.
    pub fn num_points(mut self, num_points: usize) -> Self {
        self.num_points = num_points;
        self
    }

    /// Sets the "noise" in the simulated measurements of angles
    pub fn noise(mut self, noise: f64) -> Self {
        self.noise = noise;
        self
    }

    /// Defines how far the ring of sound sources is from the listener.
    pub fn range(mut self, range: f64) -> Self {
        self.range = range;
        self
    }

    /// Sets the delay between update calculations in the `DummyHdm`.
    pub fn delay(mut self, delay: f64) -> Self {
        self.delay = delay;
        self
    }

    /// Consumes the builder, instantiating and starting a new `DummyHdm`.
    pub fn build(self) -> DummyHdm {
        DummyHdm::new_from_builder(self)
    }
}

enum Signal {
    Stop,
}

// HardwareDataManager inherits from DummyHdm
impl HardwareDataManager for DummyHdm {
    /// Empty the update queue
    fn clear(&mut self) {
        self.msgs.lock().unwrap().clear();
    }
}

// Notice that all we need to implement iterator is a way to get the next
// element, Rust takes care of the rest.
impl Iterator for DummyHdm {
    type Item = Update;
    fn next(&mut self) -> Option<Self::Item> {
        // aquire the lock on the update queue, then remove and return the
        // first element
        self.msgs.lock().unwrap().pop_front()
    }
}

// Of course, we can add more functionality beyond what is defined in the
// traits. Here are functions to instantiate from a DummyHdmBuilder, get a
// builder, stop the HDM, and get the debug locations.
impl DummyHdm {
    /// Instante an instance with default settings.
    fn new() -> Self {
        let b = DummyHdmBuilder::new();
        Self::new_from_builder(b) // invokes DummyHdm::new_from_builder
    }


    /// Instantiates and starts a dummy hardware data manager from a `DummyHdmBuilder`
    ///
    /// Will continue to run and create new updates until `.stop()` is called.
    fn new_from_builder(b: DummyHdmBuilder) -> Self {
        // Create a communication channel so that we can tell this thing to stop
        let (tx, rx) = mpsc::channel::<Signal>();

        // `Arc<Mutex<T>>` allows thread-safe access to a data structure. `Arc`
        // stands for an "atomic, reference-counted" pointer, meaning that the
        // data will live as long as there is a reference to it, and references
        // can be passed between threads. The `Mutex` ensures that only one
        // thread has access to the data at a time
        let msgs = Arc::new(Mutex::new(VecDeque::new()));

        // This is the reference to the update queue that is going to be
        // passed into the thread
        let th_msgs = Arc::clone(&msgs);

        // Generate the true coordinates of the objects once
        let debug_coordinates = generate_circular_points(b.num_points, b.range);

        // We need to make a clone because it is going to be moved in to the thread,
        // and we need to have access for debug purposes out here in the struct
        let th_debug_coords = debug_coordinates.clone();

        // The closure passed in to `thread::spawn` is going to run in its own
        // thread! `move` means that the closure is going to take ownership of
        // every value that is captured (th_debug_coords, b.noise, and b.delay)
        // rather than trying to borrow them.
        let handle = thread::spawn(move || {
            let mut running = true;
            while running {
                // if we receive a Signal::Stop, stop looping
                if let Ok(received) = rx.try_recv() {
                    match received {
                        Signal::Stop => running = false,
                    }
                }
                // insert a fresh batch of updates into the update queue we need
                // to take the lock on the queue so that no one can muck with it
                // while we are appending to it
                th_msgs
                    .lock()
                    .unwrap()
                    .append(&mut generate_flat_updates(&th_debug_coords, b.noise));

                thread::sleep(Duration::from_secs_f64(b.delay));
            }
        });

        DummyHdm {
            handle: Some(handle),
            tx,
            msgs,
            debug_coordinates,
        }
    }

    /// Emits a Builder that allows a user to configure a custom HDM
    /// Call `.build()` on the resulting object to instantiate an HDM.
    pub fn builder() -> DummyHdmBuilder {
        DummyHdmBuilder::new()
    }

    /// Tells the HDM to stop generating updates
    pub fn stop(&mut self) {
        self.tx.send(Signal::Stop).unwrap();
        // We have to do this `Option` and `.take()` nonsense because calling
        // `.join()` on a `JoinHandle` moves the `JoinHandle` out of the calling
        // scope, which we couldn't do with this struct. The `.take()` brings
        // the `JoinHandle` into the scope of this function, rather than in the
        // struct itself, leaving `None` behind. Now we can call `.join()`.
        if let Some(thread) = self.handle.take() {
            thread.join().unwrap();
        }
    }

    /// Returns the **true** locations of the objects in the dummy HDM.
    pub fn get_debug_locations(&self) -> Vec<Point> {
        self.debug_coordinates.clone()
    }
}

/// Given a num_points, generate num_points angle measurements in radians,
/// distributed evenly around a circle. Then, convert these angles into
/// 2D Cartesian Points around a circle with radius range.
fn generate_circular_points(num_points: usize, range: f64) -> Vec<Point> {
    let mut others: Vec<_> = (0..num_points)
        .map(|v| -> Radian { (v as f64 / num_points as f64) * 2.0 * PI })
        .map(|angle: Radian| -> Point {
            Point {
                x: angle.cos() * range,
                y: angle.sin() * range,
            }
        })
        .collect();

    others.insert(0, Point { x: 0.0, y: 0.0 });

    others
}

/// Given an array of Points, generate Updates that describe the azimuth
/// between all possible pairs of Points (with some noise).
///
/// All updates are "flat" for this function, meaning that they have
/// zero elevation.
fn generate_flat_updates(points: &[Point], noise: f64) -> VecDeque<Update> {
    let mut rng = thread_rng();
    points
        .iter()
        .enumerate()
        // flat_map first maps, then flattens the result. We need this because
        // we are going to generate a vector of updates for each point, then
        // flatten
        .flat_map(|(i, &p1)| -> Vec<Update> {
            points
                .iter()
                .enumerate()
                .filter(|(j, _)| i != *j)
                // for all Point pairs &(p1, &p2), where p1 != p2
                .map(|(j, &p2)| -> Update {
                    let dx = p2.x - p1.x + rng.gen_range(-noise..noise);
                    let dy = p2.y - p1.y + rng.gen_range(-noise..noise);
                    let azimuth = dy.atan2(dx);
                    Update {
                        src: i,
                        dst: j,
                        elv: 0.0, // working in a flat 2D plane, for now
                        azm: azimuth,
                    }
                })
                .collect()
            // now we have all of the updates from p1, so we do that for all
            // possible p1s, and flatten the resulting update vectors
            // into one big vector
        })
        .collect()
}

#[allow(dead_code)]
fn unflatten_updates(updates: &[Update], noise: f64) -> VecDeque<Update> {
    let mut rng = thread_rng();
    updates
        .iter()
        .map(|u| -> Update {
            // Here we are using the "struct update syntax" to create a new
            // update with the same src, dst, and azm; but with a nonzero elv.
            Update {
                elv: rng.gen_range(-noise..noise),
                ..u.clone()
            }
        })
        .collect()
}

// Unit tests in Rust are fantastic. Just look!
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_some_points() {
        let generated_points = generate_circular_points(4, 1.0);
        let real_points = vec![
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
            Point { x: 0.0, y: 1.0 },
            Point { x: -1.0, y: 0.0 },
            Point { x: 0.0, y: -1.0 },
        ];

        generated_points
            .iter()
            .zip(real_points)
            .for_each(|(gen, other)| {
                assert!(other.abs_dist(gen) < 0.0001);
            });
    }
}
