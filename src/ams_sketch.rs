#![allow(deprecated)]
use murmurhash3::murmurhash3_x86_32;
use rand::random;

type Seed = u32;

fn create_seed() -> Seed {
    random()
}

fn hash(key: Seed, item: usize) -> u32 {
    let data: [u8; 4] = unsafe { ::std::mem::transmute(item as u32) };
    murmurhash3_x86_32(&data, key)
}

pub struct AMSSketch {
    k: u32,
    data: Vec<Vec<(Seed, isize)>>,
}

#[allow(dead_code)]
impl AMSSketch {
    pub fn new(k: u32, lambda: f32, epsilon: f32) -> Self {
        let (s1, s2) = Self::attributes(k,lambda, epsilon);
        let data = (0..s2)
            .map(|_| (0..s1).map(|_| (create_seed(), 0)).collect())
            .collect();
        AMSSketch { k, data }
    }

    pub fn attributes(k: u32, lambda: f32, epsilon: f32) -> (usize, usize) {
        let s1 = (8.0 * k as f32 / lambda.powi(2)).ceil() as usize;
        let s2 = (2.0 * (1.0 / epsilon).ln()).ceil() as usize;
        (s1, s2)
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

#[cfg(test)]
mod tests {
    use crate::ams_sketch::AMSSketch;
    use std::collections::HashMap;
    use std::error::Error;
    use std::fs::File;
    use std::io::Write;

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
    fn test() -> Result<(), Box<dyn Error>> {
        test_impl(0.5, 0.1)?;
        test_impl(0.6, 0.1)?;
        Ok(())
    }

    fn test_impl(lambda: f32, epsilon: f32) -> Result<(), Box<dyn Error>> {
        let count = 500;
        let mut runs = Vec::with_capacity(count);
        let k = 2;
        let (s1, s2) = AMSSketch::attributes(k, lambda, epsilon);
        for _ in 0..count {
            let mut sketch = AMSSketch::new(k, lambda, epsilon);
            let mut tester = Tester::new(k);

            for i in 0..100 {
                for _ in 0..10 {
                    sketch.update(i);
                    tester.update(i);
                }
            }
            let ams_estimate = sketch.estimate();
            let actual = tester.estimate();
            runs.push((ams_estimate, actual));
        }
        let median_error = {
            let mut runs = runs
                .iter()
                .map(|(ams, det)| (ams - det).abs() as u32)
                .collect::<Vec<_>>();
            runs.sort();
            runs[runs.len() / 2]
        };

        let error_proportion = runs
            .iter()
            .filter(|(ams, det)| {
                let error_dist = det * lambda;
                (ams - det).abs() > error_dist
            })
            .count()
            / runs.len();

        {
            let mut output = File::create(format!(
                "test_results/k_{}_lambda_{}_epsilon_{}.csv",
                k, lambda, epsilon
            ))?;
            writeln!(output, "estimate,actual")?;
            for (ams_estimate, actual) in runs {
                writeln!(output, "{},{}", ams_estimate, actual)?;
            }
        }

        println!(
            "{} runs with λ = {}, ε = {} (buckets = {}, copies = {})",
            count, lambda, epsilon, s1, s2
        );
        println!(
            "median error {}, error proportion {}",
            median_error, error_proportion
        );
        Ok(())
    }
}
