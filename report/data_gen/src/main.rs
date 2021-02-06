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
    pub use crate::support::bool_then;
    pub use crate::support::*;
    pub use crate::Args;
    pub(crate) use crate::FigureBuilder;
    pub use axgeom::vec2;
    pub use axgeom::vec2same;
    pub use axgeom::Rect;
    pub use axgeom::Vec2;
    pub use broccoli::build::*;
    pub use broccoli::node::*;
    pub use broccoli::pmut::PMut;
    pub use broccoli::prelude::*;
    pub use broccoli::query::colfind::NotSortedQueries;
    pub use broccoli::query::*;
    pub use broccoli::RayonJoin;
    pub use broccoli::*;
    pub use gnuplot::*;
    pub use serde::Serialize;
    pub use std::time::Duration;
    pub use std::time::Instant;
}

#[macro_use]
mod support;
mod colfind;
pub(crate) mod datanum;
mod spiral;

use gnuplot::*;
use std::env;

pub struct FigureBuilder {
    folder: String,
    last_file_name: Option<String>,
}
use serde::Serialize;

pub struct Args<'a, S: Serialize, I: Iterator<Item = (f32, S)>> {
    filename: &'a str,
    title: &'a str,
    xname: &'a str,
    yname: &'a str,
    plots: I,
    stop_values: &'a [(&'a str, f32)],
}

impl FigureBuilder {
    fn new(folder: String) -> FigureBuilder {
        FigureBuilder {
            folder,
            last_file_name: None,
        }
    }

    fn finish_plot(&self, splot: plotato::Plotter, filename: &str) {
        let s = format!("{}/{}.svg", &self.folder, filename);
        splot.render_to_file(&s).unwrap()
    }

    fn build(&mut self, filename: &str) -> Figure {
        let mut fg = Figure::new();
        let ss = format!("{}/{}.gplot", &self.folder, filename);

        //fg.set_terminal("pngcairo size 640,480 enhanced font 'Veranda,10'", "");
        fg.set_terminal("svg enhanced background rgb '#e1e1db'", "");

        fg.set_pre_commands(format!("set output sdir.'{}.svg'", filename).as_str());
        //fg.set_pre_commands("set output system(\"echo $FILE_PATH\")");

        //set terminal pngcairo size 350,262 enhanced font 'Verdana,10'
        self.last_file_name = Some(ss);
        fg
    }
    fn finish(&mut self, figure: Figure) {
        figure.echo_to_file(&self.last_file_name.take().unwrap());
    }
    fn make_graph<S: Serialize, I: Iterator<Item = (f32, S)>>(&mut self, args: Args<S, I>) {
        let it = args.plots;
        let filename = args.filename;
        let title = args.title;
        let xname = args.xname;
        let yname = args.yname;
        let stop_values = args.stop_values;

        use core::convert::TryInto;

        let mut rects: Vec<_> = it.collect();
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
            let num_plots = map.as_object().len();

            let names = map.as_object().clone();

            let mut plot = plotato::plot(title, xname, yname);

            for (plot_name, _) in names.iter() {
                let k = ii.clone();
                let stop_val = stop_values.iter().find(|a| a.0.eq(plot_name)).map(|a| a.1);

                plot.line(
                    plot_name,
                    core::iter::once(ff)
                        .chain(k)
                        .filter(move |(secondx, _)| {
                            if let Some(stop_val) = stop_val {
                                *secondx < stop_val
                            } else {
                                true
                            }
                        })
                        .map(move |(secondx, foo)| {
                            let mapp = MySerialize::new(foo);
                            let num: f32 = match &mapp.as_object()[plot_name] {
                                serde_json::Value::Number(val) => val.as_f64().unwrap() as f32,
                                _ => {
                                    panic!("not a number")
                                }
                            };

                            [*secondx, num]
                        }),
                );
            }

            self.finish_plot(plot, filename);
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
    ($fo:expr,$tre:expr) => {
        // The macro will expand into the contents of this block.
        print!("Running {}...", stringify!($tre));
        std::io::stdout().flush().unwrap();
        let time = Instant::now();
        $tre($fo);
        let val = into_secs(time.elapsed());
        println!("finished in {} seconds.", val);
        //Give benches some time to cool down.
        std::thread::sleep(std::time::Duration::from_millis(3000));
    };
}

fn profile_test() {
    let grow = 0.2;
    let num_bots = 50_000;
    use crate::support::*;
    use broccoli::prelude::*;
    let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

    let mut bots = distribute(grow, &mut bot_inner, |a| a.to_f32n());

    for _ in 0..30 {
        let c0 = bench_closure(|| {
            let mut tree = broccoli::new(&mut bots);
            tree.find_colliding_pairs_mut(|a, b| {
                **a.unpack_inner() += 1;
                **b.unpack_inner() += 1;
            });
        });

        dbg!(c0);
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
    //seperate into benches versus theory runs
    //run benches on laptop/new gaming laptop/android phone/web assembly, and compare differences.
    //

    let args: Vec<String> = env::args().collect();

    dbg!(&args);
    match args[1].as_ref() {
        "profile" => {
            profile_test();
        }
        "profile_cmp" => {
            let grow = 0.2;
            let num_bots = 50_000;
            use crate::support::*;
            let mut bot_inner: Vec<_> = (0..num_bots).map(|_| 0isize).collect();

            for _ in 0..30 {
                let c0 = datanum::datanum_test(|maker| {
                    let mut bots = distribute(grow, &mut bot_inner, |a| a.to_isize_dnum(maker));
                    use broccoli::prelude::*;

                    let mut tree = broccoli::new(&mut bots);
                    tree.find_colliding_pairs_mut(|a, b| {
                        **a.unpack_inner() += 1;
                        **b.unpack_inner() += 1;
                    });
                });

                dbg!(c0);
            }
        }
        "theory" => {
            let folder = args[2].clone();
            let path = Path::new(folder.trim_end_matches('/'));
            std::fs::create_dir_all(&path).expect("failed to create directory");
            let mut fb = FigureBuilder::new(folder);
            //run_test!(&mut fb, colfind::colfind::handle_theory);
            run_test!(&mut fb, colfind::construction_vs_query::handle_theory);

            /*
            run_test!(&mut fb, colfind::query_evenness::handle_num_node);
            run_test!(&mut fb, colfind::query_evenness::handle_theory);

            run_test!(&mut fb, colfind::level_analysis::handle_theory);


            run_test!(&mut fb, spiral::handle);

            run_test!(&mut fb, colfind::construction_vs_query::handle_theory);

            run_test!(&mut fb, colfind::theory_colfind_3d::handle);
            */
        }
        "bench" => {
            let folder = args[2].clone();
            let path = Path::new(folder.trim_end_matches('/'));
            std::fs::create_dir_all(&path).expect("failed to create directory");
            let mut fb = FigureBuilder::new(folder);
            //run_test!(&mut fb, colfind::colfind::handle_bench);
            //run_test!(&mut fb, colfind::construction_vs_query::handle_bench);

            /*
            run_test!(&mut fb, colfind::optimal_query::handle);
            run_test!(&mut fb, colfind::level_analysis::handle_bench);
            run_test!(&mut fb, colfind::parallel_heur_comparison::handle);

            //done
            run_test!(&mut fb, colfind::rebal_strat::handle);


            run_test!(&mut fb, colfind::tree_direct_indirect::handle);

            run_test!(&mut fb, colfind::float_vs_integer::handle);

            //This is the one thats interesting to see what the results are on phone/vs/laptop

            run_test!(&mut fb, colfind::height_heur_comparison::handle);
            */
            //nbody::theory::handle(&mut fb);
        }
        "graph" => {
            let folder = args[2].clone();

            let path = Path::new(folder.trim_end_matches('/'));

            let target_folder = args[3].clone();
            let target_dir = Path::new(target_folder.trim_end_matches('/'));
            std::fs::create_dir_all(&target_dir).expect("failed to create directory");

            let paths = std::fs::read_dir(path).unwrap();

            for path in paths {
                let path = match path {
                    Ok(path) => path,
                    _ => continue,
                };

                if let Some(ext) = path.path().extension() {
                    if ext == "gplot" {
                        let path_command = path.path();
                        println!("generating {:?}", path.file_name());

                        //let output=format!("-e \"output='{}' \"",path.path().with_extension("png").to_str().unwrap());
                        //gnuplot -e "filename='foo.data'" foo.plg

                        let mut command = std::process::Command::new("gnuplot");

                        //let new_path = path.path().with_extension("svg");
                        //let blag = Path::new(new_path.file_name().unwrap().to_str().unwrap());
                        command
                            .arg("-e")
                            .arg(format!("sdir='{}/'", target_dir.to_str().unwrap()))
                            .arg("-p")
                            .arg(path_command);

                        println!("{:?}", command);

                        command.status()
                            .expect("Couldn't spawn gnuplot. Make sure it is installed and available in PATH.");
                    }
                }
            }
            println!("Finished generating graphs");
        }
        _ => {
            println!("Check code to see what it should be");
        }
    }
}
