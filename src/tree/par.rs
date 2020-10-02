///A suggested height at which to switch from parallel
///to sequential. Once the tree construction reaches
///this height, it will no longer call rayon::join(),
///on each sub problem.
pub const SWITCH_SEQUENTIAL_DEFAULT: usize = 6;

///Returns the height at which the recursive construction algorithm turns to sequential from parallel.
#[inline]
pub fn compute_level_switch_sequential(depth: usize, height: usize) -> Parallel {
    let dd = depth;

    let gg = if height <= dd { 0 } else { height - dd };

    Parallel::new(gg)
}

///Returns either two Parallels or two Sequentials.
pub enum ParResult<X, Y> {
    Parallel([X; 2]),
    Sequential([Y; 2]),
}

///Common trait over Parallel and Sequential to make writing generic code easier.
pub trait Joiner: Sized + Send + Sync {
    fn next(self) -> ParResult<Self, Sequential>;
}

///Indicates that an algorithm should run in parallel up until
///the specified height.
#[derive(Copy, Clone)]
pub struct Parallel {
    depth_to_switch_at: usize,
    current_depth: usize,
}
impl Parallel {
    ///The depth at which to switch to sequential.
    pub fn new(depth_to_switch_at: usize) -> Self {
        Parallel {
            depth_to_switch_at,
            current_depth: 0,
        }
    }

    pub fn get_depth_to_switch_at(&self) -> usize {
        self.depth_to_switch_at
    }

    pub fn get_current_depth(&self) -> usize {
        self.current_depth
    }
}

impl Joiner for Parallel {
    fn next(mut self) -> ParResult<Self, Sequential> {
        if self.current_depth >= self.depth_to_switch_at {
            ParResult::Sequential([Sequential, Sequential])
        } else {
            self.current_depth += 1;
            ParResult::Parallel([self; 2])
        }
    }
}

///Indicates that an algorithm should run sequentially.
///Once we transition to sequential, we always want to recurse sequentially.
#[derive(Copy, Clone)]
pub struct Sequential;
impl Joiner for Sequential {
    fn next(self) -> ParResult<Self, Sequential> {
        ParResult::Sequential([Sequential, Sequential])
    }
}
