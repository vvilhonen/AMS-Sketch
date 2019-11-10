use murmurhash3::murmurhash3_x86_32;
use rand::random;

pub struct AMSSketch {
    k: u32,
    data: Vec<Vec<(u32, isize)>>,
}

impl AMSSketch {
    pub fn new(k: u32, n: usize, lambda: f32, epsilon: f32) -> Self {
        let kf = k as f32;
        let n = n as f32;
        let s1 = (8.0 * kf * n.powf(1.0 - 1.0 / kf) / lambda.powi(2)).ceil() as usize;
        let s2 = (2.0 * (1.0 / epsilon).ln()).ceil() as usize;
        //        println!("k {} n {} lambda {} epsilon {} -> s1 {} s2 {}", k, n, lambda, epsilon, s1, s2);
        let data = (0..s2)
            .map(|_| (0..s1).map(|_| (random(), 0)).collect())
            .collect();
        AMSSketch { k, data }
    }

    pub fn update(&mut self, index: usize) {
        for list in self.data.iter_mut() {
            for (key, count) in list.iter_mut() {
                let sign = (hash(*key, index) % 2) as i32 * 2 - 1;
                *count += sign as isize;
            }
        }
    }

    pub fn estimate(&self) -> f32 {
        let mut means = self
            .data
            .iter()
            .map(|col| {
                col.iter()
                    .map(|(_, count)| count.pow(self.k))
                    .sum::<isize>() as usize
                    / col.len()
            })
            .collect::<Vec<usize>>();
        means.sort();
        let center = means.len() / 2;
        if means.len() % 2 == 1 {
            means[center] as f32
        } else {
            (means[center - 1] + means[center]) as f32 / 2.0
        }
    }
}

fn hash(key: u32, item: usize) -> u32 {
    let data: [u8; 4] = unsafe { ::std::mem::transmute(item as u32) };
    murmurhash3_x86_32(&data, key)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::ams_sketch::AMSSketch;

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

    #[test]
    fn test() {
        let (lambda, epsilon) = (0.5, 0.01);

        let count = 100;
        let mut runs = Vec::with_capacity(count);
        for _ in 0..count {
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
        let median_error = {
            let mut runs = runs.iter().map(|(ams, det)| (ams - det).abs() as u32).collect::<Vec<_>>();
            runs.sort();
            runs[runs.len() / 2]
        };

        println!("{} runs with ε = {}, λ = {}", count, epsilon, lambda);
        println!("median error {}", median_error);
    }
}
