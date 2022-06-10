use support::prelude::tree::splitter::Splitter;

use std::time::Instant;
#[derive(Debug)]
pub struct LevelTimer {
    level: usize,
    stuff: Vec<(usize, f64)>,
    start: Instant,
}

impl LevelTimer {
    pub fn level(&self) -> usize {
        self.level
    }
    pub fn new(level: usize, data: Vec<(usize, f64)>) -> LevelTimer {
        LevelTimer {
            level,
            stuff: data,
            start: Instant::now(),
        }
    }

    fn restart(&mut self, level: usize) {
        self.level = level;
        self.start = Instant::now();
    }

    pub fn into_levels(mut self) -> Vec<f64> {
        self.consume();
        //self.consume().into_iter().map(|x| x.1).collect()
        let mut v: Vec<_> = self.stuff.into_iter().map(|x| x.1).collect();

        v.reverse();
        let mut n = vec![];

        for i in (0..v.len()).rev() {
            let sum = v[..i + 1].iter().sum();

            n.push(sum);
        }
        n
    }

    pub fn consume(&mut self) {
        let dur = support::into_secs(self.start.elapsed());
        //stop self timer.
        let level = self.level();
        if let Some(a) = self.stuff.iter_mut().find(|x| x.0 == level) {
            a.1 += dur;
        } else {
            self.stuff.push((self.level, dur));
        }
    }
}

impl Splitter for LevelTimer {
    #[inline]
    fn div(&mut self) -> Self {
        let level = self.level();

        self.consume();

        self.restart(level + 1);
        LevelTimer::new(level + 1, vec![])
    }
    #[inline]
    fn add(&mut self, mut b: Self) {
        let l1 = self.level();
        let l2 = b.level();
        assert_eq!(l1, l2);

        self.consume();
        b.consume();

        let v1 = &mut self.stuff;
        let v2 = &mut b.stuff;

        //the left vec is bigger
        for a in v2.iter_mut() {
            let b = v1.iter_mut().find(|x| x.0 == a.0).unwrap();
            b.1 += a.1;
        }

        self.restart(l1 - 1);
    }
}
