pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}


pub mod bbox_helper{
    use broccoli::Num;
    use broccoli::bbox;
    use axgeom::Rect;
    use broccoli::BBox;

    pub fn create_bbox_mut<T,N:Num>(arr:&mut [T],mut func:impl FnMut(&T)->Rect<N>)->Vec<BBox<N,&mut T>>{
        arr.iter_mut().map(|a|bbox(func(a),a)).collect()
    }
}


mod inner_prelude {
    pub use super::bbox_helper;
    pub use crate::support::*;
    pub(crate) use crate::FigureBuilder;
    pub use broccoli::query::*;
    
    pub use broccoli::*;
    pub use broccoli::analyze::*;
    pub use broccoli::prelude::*;
    pub(crate) use duckduckgeo::bot;
    pub use crate::black_box;
    pub(crate) use crate::datanum;
    pub use ordered_float::NotNan;
    pub use axgeom::vec2;
    pub use axgeom::vec2same;
    pub use axgeom::Rect;
    pub use axgeom::Vec2;
    pub(crate) use duckduckgeo::dists;
    pub use gnuplot::*;
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

impl FigureBuilder {
    fn new(folder: String) -> FigureBuilder {
        FigureBuilder {
            folder,
            last_file_name: None,
        }
    }
    fn build(&mut self, filename: &str) -> Figure {
        let mut fg = Figure::new();
        let ss = format!("{}/{}.gplot", &self.folder, filename);

        //fg.set_terminal("pngcairo size 640,480 enhanced font 'Veranda,10'", "");
        fg.set_terminal("svg", "");

        fg.set_pre_commands(format!("set output sdir.'{}.svg'",filename).as_str());
        //fg.set_pre_commands("set output system(\"echo $FILE_PATH\")");

        //set terminal pngcairo size 350,262 enhanced font 'Verdana,10'
        self.last_file_name = Some(ss);
        fg
    }
    fn finish(&mut self, figure: Figure) {
        figure.echo_to_file(&self.last_file_name.take().unwrap());
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

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get_physical())
        .build_global()
        .unwrap();

    //to run program to generate android bench data.
    //build armv7-linux-androideabi
    //adb -d push broccoli_data /data/local/tmp/dinotree_data
    //adb -d shell pm grant /data/local/tmp/dinotree_data android.permission.WRITE_EXTERNAL_STORAGE
    //adb -d shell /data/local/tmp/dinotree_data bench /sdcard/dinotree/graphs
    //adb -d pull "/sdcard/dinotree/graphs"
    //
    //TODO
    //seperate into benches versus theory runs
    //run benches on laptop/new gaming laptop/android phone/web assembly, and compare differences.
    //

    //println!("{:?}",stringify!(spiral::handle));

    let args: Vec<String> = env::args().collect();
    //assert_eq!(args.len(),2,"First arguent needs to be gen or graph");

    dbg!(&args);
    match args[1].as_ref() {
        "theory" => {
            let folder = args[2].clone();
            let path = Path::new(folder.trim_end_matches('/'));
            std::fs::create_dir_all(&path).expect("failed to create directory");
            let mut fb = FigureBuilder::new(folder);

            
            run_test!(&mut fb, spiral::handle);
            
            run_test!(&mut fb, colfind::colfind::handle_theory);

            run_test!(&mut fb, colfind::construction_vs_query::handle_theory);
            run_test!(&mut fb, colfind::level_analysis::handle_theory);
            
            run_test!(&mut fb, colfind::theory_colfind_3d::handle);
            
            
        }
        "bench" => {
            let folder = args[2].clone();
            let path = Path::new(folder.trim_end_matches('/'));
            std::fs::create_dir_all(&path).expect("failed to create directory");
            let mut fb = FigureBuilder::new(folder);
            
            
            run_test!(&mut fb, colfind::colfind::handle_bench);

            //done
            run_test!(&mut fb, colfind::rebal_strat::handle);
            run_test!(&mut fb, colfind::dinotree_direct_indirect::handle);
            run_test!(&mut fb, colfind::construction_vs_query::handle_bench);
            run_test!(&mut fb, colfind::colfind::handle_bench);
            run_test!(&mut fb, colfind::float_vs_integer::handle);
            run_test!(&mut fb, colfind::level_analysis::handle_bench);
            
            //This is the one thats interesting to see what the results are on phone/vs/laptop
            run_test!(&mut fb, colfind::parallel_heur_comparison::handle);
            run_test!(&mut fb, colfind::height_heur_comparison::handle);
            
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

                        let new_path = path.path().with_extension("svg");
                        let blag = Path::new(new_path.file_name().unwrap().to_str().unwrap());
                        //let file_path = target_dir.join(blag);
                        command
                            .arg("-e")
                            .arg(format!("sdir='{}/'",target_dir.to_str().unwrap()))
                            .arg("-p")
                            .arg(path_command);
                        
                        println!("{:?}",command);
                            //.env("FILE_PATH", file_path.to_str().unwrap());

                        command.status()
                            .expect("Couldn't spawn gnuplot. Make sure it is installed and available in PATH.");
                    }
                }
            }
            //gnuplot -p "colfind_rebal_vs_query_num_bots_grow_of_1.gplot"
            println!("Finished generating graphs");
        }
        _ => {
            println!("Check code to see what it should be");
        }
    }
}
