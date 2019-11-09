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
