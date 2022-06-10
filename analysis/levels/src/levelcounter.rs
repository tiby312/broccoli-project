use support::{datanum::DnumManager, prelude::tree::splitter::Splitter};

#[derive(Debug)]
pub struct Single {
    level: usize,
    dur: usize,
}
#[derive(Debug)]
pub struct LevelCounter<'a> {
    man: &'a DnumManager,
    level: usize,
    stuff: Vec<Single>,
    start: usize,
}
impl<'a> LevelCounter<'a> {
    pub fn new(man: &'a DnumManager, level: usize, buffer: Vec<Single>) -> LevelCounter {
        let now = man.counter();
        LevelCounter {
            man,
            level,
            stuff: buffer,
            start: now,
        }
    }

    fn restart(&mut self, level: usize) {
        let now = self.man.counter();
        self.level = level;
        self.start = now;
    }
    pub fn level(&self) -> usize {
        self.level
    }

    pub fn consume(&mut self) {
        let dur = self.man.counter() - self.start;

        //stop self timer.
        let level = self.level();

        if let Some(a) = self.stuff.iter_mut().find(|x| x.level == level) {
            a.dur += dur;
        } else {
            self.stuff.push(Single {
                level: self.level,
                dur,
            });
        }
    }

    pub fn into_levels(mut self) -> Vec<usize> {
        self.consume();
        let mut v: Vec<_> = self.stuff.into_iter().map(|x| x.dur).collect();

        v.reverse();
        let mut n = vec![];

        for i in (0..v.len()).rev() {
            let sum = v[..i + 1].iter().sum();

            n.push(sum);
        }
        n
    }
}
impl<'a> Splitter for LevelCounter<'a> {
    #[inline]
    fn div(&mut self) -> Self {
        let level = self.level();

        self.consume();

        self.restart(level + 1);

        LevelCounter::new(self.man, level + 1, vec![])
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
            let b = v1.iter_mut().find(|x| x.level == a.level).unwrap();
            b.dur += a.dur;
        }

        self.restart(l1 - 1);
    }
}
