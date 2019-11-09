use crate::ams_sketch::AMSSketch;
use std::collections::HashMap;

mod ams_sketch;

fn main() {
    let (lambda, epsilon) = (0.5, 0.01);

    let count = 10000;
    let mut runs = Vec::with_capacity(count);
    for _ in 0..10 {
        let mut sketch = AMSSketch::new(2, 100, lambda, epsilon);
        let mut tester = Tester::new(2);

        for i in 0..100 {
            for _ in 0..10 {
                sketch.update(i);
                tester.update(i);
            }
        }
        let ams_est = sketch.estimate();
        let det_est = tester.estimate();
        runs.push((ams_est, det_est));
        //        println!("DELTA: {} AMS sketch: {} Tester: {}", (ams_est - det_est).abs(), ams_est, det_est);
    }
    let mean_error =
        runs.iter().map(|(ams, det)| (ams - det).abs()).sum::<f32>() / runs.len() as f32;

    println!("{} runs with ε = {}, λ = {}", count, epsilon, lambda);
    println!("mean error {}", mean_error);
}

struct Tester(HashMap<usize, usize>, u32);

impl Tester {
    pub fn new(k: u32) -> Self {
        Tester(HashMap::new(), k)
    }

    pub fn update(&mut self, i: usize) {
        let entry = self.0.entry(i).or_insert(0);
        *entry += 1;
    }

    pub fn estimate(&self) -> f32 {
        self.0.values().map(|val| val.pow(self.1)).sum::<usize>() as f32
    }
}
