use std::collections::vec_deque;
use std::iter::{Enumerate, Flatten, Skip, Take};
use std::{
    collections::{VecDeque, vec_deque::IterMut},
    iter,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Packet {
    class: u8,
    flow: u16,
    payload: u64,
}

impl Packet {
    pub fn new(payload: u64, class: u8, flow: u16) -> Result<Self, String> {
        if class > 7 {
            Err(format!("Class cannot be greater than 7: Found {class}"))
        } else {
            Ok(Self {
                payload,
                class,
                flow,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scheduler {
    queues: [VecDeque<Packet>; 8],
    weights: [usize; 8],
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            queues: Default::default(),
            weights: [1, 1, 1, 1, 1, 1, 1, 1], // zero weight will drop the queue entirely
        }
    }
}

impl Scheduler {
    pub fn enqueue(&mut self, packets: &[Packet]) {
        for packet in packets {
            self.queues[packet.class as usize].push_back(*packet);
        }
    }

    pub fn new(weights: [usize; 8]) -> Self {
        Self {
            queues: Default::default(),
            weights,
        }
    }

    pub fn iter<'s>(&'s self) -> SchedulerIter<'s> {
        SchedulerIter {
            scheduler: &self.queues,
            weights: self.weights,
            iterator: None,
            index: 0,
        }
    }

    pub fn iter_mut<'s>(&'s mut self) -> SchedulerIterMut<'s> {
        SchedulerIterMut {
            scheduler: self
                .queues
                .iter_mut()
                .map(|queue| queue.iter_mut())
                .collect(),
            buffer: Default::default(),
            weights: self.weights,
        }
    }
}

impl<'a> IntoIterator for &'a Scheduler {
    type Item = &'a Packet;

    type IntoIter = SchedulerIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a> IntoIterator for &'a mut Scheduler {
    type Item = &'a mut Packet;

    type IntoIter = SchedulerIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl IntoIterator for Scheduler {
    type Item = Packet;
    type IntoIter = SchedulerIntoIter;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            scheduler: self,
            buffer: Default::default(),
        }
    }
}

type PacketIter<'a> = Take<Skip<vec_deque::Iter<'a, Packet>>>;

struct CustomMap<'a, IT: Iterator<Item = (usize, &'a VecDeque<Packet>)>> {
    it: IT,
    weights: [usize; 8],
    index: usize,
}

impl<'a, IT: Iterator<Item = (usize, &'a VecDeque<Packet>)>> Iterator for CustomMap<'a, IT> {
    type Item = PacketIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, queue) = self.it.next()?;
        Some(
            queue
                .iter()
                .skip(self.weights[index] * self.index)
                .take(self.weights[index]),
        )
    }
}

pub struct SchedulerIter<'a> {
    scheduler: &'a [VecDeque<Packet>],
    iterator: Option<Flatten<CustomMap<'a, Enumerate<std::slice::Iter<'a, VecDeque<Packet>>>>>>,
    weights: [usize; 8],
    index: usize,
}

impl<'a> Iterator for SchedulerIter<'a> {
    type Item = &'a Packet;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iterator.iter_mut().flatten().next() {
            None => {
                let enumerated = self.scheduler.iter().enumerate();

                let mapped = CustomMap {
                    it: enumerated,
                    index: self.index,
                    weights: self.weights,
                };

                self.iterator = Some(mapped.flatten());

                self.index += 1;

                self.iterator.iter_mut().flatten().next()
            }
            Some(value) => Some(value),
        }
    }
}
pub struct SchedulerIterMut<'a> {
    scheduler: Vec<IterMut<'a, Packet>>,
    buffer: VecDeque<&'a mut Packet>,
    weights: [usize; 8],
}

impl<'a> Iterator for SchedulerIterMut<'a> {
    type Item = &'a mut Packet;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            self.buffer.append(
                &mut self
                    .scheduler
                    .iter_mut()
                    .enumerate()
                    .flat_map(|(index, queue)| {
                        iter::repeat_with(|| queue.next())
                            .take(self.weights[index])
                            .flatten()
                    })
                    .collect(),
            );
        }
        self.buffer.pop_front()
    }
}

pub struct SchedulerIntoIter {
    scheduler: Scheduler,
    buffer: VecDeque<Packet>,
}

impl Iterator for SchedulerIntoIter {
    type Item = Packet;

    fn next(&mut self) -> Option<Self::Item> {
        let queues = &mut self.scheduler.queues;
        let weights = self.scheduler.weights;
        if self.buffer.is_empty() {
            self.buffer.append(
                &mut queues
                    .iter_mut()
                    .enumerate()
                    .flat_map(|(index, queue)| {
                        iter::repeat_with(|| queue.pop_front())
                            .take(weights[index])
                            .flatten()
                    })
                    .collect(),
            );
        }
        self.buffer.pop_front()
    }
}

#[cfg(test)]
mod tests;
