pub use broccoli::axgeom;

pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

pub mod bbox_helper {
    use broccoli::axgeom::Rect;
    use broccoli::{bbox, node::*};
    pub fn create_bbox_mut<T, N: Num>(
        arr: &mut [T],
        mut func: impl FnMut(&T) -> Rect<N>,
    ) -> Vec<BBox<N, &mut T>> {
        arr.iter_mut().map(|a| bbox(func(a), a)).collect()
    }
}

mod inner_prelude {
    pub use super::bbox_helper;
    pub use crate::black_box;
    pub(crate) use crate::datanum;
    pub use crate::support::*;
    pub use crate::Args;
    pub(crate) use crate::FigureBuilder;
    pub use axgeom::vec2;
    pub use axgeom::vec2same;
    pub use axgeom::Rect;
    pub use axgeom::Vec2;
    pub use broccoli::build::*;
    pub use broccoli::node::*;
    pub use broccoli::par::*;
    pub use broccoli::pmut::PMut;
    pub use broccoli::query::*;
    pub use broccoli::*;
    pub use poloto::prelude::*;
    pub use poloto::*;
    pub use serde::Serialize;
    pub use std::time::Duration;
    pub use std::time::Instant;
    pub use tagger;
}

#[macro_use]
mod support;
mod colfind;
pub(crate) mod datanum;
mod spiral;

use std::env;

pub struct FigureBuilder {
    folder: String,
}
use serde::Serialize;

pub struct Args<'a, S: Serialize, I: Iterator<Item = (f64, S)>> {
    filename: &'a str,
    title: &'a str,
    xname: &'a str,
    yname: &'a str,
    plots: I,
    stop_values: &'a [(&'a str, f64)],
}

impl FigureBuilder {
    fn new(folder: String) -> FigureBuilder {
        FigureBuilder { folder }
    }

    pub fn canvas(&self) -> poloto::render::CanvasBuilder {
        poloto::render::canvas_builder()
    }

    fn finish_plot<K: poloto::render::Disp>(
        &self,
        mut plot: poloto::render::Plotter<K>,
        filename: impl core::fmt::Display,
    ) {
        let s = format!("{}/{}.svg", &self.folder, filename);
        let mut file = std::fs::File::create(s).unwrap();

        let mut e = poloto::upgrade_write(&mut file);

        use std::fmt::Write;
        write!(&mut e, "{}", poloto::simple_theme::SVG_HEADER).unwrap();
        plot.render(&mut e).unwrap();
        write!(&mut e, "{}", poloto::simple_theme::SVG_END).unwrap();
    }

    fn make_graph<S: Serialize, I: Iterator<Item = (f64, S)>>(&mut self, args: Args<S, I>) {
        let it = args.plots;
        let filename = args.filename;
        let title = args.title;
        let xname = args.xname;
        let yname = args.yname;
        let stop_values = args.stop_values;

        let rects: Vec<_> = it.collect();
        let mut ii = rects.iter();

        struct MySerialize {
            value: serde_json::Value,
        }

        impl MySerialize {
            fn new<S: Serialize>(s: &S) -> Self {
                let serialized = serde_json::to_string(s).unwrap();
                let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
                MySerialize { value }
            }
            fn as_object(&self) -> &serde_json::map::Map<String, serde_json::value::Value> {
                self.value.as_object().unwrap()
            }
        }

        if let Some(ff) = ii.next() {
            let map = MySerialize::new(&ff.1);

            let names = map.as_object().clone();

            let mut data = vec![];
            for (plot_name, _) in names.iter() {
                let k = ii.clone();
                let stop_val = stop_values.iter().find(|a| a.0.eq(plot_name)).map(|a| a.1);
                data.push(poloto::build::line(
                    plot_name,
                    core::iter::once(ff)
                        .chain(k)
                        .filter(move |(secondx, _)| {
                            if let Some(stop_val) = stop_val {
                                *secondx <= stop_val
                            } else {
                                true
                            }
                        })
                        .map(move |(secondx, foo)| {
                            let map = MySerialize::new(foo);
                            let num: f64 = match &map.as_object()[plot_name] {
                                serde_json::Value::Number(val) => val.as_f64().unwrap(),
                                _ => {
                                    panic!("not a number")
                                }
                            };

                            [*secondx, num]
                        }),
                ));
            }
            self.finish_plot(
                self.canvas()
                    .build()
                    .build_with(poloto::build::plots_dyn(data), [], [0.0])
                    .plot(title, xname, yname),
                filename,
            );
        }
    }
}

use std::io::Write;
use std::path::Path;
use std::time::*;

fn into_secs(elapsed: std::time::Duration) -> f64 {
    (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0)
}

// This is a simple macro named `say_hello`.
macro_rules! run_test {
    // `()` indicates that the macro takes no argument.
    ($builder:expr,$test:expr) => {
        // The macro will expand into the contents of this block.
        print!("Running {}...", stringify!($test));
        std::io::stdout().flush().unwrap();
        let time = Instant::now();
        $test($builder);
        let val = into_secs(time.elapsed());
        println!("finished in {} seconds.", val);
        //Give benches some time to cool down.
        std::thread::sleep(std::time::Duration::from_millis(3000));
    };
}

fn profile_test(num_bots: usize) {
    let grow = DEFAULT_GROW;
    //let num_bots = 50_000;
    use crate::support::*;
    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

    let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f64n());

    for _ in 0..30 {
        let mut num_collision = 0;
        let c0 = bench_closure(|| {
            let mut tree = broccoli::new(&mut bots);
            tree.find_colliding_pairs_mut(|a, b| {
                num_collision += 1;
                **a.unpack_inner() += 1;
                **b.unpack_inner() += 1;
            });
        });

        dbg!(c0, num_collision);
    }
}
fn main() {
    rayon_core::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get_physical())
        .build_global()
        .unwrap();

    //profile_test();
    //return;

    //to run program to generate android bench data.
    //build armv7-linux-androideabi
    //adb -d push broccoli_data /data/local/tmp/broccoli_data
    //adb -d shell pm grant /data/local/tmp/broccoli_data android.permission.WRITE_EXTERNAL_STORAGE
    //adb -d shell /data/local/tmp/broccoli_data bench /sdcard/broccoli/graphs
    //adb -d pull "/sdcard/broccoli/graphs"
    //
    //TODO
    //separate into benches versus theory runs
    //run benches on laptop/new gaming laptop/android phone/web assembly, and compare differences.
    //

    let args: Vec<String> = env::args().collect();

    dbg!(&args);
    match args[1].as_ref() {
        "profile" => {
            profile_test(args[2].parse().unwrap());
        }
        "profile_cmp" => {
            let grow = DEFAULT_GROW;
            let num_bots = 20_000;
            use crate::support::*;
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

            for _ in 0..30 {
                let c0 = datanum::datanum_test(|maker| {
                    let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));

                    let mut tree = broccoli::new(&mut bots);
                    let mut num_collide = 0;

                    tree.find_colliding_pairs_mut(|a, b| {
                        **a.unpack_inner() += 1;
                        **b.unpack_inner() += 1;
                        num_collide += 1;
                    });
                    dbg!(num_collide);
                });

                dbg!(c0);
            }
        }
        "theory" => {
            let folder = args[2].clone();
            let path = Path::new(folder.trim_end_matches('/'));
            std::fs::create_dir_all(&path).expect("failed to create directory");
            let mut fb = FigureBuilder::new(folder);
            run_test!(&mut fb, spiral::handle);
            run_test!(&mut fb, colfind::colfind::handle_theory);
            run_test!(&mut fb, colfind::construction_vs_query::handle_theory);
            run_test!(&mut fb, colfind::level_analysis::handle_theory);
            run_test!(&mut fb, colfind::query_evenness::handle_theory);
        }
        "bench" => {
            let folder = args[2].clone();
            let path = Path::new(folder.trim_end_matches('/'));
            std::fs::create_dir_all(&path).expect("failed to create directory");
            let mut fb = FigureBuilder::new(folder);

            run_test!(&mut fb, colfind::colfind::handle_bench);
            run_test!(&mut fb, colfind::construction_vs_query::handle_bench);
            run_test!(&mut fb, colfind::level_analysis::handle_bench);
            run_test!(&mut fb, colfind::float_vs_integer::handle);
            run_test!(&mut fb, colfind::height_heur_comparison::handle);
            run_test!(&mut fb, colfind::optimal_query::handle);
            run_test!(&mut fb, colfind::parallel_heur_comparison::handle);
            run_test!(&mut fb, colfind::tree_direct_indirect::handle);
        }
        _ => {
            println!("Check code to see what it should be");
        }
    }
}
