//!Contains code to write generic code that can be run in parallel, or sequentially. The api is exposed
//!in case users find it useful when writing parallel query code to operate on the tree.

///A suggested height at which to switch from parallel
///to sequential. Once the tree construction reaches
///this height, it will no longer call `rayon::join()`,
///on each sub problem.
pub const SWITCH_SEQUENTIAL_DEFAULT: usize = 6;

pub struct ParallelBuilder{
    height_switch:usize
}
impl ParallelBuilder{
    pub fn new()->Self{
        ParallelBuilder{height_switch:SWITCH_SEQUENTIAL_DEFAULT}
    }
    pub fn with_switch_height(&mut self,height:usize){
        self.height_switch=height;
    }

    pub fn build_for_tree_of_height(&self,tree_height:usize)->Parallel{
        Parallel::new(if tree_height<self.height_switch{
            0
        }else{
            tree_height-self.height_switch
        })
    }
    
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
    const fn new(depth_to_switch_at: usize) -> Self {
        Parallel {
            depth_to_switch_at,
            current_depth: 0,
        }
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

