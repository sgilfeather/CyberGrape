use crate::hardware_data_manager::*;
use crate::Point;
use rand::prelude::*;
use std::collections::VecDeque;
use std::f64::consts::PI;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct DummyHdm {
    handle: Option<thread::JoinHandle<()>>,
    tx: mpsc::Sender<Signal>,
    msgs: Arc<Mutex<VecDeque<Update>>>,
    debug_coordinates: Vec<Point>,
}

pub struct DummyHdmBuilder {
    num_points: usize,
    noise: f64,
    range: f64,
    delay: f64,
}

impl DummyHdmBuilder {
    fn new() -> Self {
        Self {
            num_points: 1,
            noise: f64::MIN_POSITIVE,
            range: 1.0,
            delay: 0.25,
        }
    }

    pub fn num_points(mut self, num_points: usize) -> Self {
        self.num_points = num_points;
        self
    }

    pub fn noise(mut self, noise: f64) -> Self {
        self.noise = noise;
        self
    }

    pub fn range(mut self, range: f64) -> Self {
        self.range = range;
        self
    }

    pub fn delay(mut self, delay: f64) -> Self {
        self.delay = delay;
        self
    }

    pub fn build(self) -> DummyHdm {
        DummyHdm::new_from_builder(self)
    }
}

enum Signal {
    Stop,
}

impl HardwareDataManager for DummyHdm {
    fn new() -> Self {
        let b = DummyHdmBuilder::new();
        Self::new_from_builder(b)
    }

    fn clear(&mut self) {
        self.msgs.lock().unwrap().clear();
    }
}

// Notice that all we need to implement iterator is a way to get the next
// element, Rust takes care of the rest.
impl Iterator for DummyHdm {
    type Item = Update;
    fn next(&mut self) -> Option<Self::Item> {
        self.msgs.lock().unwrap().pop_front()
    }
}

// Of course, we can add more functionality beyond what is defined in the
// traits. Here are some control functions.
impl DummyHdm {
    fn new_from_builder(b: DummyHdmBuilder) -> Self {
        let (tx, rx) = mpsc::channel::<Signal>();
        let msgs = Arc::new(Mutex::new(VecDeque::new()));
        let th_msgs = Arc::clone(&msgs);

        let debug_coordinates = generate_circular_points(b.num_points, b.range);
        let th_debug_coords = debug_coordinates.clone();

        let handle = thread::spawn(move || {
            let mut running = true;
            while running {
                if let Ok(received) = rx.try_recv() {
                    match received {
                        Signal::Stop => running = false,
                    }
                }
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

    pub fn builder() -> DummyHdmBuilder {
        DummyHdmBuilder::new()
    }

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

    pub fn get_debug_locations(&self) -> Vec<Point> {
        self.debug_coordinates.clone()
    }
}

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

    others.push(Point { x: 0.0, y: 0.0 });

    others
}

fn generate_flat_updates(points: &[Point], noise: f64) -> VecDeque<Update> {
    let mut rng = thread_rng();
    points
        .iter()
        .enumerate()
        .flat_map(|(i, &p1)| -> Vec<Update> {
            points
                .iter()
                .enumerate()
                .filter(|(j, _)| i != *j)
                .map(|(j, &p2)| -> Update {
                    let dx = p2.x - p1.x + rng.gen_range(-noise..noise);
                    let dy = p2.y - p1.y + rng.gen_range(-noise..noise);
                    let azimuth = dy.atan2(dx);
                    Update {
                        src: i,
                        dst: j,
                        elv: 0.0,
                        azm: azimuth,
                    }
                })
                .collect()
        })
        .collect()
}

#[allow(dead_code)]
fn unflatten_updates(updates: &[Update], noise: f64) -> VecDeque<Update> {
    let mut rng = thread_rng();
    updates
        .iter()
        .map(|u| -> Update {
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
            Point { x: 1.0, y: 0.0 },
            Point { x: 0.0, y: 1.0 },
            Point { x: -1.0, y: 0.0 },
            Point { x: 0.0, y: -1.0 },
            Point { x: 0.0, y: 0.0 },
        ];

        generated_points
            .iter()
            .zip(real_points)
            .for_each(|(gen, other)| {
                assert!(other.abs_dist(gen) < 0.0001);
            });
    }
}
