use crate::inner_prelude::*;
use broccoli::pmut::PMut;

pub trait TestTrait: Copy + Send + Sync {}
impl<T: Copy + Send + Sync> TestTrait for T {}

#[derive(Copy, Clone)]
pub struct Bot<T> {
    num: usize,
    aabb: Rect<i32>,
    _val: T,
}

#[derive(Copy, Clone, Debug)]
pub struct TestResult {
    rebal: f64,
    query: f64,
}

fn test_seq<T: Aabb>(bots: &mut [T], func: impl Fn(PMut<T>, PMut<T>)) -> TestResult {
    let (mut tree, construct_time) = bench_closure_ret(|| broccoli::new(bots));

    let (tree, query_time) = bench_closure_ret(|| {
        tree.find_colliding_pairs_mut(|a, b| {
            func(a, b);
        });
        tree
    });

    black_box(tree);

    TestResult {
        rebal: construct_time,
        query: query_time,
    }
}
fn test_par<T: Aabb + Send + Sync>(
    bots: &mut [T],
    func: impl Fn(PMut<T>, PMut<T>) + Send + Sync,
) -> TestResult {
    let (mut tree, construct_time) = bench_closure_ret(|| broccoli::new_par(bots));

    let (tree, query_time) = bench_closure_ret(|| {
        tree.find_colliding_pairs_mut_par(|a, b| {
            func(a, b);
        });
        tree
    });

    black_box(tree);

    TestResult {
        rebal: construct_time,
        query: query_time,
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CompleteTestResult {
    direct_seq: TestResult,
    direct_par: TestResult,

    indirect_seq: TestResult,
    indirect_par: TestResult,

    default_seq: TestResult,
    default_par: TestResult,
}
impl CompleteTestResult {
    fn into_arr(self) -> [TestResult; 6] {
        [
            self.direct_seq,
            self.direct_par,
            self.indirect_seq,
            self.indirect_par,
            self.default_seq,
            self.default_par,
        ]
    }
}

fn complete_test<T: TestTrait>(num_bots: usize, grow: f64, t: T) -> CompleteTestResult {
    let (direct_seq, direct_par) = {
        let mut bots: Vec<_> = abspiral_f64(grow as f64)
            .take(num_bots)
            .map(|a| a.inner_as::<i32>())
            .map(|a| bbox(a, (0isize, t)))
            .collect();

        let collide = |mut b: PMut<BBox<i32, (isize, T)>>, mut c: PMut<BBox<i32, (isize, T)>>| {
            b.inner_mut().0 += 1;
            c.inner_mut().0 += 1;
        };

        (test_seq(&mut bots, collide), test_par(&mut bots, collide))
    };

    let (indirect_seq, indirect_par) = {
        let mut bots: Vec<_> = abspiral_f64(grow as f64)
            .take(num_bots)
            .map(|a| a.inner_as::<i32>())
            .map(|a| bbox(a, (0isize, t)))
            .collect();
        let mut indirect: Vec<_> = bots.iter_mut().collect();

        let collide = |mut b: PMut<&mut BBox<i32, (isize, T)>>,
                       mut c: PMut<&mut BBox<i32, (isize, T)>>| {
            b.inner_mut().0 += 1;
            c.inner_mut().0 += 1;
        };

        (
            test_seq(&mut indirect, collide),
            test_par(&mut indirect, collide),
        )
    };
    let (default_seq, default_par) = {
        let mut bot_inner: Vec<_> = (0..num_bots).map(|_| (0isize, t)).collect();
        let mut default: Vec<BBox<i32, &mut (isize, T)>> = abspiral_f64(grow)
            .take(num_bots)
            .map(|a| a.inner_as::<i32>())
            .zip(bot_inner.iter_mut())
            .map(|(a, b)| bbox(a, b))
            .collect();

        let collide = |mut b: PMut<BBox<i32, &mut (isize, T)>>,
                       mut c: PMut<BBox<i32, &mut (isize, T)>>| {
            b.inner_mut().0 += 1;
            c.inner_mut().0 += 1;
        };

        (
            test_seq(&mut default, collide),
            test_par(&mut default, collide),
        )
    };

    CompleteTestResult {
        direct_seq,
        direct_par,
        indirect_seq,
        indirect_par,
        default_seq,
        default_par,
    }
}

pub fn handle(fb: &mut FigureBuilder) {
    handle_num_bots(fb, 0.1, [0u8; 8]);
    handle_num_bots(fb, 0.1, [0u8; 16]);
    handle_num_bots(fb, 0.1, [0u8; 32]);
    handle_num_bots(fb, 0.1, [0u8; 128]);
    handle_num_bots(fb, 0.1, [0u8; 256]);

    handle_num_bots(fb, 0.01, [0u8; 128]);
    handle_num_bots(fb, 1.0, [0u8; 128]);
}

#[derive(Debug)]
struct Record {
    num_bots: usize,
    arr: CompleteTestResult,
}
impl Record {
    fn draw(
        records: &[Record],
        fg: &mut Figure,
        grow: f64,
        construction: &str,
        name: &str,
        func: impl Fn(TestResult) -> f64,
    ) {
        const NAMES: &[&str] = &[
            "direct seq",
            "direct par",
            "indirect seq",
            "indirect par",
            "default seq",
            "default par",
        ];
        {
            let k = fg
                .axes2d()
                .set_title(
                    &format!(
                        "{} Dinotree vs Direct vs Indirect with abspiral-size(x, {}, {})",
                        construction, grow, name
                    ),
                    &[],
                )
                .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
                .set_x_label("Number of Elements", &[])
                .set_y_label("Time in Seconds", &[]);

            let x = records.iter().map(|a| a.num_bots);
            for index in 0..6 {
                let y = records.iter().map(|a| func(a.arr.into_arr()[index]));
                k.lines(
                    x.clone(),
                    y,
                    &[Caption(NAMES[index]), Color(COLS[index]), LineWidth(1.0)],
                );
            }
        }
    }
}

fn handle_num_bots<T: TestTrait>(fb: &mut FigureBuilder, grow: f64, val: T) {
    let mut rects = Vec::new();

    for num_bots in (0..30_000).rev().step_by(200) {
        let r = Record {
            num_bots,
            arr: complete_test(num_bots, grow, val.clone()),
        };
        rects.push(r);
    }
    let name = format!("{}_bytes", core::mem::size_of::<T>());
    let name2 = format!("{} bytes", core::mem::size_of::<T>());

    let mut fg = fb.build(&format!("dinotree_direct_indirect_rebal_{}_{}", grow, name));
    Record::draw(&rects, &mut fg, grow, "Construction:", &name2, |a| a.rebal);
    fb.finish(fg);

    let mut fg = fb.build(&format!("dinotree_direct_indirect_query_{}_{}", grow, name));
    Record::draw(&rects, &mut fg, grow, "Querying:", &name2, |a| a.query);
    fb.finish(fg);
}
