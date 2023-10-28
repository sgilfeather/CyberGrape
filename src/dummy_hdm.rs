use crate::hardware_data_manager::*;
use rand::prelude::*;
use std::collections::VecDeque;
use std::f64::consts::PI;
use std::fmt::Display;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct DummyHdm {
    handle: Option<thread::JoinHandle<()>>,
    tx: mpsc::Sender<Signal>,
    msgs: Arc<Mutex<VecDeque<Update>>>,
}

enum Signal {
    NumPts(usize),
    Noise(f64),
    Range(f64),
    Stop,
}

impl HardwareDataManager for DummyHdm {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Signal>();
        let msgs = Arc::new(Mutex::new(VecDeque::new()));
        let th_msgs = Arc::clone(&msgs);

        let handle = thread::spawn(move || {
            let mut running = true;
            let mut num_pts = 0;
            let mut noise = 0.000001;
            let mut range = 1.0;
            while running {
                if let Ok(received) = rx.try_recv() {
                    match received {
                        Signal::NumPts(new_num_pts) => num_pts = new_num_pts,
                        Signal::Noise(new_noise) => noise = new_noise,
                        Signal::Range(new_range) => range = new_range,
                        Signal::Stop => running = false,
                    }
                }
                th_msgs.lock().unwrap().append(&mut generate_flat_updates(
                    &generate_circular_points(num_pts, range),
                    noise,
                ));
                thread::sleep(Duration::from_secs_f32(0.5));
            }
        });

        DummyHdm {
            handle: Some(handle),
            tx,
            msgs,
        }
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
    pub fn set_blockcount(&self, block_count: usize) {
        // Calling `.unwrap()` because we want to panic if the `.send()` fails.
        self.tx.send(Signal::NumPts(block_count)).unwrap();
    }
    pub fn set_noise(&self, noise: f64) {
        self.tx.send(Signal::Noise(noise)).unwrap();
    }
    pub fn set_range(&self, range: f64) {
        self.tx.send(Signal::Range(range)).unwrap();
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
}

// `Copy` is what we call types that do not need to be borrowed. This is very
// similar to pass-by-value in C/C++. Basic types (integers, floats, etc.) are
// all `Copy`. Fancier things like `String`s are "not `Copy`" because we want
// to borrow those usually. We cannot manually implement `Copy`, it is just a
// signal to the compiler. Any type that is `Copy` must also implement `Clone`.
#[derive(Debug, PartialEq, Clone, Copy)]
struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    #[allow(dead_code)]
    pub fn abs_dist(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

// `Debug` is for dirty, exhaustive, and specific output; the kind that the
// compiler can come up with. If we want something that looks nicer, we use
// another trait, `Display`. This one cannot be `#[derive()]`d, since asthetics
// are not something the compiler cares about, so we implement it ourselves.
impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.3}, {:.3})", self.x, self.y)
    }
}

fn generate_circular_points(num_points: usize, range: f64) -> Vec<Point> {
    (0..num_points)
        .map(|v| -> Radian { (v as f64 / num_points as f64) * 2.0 * PI })
        .map(|angle: Radian| -> Point {
            Point {
                x: angle.cos() * range,
                y: angle.sin() * range,
            }
        })
        .collect()
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
                    let azimuth = (dy / dx).atan();
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
        ];

        generated_points
            .iter()
            .zip(real_points)
            .for_each(|(gen, other)| {
                assert!(other.abs_dist(gen) < 0.0001);
            });
    }
}
