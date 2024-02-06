use std::rc::Rc;
use rand::prelude::{SliceRandom, ThreadRng};

pub struct BagRandom<T> {
    rng: ThreadRng,
    sample: Vec<Rc<T>>,
    bag: Vec<usize>
}

impl<T> BagRandom<T> {
    pub fn new(sample: Vec<T>) -> Self {
        assert!(!sample.is_empty());
        let sample = sample.into_iter().map(|t| Rc::new(t)).collect();
        Self { rng: Default::default(), sample, bag: vec![] }
    }
}

impl<T> Iterator for BagRandom<T> {
    type Item = Rc<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.bag.pop() {
            return self.sample.get(index).map(|t| t.clone());
        }

        self.bag = (0..self.sample.len()).collect();
        self.bag.shuffle(&mut self.rng);
        self.sample.get(self.bag.pop().unwrap()).map(|t| t.clone())
    }
}