use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

#[derive(Debug, Clone)]
pub struct RandomGenerator {
    rng: SmallRng,
}

impl RandomGenerator {
    pub fn new() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    pub fn gen_bytes(&mut self, length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        self.rng.fill(&mut bytes[..]);
        bytes
    }

    pub fn gen_u8(&mut self) -> u8 {
        self.rng.gen()
    }

    pub fn gen_u16(&mut self) -> u16 {
        self.rng.gen()
    }

    pub fn gen_u32(&mut self) -> u32 {
        self.rng.gen()
    }

    pub fn gen_u64(&mut self) -> u64 {
        self.rng.gen()
    }

    pub fn gen_i8(&mut self) -> i8 {
        self.rng.gen()
    }

    pub fn gen_i16(&mut self) -> i16 {
        self.rng.gen()
    }

    pub fn gen_i32(&mut self) -> i32 {
        self.rng.gen()
    }

    pub fn gen_i64(&mut self) -> i64 {
        self.rng.gen()
    }

    pub fn gen_f32(&mut self) -> f32 {
        self.rng.gen()
    }

    pub fn gen_f64(&mut self) -> f64 {
        self.rng.gen()
    }

    pub fn gen_bool(&mut self) -> bool {
        self.rng.gen()
    }

    pub fn gen_range(&mut self, min: u64, max: u64) -> u64 {
        self.rng.gen_range(min..max)
    }

    pub fn gen_range_inclusive(&mut self, min: u64, max: u64) -> u64 {
        self.rng.gen_range(min..=max)
    }

    pub fn gen_weighted_choice<'a, T>(&mut self, items: &'a [(T, u32)]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }

        let total_weight: u32 = items.iter().map(|(_, weight)| *weight).sum();
        if total_weight == 0 {
            return None;
        }

        let mut random_weight = self.gen_range(0, total_weight as u64) as u32;

        for (item, weight) in items {
            if random_weight < *weight {
                return Some(item);
            }
            random_weight -= *weight;
        }

        items.last().map(|(item, _)| item)
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        use rand::seq::SliceRandom;
        slice.shuffle(&mut self.rng);
    }

    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        use rand::seq::SliceRandom;
        slice.choose(&mut self.rng)
    }

    pub fn choose_multiple<'a, T>(&mut self, slice: &'a [T], count: usize) -> Vec<&'a T> {
        use rand::seq::SliceRandom;
        slice.choose_multiple(&mut self.rng, count).collect()
    }

    pub fn sample_unique<T: Clone>(&mut self, items: &[T], count: usize) -> Vec<T> {
        if count >= items.len() {
            return items.to_vec();
        }

        let mut indices: Vec<usize> = (0..items.len()).collect();
        self.shuffle(&mut indices);

        indices[..count].iter().map(|&i| items[i].clone()).collect()
    }

    pub fn gen_string(&mut self, length: usize, charset: &str) -> String {
        let chars: Vec<char> = charset.chars().collect();
        (0..length)
            .map(|_| chars[self.gen_range(0, chars.len() as u64) as usize])
            .collect()
    }

    pub fn gen_alphanumeric(&mut self, length: usize) -> String {
        self.gen_string(length, "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789")
    }

    pub fn gen_hex(&mut self, length: usize) -> String {
        self.gen_string(length, "0123456789abcdef")
    }

    pub fn gen_base64(&mut self, length: usize) -> String {
        self.gen_string(length, "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/")
    }

    pub fn gen_uuid(&mut self) -> String {
        let bytes = self.gen_bytes(16);
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11],
            bytes[12], bytes[13], bytes[14], bytes[15]
        )
    }

    pub fn gen_seed(&mut self) -> u64 {
        self.gen_u64()
    }

    pub fn reset_with_seed(&mut self, seed: u64) {
        self.rng = SmallRng::seed_from_u64(seed);
    }

    pub fn fork(&self) -> Self {
        Self {
            rng: SmallRng::from_rng(self.rng.clone()).unwrap_or_else(|_| SmallRng::from_entropy()),
        }
    }
}

impl Default for RandomGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct WeightedRandom<T> {
    items: Vec<(T, u32)>,
    total_weight: u32,
}

impl<T> WeightedRandom<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            total_weight: 0,
        }
    }

    pub fn add_item(&mut self, item: T, weight: u32) {
        self.total_weight += weight;
        self.items.push((item, weight));
    }

    pub fn add_items(&mut self, items: impl IntoIterator<Item = (T, u32)>) {
        for (item, weight) in items {
            self.add_item(item, weight);
        }
    }

    pub fn remove_item(&mut self, index: usize) -> Option<(T, u32)> {
        if index < self.items.len() {
            let removed = self.items.remove(index);
            self.total_weight -= removed.1;
            Some(removed)
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.total_weight = 0;
    }

    pub fn choose(&self, rng: &mut RandomGenerator) -> Option<&T> {
        if self.items.is_empty() || self.total_weight == 0 {
            return None;
        }

        let mut random_weight = rng.gen_range(0, self.total_weight as u64) as u32;

        for (item, weight) in &self.items {
            if random_weight < *weight {
                return Some(item);
            }
            random_weight -= *weight;
        }

        self.items.last().map(|(item, _)| item)
    }

    pub fn choose_mut(&mut self, rng: &mut RandomGenerator) -> Option<&mut T> {
        if self.items.is_empty() || self.total_weight == 0 {
            return None;
        }

        let mut random_weight = rng.gen_range(0, self.total_weight as u64) as u32;

        for index in 0..self.items.len() {
            let weight = self.items[index].1;
            if random_weight < weight {
                return self.items.get_mut(index).map(|(item, _)| item);
            }
            random_weight -= weight;
        }

        self.items.last_mut().map(|(item, _)| item)
    }

    pub fn items(&self) -> &[(T, u32)] {
        &self.items
    }

    pub fn total_weight(&self) -> u32 {
        self.total_weight
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T> Default for WeightedRandom<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PerlinNoise {
    permutation: [u8; 512],
    gradients: [(f32, f32); 256],
}

impl PerlinNoise {
    pub fn new() -> Self {
        let mut rng = RandomGenerator::new();
        Self::with_rng(&mut rng)
    }

    pub fn with_seed(seed: u64) -> Self {
        let mut rng = RandomGenerator::with_seed(seed);
        Self::with_rng(&mut rng)
    }

    fn with_rng(rng: &mut RandomGenerator) -> Self {
        let mut permutation = [0u8; 256];
        for i in 0..256 {
            permutation[i] = i as u8;
        }
        rng.shuffle(&mut permutation);

        let mut full_permutation = [0u8; 512];
        full_permutation[..256].copy_from_slice(&permutation);
        full_permutation[256..].copy_from_slice(&permutation);

        let mut gradients = [(0.0f32, 0.0f32); 256];
        for i in 0..256 {
            let angle = rng.gen_range(0, 360) as f32 * std::f32::consts::PI / 180.0;
            gradients[i] = (angle.cos(), angle.sin());
        }

        Self {
            permutation: full_permutation,
            gradients,
        }
    }

    pub fn noise(&self, x: f32, y: f32) -> f32 {
        let x_floor = x.floor() as i32;
        let y_floor = y.floor() as i32;

        let xf = x - x_floor as f32;
        let yf = y - y_floor as f32;

        let u = self.fade(x - xf);
        let v = self.fade(y - yf);

        let aa = self.hash(x_floor, y_floor);
        let ab = self.hash(x_floor, y_floor + 1);
        let ba = self.hash(x_floor + 1, y_floor);
        let bb = self.hash(x_floor + 1, y_floor + 1);

        let x1 = self.lerp(
            self.dot(self.gradients[aa as usize], x - xf, y - yf),
            self.dot(self.gradients[ba as usize], x - xf - 1.0, y - yf),
            u,
        );

        let x2 = self.lerp(
            self.dot(self.gradients[ab as usize], x - xf, y - yf - 1.0),
            self.dot(self.gradients[bb as usize], x - xf - 1.0, y - yf - 1.0),
            u,
        );

        self.lerp(x1, x2, v)
    }

    pub fn noise_octave(&self, x: f32, y: f32, octaves: u32, persistence: f32, scale: f32) -> f32 {
        let mut total = 0.0f32;
        let mut frequency = scale;
        let mut amplitude = 1.0f32;
        let mut max_value = 0.0f32;

        for _ in 0..octaves {
            total += self.noise(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        total / max_value
    }

    fn fade(&self, t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn lerp(&self, a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    fn dot(&self, gradient: (f32, f32), x: f32, y: f32) -> f32 {
        gradient.0 * x + gradient.1 * y
    }

    fn hash(&self, x: i32, y: i32) -> u8 {
        self.permutation[(self.permutation[x as usize & 255] as usize + y as usize & 255) & 255]
    }
}

impl Default for PerlinNoise {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct NoiseGenerator {
    perlin: PerlinNoise,
    scale: f32,
    octaves: u32,
    persistence: f32,
}

impl NoiseGenerator {
    pub fn new() -> Self {
        Self {
            perlin: PerlinNoise::new(),
            scale: 0.01,
            octaves: 4,
            persistence: 0.5,
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            perlin: PerlinNoise::with_seed(seed),
            scale: 0.01,
            octaves: 4,
            persistence: 0.5,
        }
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn octaves(mut self, octaves: u32) -> Self {
        self.octaves = octaves;
        self
    }

    pub fn persistence(mut self, persistence: f32) -> Self {
        self.persistence = persistence;
        self
    }

    pub fn generate_2d(&self, width: usize, height: usize) -> Vec<f32> {
        let mut noise = Vec::with_capacity(width * height);

        for y in 0..height {
            for x in 0..width {
                let value = self.perlin.noise_octave(
                    x as f32 * self.scale,
                    y as f32 * self.scale,
                    self.octaves,
                    self.persistence,
                    1.0,
                );
                noise.push((value + 1.0) * 0.5);
            }
        }

        noise
    }

    pub fn generate_1d(&self, length: usize) -> Vec<f32> {
        let mut noise = Vec::with_capacity(length);

        for x in 0..length {
            let value = self.perlin.noise_octave(
                x as f32 * self.scale,
                0.0,
                self.octaves,
                self.persistence,
                1.0,
            );
            noise.push((value + 1.0) * 0.5);
        }

        noise
    }

    pub fn apply_to_bytes(&self, data: &mut [u8], strength: f32) {
        let noise_1d = self.generate_1d(data.len());

        for (i, byte) in data.iter_mut().enumerate() {
            let noise_value = noise_1d[i];
            let offset = ((noise_value - 0.5) * strength * 255.0) as i32;
            *byte = (*byte as i32 + offset).clamp(0, 255) as u8;
        }
    }

    pub fn apply_to_pixels(&self, pixels: &mut [u8], width: usize, height: usize, strength: f32) {
        let noise_2d = self.generate_2d(width, height);

        for (i, pixel) in pixels.iter_mut().enumerate() {
            let x = i % width;
            let y = i / width;
            let noise_value = noise_2d[y * width + x];
            let offset = ((noise_value - 0.5) * strength * 255.0) as i32;
            *pixel = (*pixel as i32 + offset).clamp(0, 255) as u8;
        }
    }
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RandomDistribution {
    generator: RandomGenerator,
}

impl RandomDistribution {
    pub fn new() -> Self {
        Self {
            generator: RandomGenerator::new(),
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            generator: RandomGenerator::with_seed(seed),
        }
    }

    pub fn uniform(&mut self, min: f64, max: f64) -> f64 {
        self.generator.gen_range(min as u64, max as u64) as f64 + (min.fract())
    }

    pub fn normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        use rand_distr::{Distribution, Normal};
        let normal = Normal::new(mean, std_dev).unwrap();
        normal.sample(&mut self.generator.rng)
    }

    pub fn exponential(&mut self, lambda: f64) -> f64 {
        use rand_distr::{Distribution, Exp};
        let exp = Exp::new(lambda).unwrap();
        exp.sample(&mut self.generator.rng)
    }

    pub fn poisson(&mut self, lambda: f64) -> u64 {
        use rand_distr::{Distribution, Poisson};
        let poisson = Poisson::new(lambda).unwrap();
        poisson.sample(&mut self.generator.rng) as u64
    }

    pub fn binomial(&mut self, n: u64, p: f64) -> u64 {
        use rand_distr::{Binomial, Distribution};
        let binomial = Binomial::new(n, p).unwrap();
        binomial.sample(&mut self.generator.rng)
    }

    pub fn geometric(&mut self, p: f64) -> u64 {
        use rand_distr::{Distribution, Geometric};
        let geometric = Geometric::new(p).unwrap();
        geometric.sample(&mut self.generator.rng)
    }
}

impl Default for RandomDistribution {
    fn default() -> Self {
        Self::new()
    }
}

pub fn create_random_generator() -> RandomGenerator {
    RandomGenerator::new()
}

pub fn create_random_generator_with_seed(seed: u64) -> RandomGenerator {
    RandomGenerator::with_seed(seed)
}

pub fn create_weighted_random<T>() -> WeightedRandom<T> {
    WeightedRandom::new()
}

pub fn create_perlin_noise() -> PerlinNoise {
    PerlinNoise::new()
}

pub fn create_perlin_noise_with_seed(seed: u64) -> PerlinNoise {
    PerlinNoise::with_seed(seed)
}

pub fn create_noise_generator() -> NoiseGenerator {
    NoiseGenerator::new()
}

pub fn create_noise_generator_with_seed(seed: u64) -> NoiseGenerator {
    NoiseGenerator::with_seed(seed)
}

pub fn create_random_distribution() -> RandomDistribution {
    RandomDistribution::new()
}

pub fn create_random_distribution_with_seed(seed: u64) -> RandomDistribution {
    RandomDistribution::with_seed(seed)
}

pub fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut rng = create_random_generator();
    rng.gen_bytes(length)
}

pub fn generate_random_bytes_with_seed(length: usize, seed: u64) -> Vec<u8> {
    let mut rng = create_random_generator_with_seed(seed);
    rng.gen_bytes(length)
}

pub fn generate_random_string(length: usize) -> String {
    let mut rng = create_random_generator();
    rng.gen_alphanumeric(length)
}

pub fn generate_random_string_with_seed(length: usize, seed: u64) -> String {
    let mut rng = create_random_generator_with_seed(seed);
    rng.gen_alphanumeric(length)
}

pub fn generate_uuid() -> String {
    let mut rng = create_random_generator();
    rng.gen_uuid()
}

pub fn generate_uuid_with_seed(seed: u64) -> String {
    let mut rng = create_random_generator_with_seed(seed);
    rng.gen_uuid()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_generator() {
        let mut rng = create_random_generator();

        let bytes = rng.gen_bytes(10);
        assert_eq!(bytes.len(), 10);

        let value = rng.gen_range(10, 20);
        assert!(value >= 10 && value < 20);

        let string = rng.gen_alphanumeric(20);
        assert_eq!(string.len(), 20);
        assert!(string.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_random_generator_with_seed() {
        let mut rng1 = create_random_generator_with_seed(42);
        let mut rng2 = create_random_generator_with_seed(42);

        let bytes1 = rng1.gen_bytes(10);
        let bytes2 = rng2.gen_bytes(10);

        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_weighted_random() {
        let mut weighted = create_weighted_random();
        weighted.add_item("a", 1);
        weighted.add_item("b", 2);
        weighted.add_item("c", 3);

        let mut rng = create_random_generator();
        let choice = weighted.choose(&mut rng);
        assert!(choice.is_some());
    }

    #[test]
    fn test_perlin_noise() {
        let noise = create_perlin_noise_with_seed(42);
        let value1 = noise.noise(0.0, 0.0);
        let value2 = noise.noise(1.0, 1.0);

        assert!(value1 >= -1.0 && value1 <= 1.0);
        assert!(value2 >= -1.0 && value2 <= 1.0);
        assert_ne!(value1, value2);
    }

    #[test]
    fn test_noise_generator() {
        let noise_gen = create_noise_generator_with_seed(42);
        let noise_1d = noise_gen.generate_1d(100);
        let noise_2d = noise_gen.generate_2d(10, 10);

        assert_eq!(noise_1d.len(), 100);
        assert_eq!(noise_2d.len(), 100);

        for value in &noise_1d {
            assert!(*value >= 0.0 && *value <= 1.0);
        }

        for value in &noise_2d {
            assert!(*value >= 0.0 && *value <= 1.0);
        }
    }

    #[test]
    fn test_random_distribution() {
        let mut dist = create_random_distribution();

        let uniform = dist.uniform(0.0, 10.0);
        assert!(uniform >= 0.0 && uniform < 10.0);

        let normal = dist.normal(0.0, 1.0);
        assert!(!normal.is_nan());

        let exponential = dist.exponential(1.0);
        assert!(exponential >= 0.0);
    }

    #[test]
    fn test_utility_functions() {
        let bytes = generate_random_bytes_with_seed(10, 42);
        assert_eq!(bytes.len(), 10);

        let string = generate_random_string_with_seed(20, 42);
        assert_eq!(string.len(), 20);

        let uuid = generate_uuid_with_seed(42);
        assert_eq!(uuid.len(), 36);
        assert!(uuid.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
    }
}
