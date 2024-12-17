use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub(crate) struct CircularBuffer<const MAX_SIZE: usize, T> {
    head: usize,
    memory: [Option<T>; MAX_SIZE],
}

impl<const MAX_SIZE: usize, T> CircularBuffer<MAX_SIZE, T> {
    pub fn insert_point(&mut self, new_value: Option<T>) -> Option<T> {
        let current_last_idx = if self.head == 0 {
            MAX_SIZE - 1
        } else {
            self.head - 1
        };
        self.head = current_last_idx;

        std::mem::replace(&mut self.memory[current_last_idx], new_value)
    }

    pub fn fill_with(&mut self, generative_fun: impl Fn(usize) -> Option<T>, num_samples: usize) {
        for i in 0..num_samples {
            self.insert_point(generative_fun(i));
        }
    }

    pub fn valid_entries(&self) -> usize {
        self.memory.iter().filter(|x| x.is_some()).count()
    }
}

impl<const MAX_SIZE: usize, T> CircularBuffer<MAX_SIZE, T>
where
    T: PartialEq,
{
    pub fn entries_matching(&self, value: &T) -> usize {
        self.memory
            .iter()
            .filter(|x| x.as_ref().is_some_and(|x| *x == *value))
            .count()
    }
}

impl<const MAX_SIZE: usize, T> CircularBuffer<MAX_SIZE, T>
where
    T: Copy,
{
    pub fn new() -> Self {
        Self {
            head: 0,
            memory: [None; MAX_SIZE],
        }
    }

    pub fn get(&self, idx: usize) -> Option<T> {
        if idx > MAX_SIZE - 1 {
            return None;
        }
        self.memory[(self.head + idx) % MAX_SIZE]
    }

    pub fn iter(&self) -> impl Iterator<Item = Option<T>> + '_ {
        CircularBufferIter {
            buffer: self,
            idx: 0,
        }
    }

    pub fn clear(&mut self) {
        self.memory.fill(None);
        self.head = 0;
    }

    pub fn fill(&mut self, value_to_fill: Option<T>, num_samples: usize) {
        for _ in 0..num_samples {
            self.insert_point(value_to_fill);
        }
    }
}

impl<const MAX_SIZE: usize> CircularBuffer<MAX_SIZE, f64> {
    pub fn delta(&self, position: usize) -> Option<f64> {
        if position >= MAX_SIZE {
            return None;
        }
        let front = self.memory[self.head];
        let nth = self.memory[(self.head + position) % MAX_SIZE];
        front.zip(nth).map(|(front, nth)| front - nth)
    }

    pub fn moving_avg(&self, len: usize, delay: usize) -> Option<f64> {
        if delay + len >= MAX_SIZE {
            return None;
        }

        let it = self.iter();
        let (sum, count) =
            it.skip(delay)
                .take(len)
                .fold((0.0, 0_usize), |(sum, count), item| -> (f64, usize) {
                    if let Some(val) = item {
                        (sum + val, count + 1)
                    } else {
                        (sum, count)
                    }
                });

        Some(sum / f64::try_from(i32::try_from(count).unwrap()).unwrap())
    }
}

impl<const N: usize, T> Index<usize> for CircularBuffer<N, T> {
    type Output = Option<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.memory[(self.head + index) % N]
    }
}

impl<const N: usize, T> IndexMut<usize> for CircularBuffer<N, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.memory[(self.head + index) % N]
    }
}

impl<const N: usize, T> Default for CircularBuffer<N, T>
where
    T: Copy,
{
    fn default() -> Self {
        Self::new()
    }
}

pub struct CircularBufferIter<'a, const MAX_SIZE: usize, T> {
    buffer: &'a CircularBuffer<MAX_SIZE, T>,
    idx: usize,
}

impl<'a, const MAX_SIZE: usize, T> Iterator for CircularBufferIter<'a, MAX_SIZE, T>
where
    T: Copy,
{
    type Item = Option<T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= MAX_SIZE {
            return None;
        }
        let item = self.buffer.get(self.idx);
        self.idx += 1;
        Some(item)
    }
}
