use crate::inner_prelude::*;
use dinotree_alg::nbody;
use duckduckgeo;

#[derive(Copy,Clone)]
pub struct Bot{
    pos:[f64;2],
    //vel:[f64;2],
    force:[f64;2],
    mass:f64
}
impl Bot{
    fn apply_force(&mut self,a:[f64;2]){
        self.force[0]+=a[0];
        self.force[1]+=a[1];
    }
    /*
    fn handle(&mut self){
        
        let b=self;

        b.pos[0]+=b.vel[0];
        b.pos[1]+=b.vel[1];
    
        
        //F=MA
        //A=F/M
        let accx=b.force[0]/b.mass;
        let accy=b.force[1]/b.mass;

        b.vel[0]+=accx;
        b.vel[1]+=accy;            

        

        b.force=[0.0;2];
    }
    */
    fn create_aabb(&self)->axgeom::Rect<F64n>{
        let r=5.0f64.min(self.mass.sqrt()/10.0);
        ConvF64::from_rect(aabb_from_pointf64(self.pos,[r;2]))             
    }
}

impl duckduckgeo::GravityTrait for Bot{
    type N=f64;
    fn pos(&self)->[f64;2]{
        self.pos
    }
    fn mass(&self)->f64{
        self.mass
    }
    fn apply_force(&mut self,a:[f64;2]){
        self.force[0]+=a[0];
        self.force[1]+=a[1];
    }
}



fn generate_bot_from_spiral(spiral:&dists::spiral::Spiral,num_bots:usize)->Vec<Bot>{
    let bots:Vec<Bot>=spiral.take(num_bots).map(|pos|{
                let pos=[pos[0] ,pos[1] ];
                //let vel=[0.0;2];
                let force=[0.0;2];
                let mass=1.0;
                Bot{pos,force,mass}
            }).collect();
    bots
}

mod go{

    use super::*;

    #[derive(Copy,Clone)]
    pub struct NodeMass{
        rect:axgeom::Rect<F64n>,
        center:[f64;2],
        mass:f64,
        force:[f64;2]
    }
    impl NodeMass{
        pub fn new()->NodeMass{
            let a=f64n!(0.0);
            NodeMass{
                rect:axgeom::Rect::new(a,a,a,a),
                center:[0.0;2],
                mass:0.0,
                force:[0.0;2]
            }
        }
    }

    impl duckduckgeo::GravityTrait for NodeMass{
        type N=f64;
        fn pos(&self)->[f64;2]{
            self.center
        }
        fn mass(&self)->f64{
            self.mass
        }
        fn apply_force(&mut self,a:[f64;2]){
            self.force[0]+=a[0];
            self.force[1]+=a[1];
        }
    }


    #[derive(Clone,Copy)]
    pub struct Bla{
        pub calls_to_gravitate:usize,
        pub dis:f64,
        pub dis_half:f64
    }
    impl nbody::NodeMassTraitMut for Bla{
        type T=BBox<F64n,Bot>;
        type No=NodeMass;

        fn get_rect(a:&Self::No)->&axgeom::Rect<F64n>{
            &a.rect
        }

        //gravitate this nodemass with another node mass
        fn handle_node_with_node(&mut self,a:&mut Self::No,b:&mut Self::No){
            self.calls_to_gravitate+=1;
            let _ = duckduckgeo::gravitate(a,b,0.0001,0.004,|a|a.sqrt());
        }

        //gravitate a bot with a bot
        fn handle_bot_with_bot(&mut self,a:&mut Self::T,b:&mut Self::T){
            self.calls_to_gravitate+=1;
            let _ = duckduckgeo::gravitate(&mut a.inner,&mut b.inner,0.0001,0.004,|a|a.sqrt());
        }

        //gravitate a nodemass with a bot
        fn handle_node_with_bot(&mut self,a:&mut Self::No,b:&mut Self::T){
            self.calls_to_gravitate+=1;
            let _ = duckduckgeo::gravitate(a,&mut b.inner,0.0001,0.004,|a|a.sqrt());
        }


        fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a mut self,it:I,rect:axgeom::Rect<F64n>)->Self::No{
            let mut total_x=0.0;
            let mut total_y=0.0;
            let mut total_mass=0.0;

            for i in it{
                let m=i.inner.mass;
                total_mass+=m;
                total_x+=m*i.inner.pos[0];
                total_y+=m*i.inner.pos[1];
            }
            
            let center=if total_mass!=0.0{
                [total_x/total_mass,
                total_y/total_mass]
            }else{
                [0.0;2]
            };
            NodeMass{center,mass:total_mass,force:[0.0;2],rect}
        }

        fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a mut self,a:&'a Self::No,it:I){

            if a.mass>0.0000001{

                let total_forcex=a.force[0];
                let total_forcey=a.force[1];

                for i in it{
                    let forcex=total_forcex*(i.inner.mass/a.mass);
                    let forcey=total_forcey*(i.inner.mass/a.mass);
                    i.inner.apply_force([forcex,forcey]);
                }
            }
        }

        fn is_far_enough(&self,b:[F64n;2])->bool{
            (b[0].into_inner()-b[1].into_inner()).abs()>self.dis
        }

        fn is_far_enough_half(&self,b:[F64n;2])->bool{
            (b[0].into_inner()-b[1].into_inner()).abs()>self.dis_half
        }

    }

    impl nbody::NodeMassTraitConst for Bla{
        type T=BBox<F64n,Bot>;
        type No=NodeMass;

        fn get_rect(a:&Self::No)->&axgeom::Rect<F64n>{
            &a.rect
        }

        //gravitate this nodemass with another node mass
        fn handle_node_with_node(&self,a:&mut Self::No,b:&mut Self::No){
            
            let _ = duckduckgeo::gravitate(a,b,0.0001,0.004,|a|a.sqrt());
        }

        //gravitate a bot with a bot
        fn handle_bot_with_bot(&self,a:&mut Self::T,b:&mut Self::T){
            let _ = duckduckgeo::gravitate(&mut a.inner,&mut b.inner,0.0001,0.004,|a|a.sqrt());
        }

        //gravitate a nodemass with a bot
        fn handle_node_with_bot(&self,a:&mut Self::No,b:&mut Self::T){
            
            let _ = duckduckgeo::gravitate(a,&mut b.inner,0.0001,0.004,|a|a.sqrt());
        }


        fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a self,it:I,rect:axgeom::Rect<F64n>)->Self::No{
            let mut total_x=0.0;
            let mut total_y=0.0;
            let mut total_mass=0.0;

            for i in it{
                let m=i.inner.mass;
                total_mass+=m;
                total_x+=m*i.inner.pos[0];
                total_y+=m*i.inner.pos[1];
            }
            
            let center=if total_mass!=0.0{
                [total_x/total_mass,
                total_y/total_mass]
            }else{
                [0.0;2]
            };
            NodeMass{center,mass:total_mass,force:[0.0;2],rect}
        }

        fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a self,a:&'a Self::No,it:I){

            if a.mass>0.0000001{

                let total_forcex=a.force[0];
                let total_forcey=a.force[1];

                for i in it{
                    let forcex=total_forcex*(i.inner.mass/a.mass);
                    let forcey=total_forcey*(i.inner.mass/a.mass);
                    i.inner.apply_force([forcex,forcey]);
                }
            }
        }

        fn is_far_enough(&self,b:[F64n;2])->bool{
            (b[0].into_inner()-b[1].into_inner()).abs()>self.dis
        }

        fn is_far_enough_half(&self,b:[F64n;2])->bool{
            (b[0].into_inner()-b[1].into_inner()).abs()>self.dis_half
        }

    }
}

struct Res{
    calls_to_gravitate:usize,
    calls_to_gravitate_naive:usize,
    error:f64,
    error_avg:f64
}

struct BenchRes{
    bench_naive:f64,
    bench:f64,
    bench_par:f64
}
fn bench1(bots:&mut [Bot],diff:f64)->BenchRes{
     let (bots_copy,bench_naive)={
        let mut bots_copy=Vec::new();
        bots_copy.extend_from_slice(bots);

        let instant=Instant::now();
     
        let mut calls_to_gravitate=0;
        nbody::naive_mut(&mut bots_copy,|a,b|{
            calls_to_gravitate+=1;
            let _ = duckduckgeo::gravitate(a,b,0.0001,0.004,|a|a.sqrt());
        });

        let bench_naive=instant_to_sec(instant.elapsed());
        (bots_copy,bench_naive)
    };
    

    let bench={
        let instant=Instant::now();

        let mut tree=DinoTree::new_seq(axgeom::XAXISS,(),bots,|b|{
            b.create_aabb()
        });
        let _a=f64n!(0.0);
        let mut tree=tree.with_extra(go::NodeMass::new());

        
        let mut bla=go::Bla{calls_to_gravitate:0,dis:diff,dis_half:diff/2.0};


        let border = axgeom::Rect::new(f64n!(-9000.0),f64n!(9000.0),f64n!(-9000.0),f64n!(9000.0));  
        
        nbody::nbody(&mut tree,&mut bla,border);

        tree.apply(bots,|a,b|{
            b.force=a.inner.force;
        });


        instant_to_sec(instant.elapsed())
    };

    let bench_par={
        let instant=Instant::now();

        let mut tree=DinoTree::new(axgeom::XAXISS,(),bots,|b|{
            b.create_aabb()
        });
        let _a=f64n!(0.0);
        let mut tree=tree.with_extra(go::NodeMass::new());

        
        let mut bla=go::Bla{calls_to_gravitate:0,dis:diff,dis_half:diff/2.0};


        let border = axgeom::Rect::new(f64n!(-9000.0),f64n!(9000.0),f64n!(-9000.0),f64n!(9000.0));  
        
        nbody::nbody_par(&mut tree,&mut bla,border);

        tree.apply(bots,|a,b|{
            b.force=a.inner.force;
        });

        instant_to_sec(instant.elapsed())
        

    };

    //black_box(bots_copy);
    assert_eq!(bots_copy.len(),bots.len());

    BenchRes{bench_naive,bench,bench_par}
}
fn test1(bots:&mut [Bot],diff:f64)->Res{
    //let mut counter=datanum::Counter::new();

    let (bots_copy,calls_to_gravitate_naive)={
        let mut bots_copy=Vec::new();
        bots_copy.extend_from_slice(bots);

        let mut calls_to_gravitate=0;
        nbody::naive_mut(&mut bots_copy,|a,b|{
            calls_to_gravitate+=1;
            let _ = duckduckgeo::gravitate(a,b,0.0001,0.004,|a|a.sqrt());
        });

        (bots_copy,calls_to_gravitate)
    };

    let calls_to_gravitate={
        let mut tree=DinoTree::new_seq(axgeom::XAXISS,(),bots,|b|{
            b.create_aabb()
        });
        let _a=f64n!(0.0);
        let mut tree=tree.with_extra(go::NodeMass::new());

        
        let mut bla=go::Bla{calls_to_gravitate:0,dis:diff,dis_half:diff/2.0};


        let border = axgeom::Rect::new(f64n!(-9000.0),f64n!(9000.0),f64n!(-9000.0),f64n!(9000.0));  
        
        nbody::nbody(&mut tree,&mut bla,border);

        tree.apply(bots,|a,b|{
            b.force=a.inner.force;
        });

        (bla.calls_to_gravitate)
    };

    let error={
        let mut max_diff=None;
        for (a,b) in bots.iter().zip(bots_copy.iter()){
            assert_eq!(a.mass,b.mass);
            let dis_sqr1=a.force[0]*a.force[0]+a.force[1]*a.force[1];
            let dis_sqr2=b.force[0]*b.force[0]+b.force[1]*b.force[1];
            let dis1=dis_sqr1.sqrt();
            let dis2=dis_sqr2.sqrt();
            let acc_dis1=dis1;// /a.mass;
            let acc_dis2=dis2;// /a.mass;

            let diff=(acc_dis1-acc_dis2).abs();
            
            
            let error:f64=(acc_dis2-acc_dis1).abs()/acc_dis2;
            match max_diff{
                None=>{
                    max_diff=Some((diff,error))
                },
                Some(max)=>{
                    if diff>max.0{
                        max_diff=Some((diff,error))
                    }
                }
            }
        }
        match max_diff{
            Some(x)=>x.1,
            None=>0.0
        }
    };


    let error_avg={
        let mut error_total=0.0;
        for (a,b) in bots.iter().zip(bots_copy.iter()){
            assert_eq!(a.mass,b.mass);
            let dis_sqr1=a.force[0]*a.force[0]+a.force[1]*a.force[1];
            let dis_sqr2=b.force[0]*b.force[0]+b.force[1]*b.force[1];
            let dis1=dis_sqr1.sqrt();
            let dis2=dis_sqr2.sqrt();
            let acc_dis1=dis1;// /a.mass;
            let acc_dis2=dis2;// /a.mass;

            let _diff=(acc_dis1-acc_dis2).abs();
            
            
            let error:f64=(acc_dis2-acc_dis1).abs()/acc_dis2;
            error_total+=error;
        }
        error_total/bots.len() as f64
    };

    Res{calls_to_gravitate,calls_to_gravitate_naive,error,error_avg}
}



pub fn handle(fb:&mut FigureBuilder){
    handle_spiral1(fb);
    handle_spiral2(fb);
}


fn handle_spiral1(fb:&mut FigureBuilder){
    struct Record{
        _num_bots:usize,
        dis:f64,
        res:BenchRes
    }
    let mut rects=Vec::new();

    //for num_bots in (0..10000).step_by(500){
    let num_bots=10000;
    for dis in (0..420).step_by(10).map(|a|a as f64){
        let s=dists::spiral::Spiral::new([0.0,0.0],17.0,0.3);

        let mut bots=generate_bot_from_spiral(&s,num_bots);

        let res=bench1(&mut bots,dis);
        rects.push(Record{_num_bots:num_bots,dis,res});
    }
//}

    let x=rects.iter().map(|a|a.dis as f64);
    let y1=rects.iter().map(|a|a.res.bench_naive as f64);
    let y2=rects.iter().map(|a|a.res.bench as f64);
    let y3=rects.iter().map(|a|a.res.bench_par as f64);


    let mut fg=fb.new("nbody_bench");
    fg.axes2d()
        .set_title("Comparison of Benching Dinotree Nbody algorithms vs Naive with 10,000 objects with grow of 0.3", &[])
        .lines(x.clone(), y1,  &[Caption("Naive"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Nbody sequential"), Color("green"), LineWidth(2.0)])
        .lines(x.clone(), y3,  &[Caption("Nbody parallell"), Color("red"), LineWidth(2.0)])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);
    fb.finish(fg);

    /*
    let x=rects.iter().map(|a|a.num_bots as f64);
    let y=rects.iter().map(|a|a.dis);
    let z1=rects.iter().map(|a|a.res.bench_naive as f64);
    let z2=rects.iter().map(|a|a.res.bench as f64);
    let z3=rects.iter().map(|a|a.res.bench_par as f64);

    let mut fg = Figure::new();

    fg.axes3d()/*.set_view(110.0,30.0)*/
        .set_title("Maximum Error of Dinotree Nbody versus Naive", &[])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Node Max Distance ", &[])
        .set_z_label("Percentage Error", &[Rotate(90.0),TextOffset(-3.0,0.0)])
        .points(x.clone(), y.clone(), z1.clone(), &[Caption("Dinotree"),PointSymbol('O'), Color("red"), PointSize(0.3)])
        .points(x.clone(), y.clone(), z2.clone(), &[Caption("Dinotree"),PointSymbol('O'), Color("blue"), PointSize(0.3)])
        .points(x.clone(), y.clone(), z3.clone(), &[Caption("Dinotree"),PointSymbol('O'), Color("green"), PointSize(0.3)]);
                            

    fg.show(); 
    */
}
fn handle_spiral2(fb:&mut FigureBuilder){
    struct Record{
        _num_bots:usize,
        dis:f64,
        res:Res
    }
    let mut rects=Vec::new();

    //for num_bots in (0..10000).step_by(500){
    let num_bots=10000;
        for dis in (0..420).step_by(20).map(|a|a as f64){
            let s=dists::spiral::Spiral::new([0.0,0.0],17.0,0.2);

            let mut bots=generate_bot_from_spiral(&s,num_bots);

            let res=test1(&mut bots,dis);
            
            rects.push(Record{_num_bots:num_bots,dis,res});
        }
    //}

    let x=rects.iter().map(|a|a.dis);
    let y1=rects.iter().map(|a|a.res.calls_to_gravitate as f64);
    let y2=rects.iter().map(|a|a.res.calls_to_gravitate_naive as f64);
    let y3=rects.iter().map(|a|a.res.error);  
    let y4=rects.iter().map(|a|a.res.error_avg);  

    let mut fg=fb.new("nbody_theory");

    //let mut fg = Figure::new();

    //fg.set_terminal("pngcairo size 1024, 768","graphs/theory.png");

    fg.axes2d()
        .set_pos_grid(3, 1, 0)
        .set_title("Comparison of Benching Dinotree Nbody algorithms vs Naive with 10,000 objects with grow of 0.3", &[])
        .lines(x.clone(), y1,  &[Caption("Naive"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Nbody sequential"), Color("green"), LineWidth(2.0)])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);


    fg.axes2d()
        .set_pos_grid(3,1,1)
        .set_title("Maximum Error of Dinotree Nbody versus Naive", &[])
        .lines(x.clone(), y3,  &[Caption("Naive"), Color("blue"), LineWidth(2.0)])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);


    fg.axes2d()
        .set_pos_grid(3,1,2)
        .set_title("Average Error of Dinotree Nbody versus Naive", &[])
        .lines(x.clone(), y4,  &[Caption("Nbody sequential"), Color("green"), LineWidth(2.0)])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);
    fb.finish(fg);
    /*
    let x=rects.iter().map(|a|a.num_bots as f64);
    let y=rects.iter().map(|a|a.dis);
    let z1=rects.iter().map(|a|a.res.calls_to_gravitate as f64);
    let z2=rects.iter().map(|a|a.res.calls_to_gravitate_naive as f64);
    let z3=rects.iter().map(|a|a.res.error);  
    let z4=rects.iter().map(|a|a.res.error_avg);  

    let mut fg = Figure::new();

    fg.axes3d().set_view(110.0,30.0)
        .set_title("Number of gravitation calls on two bodies", &[])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Node Max Distance", &[])
        .set_z_label("Number of gravitation calls on two bodies", &[Rotate(90.0),TextOffset(-3.0,0.0)])
       // .points(x.clone(), y.clone(), z2.clone(), &[Caption("Dinotree"),PointSymbol('O'), Color("red"), PointSize(1.0)])
        .points(x.clone(), y.clone(), z1.clone(), &[Caption("Dinotree"),PointSymbol('O'), Color("violet"), PointSize(1.0)]);
   

    fg.show();   

    let mut fg = Figure::new();

    fg.axes3d().set_view(110.0,30.0)
        .set_title("Maximum Error of Dinotree Nbody versus Naive", &[])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Node Max Distance ", &[])
        .set_z_label("Percentage Error", &[Rotate(90.0),TextOffset(-3.0,0.0)])
        .points(x.clone(), y.clone(), z3.clone(), &[Caption("Dinotree"),PointSymbol('O'), Color("red"), PointSize(1.0)]);
                

    fg.show(); 


    let mut fg = Figure::new();

    fg.axes3d().set_view(110.0,30.0)
        .set_title("Average Error of Dinotree Nbody versus Naive", &[])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Node Max Distance", &[])
        .set_z_label("Percentage Error", &[Rotate(90.0),TextOffset(-3.0,0.0)])
        .points(x.clone(), y.clone(), z4.clone(), &[Caption("Dinotree"),PointSymbol('O'), Color("red"), PointSize(1.0)]);
                

    fg.show();  
    */ 
}

/*
#[derive(Debug)]
struct Record {
    num_bots: usize,
    grow: f64,
    num_pairs:usize,
    z1: usize,
    z2: usize,
    z3: Option<usize>
}

pub struct DataColFind3d{
    num_bots:usize,
}


impl DataColFind3d{
    pub fn new(_dim:[f64;2])->DataColFind3d{    
        DataColFind3d{num_bots:0}
    }
}



fn handle_spiral(){
    let mut rects=Vec::new();

    for num_bots in (0..10000).step_by(1000){
        for grow in (0..100).map(|a|0.0005+(a as f64)*0.0001){//0.001 to 0.002
            let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);

            let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
                let pos=[pos[0] as isize,pos[1] as isize];
                Bot{num:0,pos}
            }).collect();

            let z1=test1(&mut bots);
            let z2=test2(&mut bots);
            let z3=if num_bots<8000{
                Some(test3(&mut bots))
            }else{
                None
            };

            let num_pairs={
                assert_eq!(z1.num_pairs,z2.num_pairs);
                if let Some(z3)=&z3{
                    assert_eq!(z2.num_pairs,z3.num_pairs);    
                }
                z1.num_pairs
            };
            
            
            let z1=z1.num_comparison;
            let z2=z2.num_comparison;
            let z3=z3.map(|a|a.num_comparison);
            let r=Record{num_bots,grow,num_pairs,z1,z2,z3};
            rects.push(r);      
            
        }
    }
    draw_rects(&mut rects);       
}
fn handle_spiral_two(){
    let mut rects=Vec::new();

    for num_bots in (0..10000).step_by(1000){
        for grow in (0..100).map(|a|0.2+(a as f64)*0.1){
            let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);

            
            let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
                let pos=[pos[0] as isize,pos[1] as isize];
                Bot{num:0,pos}
            }).collect();

            let z1=test1(&mut bots);
            let z2=test2(&mut bots);
            let z3=if num_bots<3000{
                Some(test3(&mut bots))
            }else{
                None
            };

            let num_pairs={
                assert_eq!(z1.num_pairs,z2.num_pairs);
                if let Some(z3)=&z3{
                    assert_eq!(z2.num_pairs,z3.num_pairs);    
                }
                z1.num_pairs
            };
            
            
            let z1=z1.num_comparison;
            let z2=z2.num_comparison;
            let z3=z3.map(|a|a.num_comparison);
            let r=Record{num_bots,grow,num_pairs,z1,z2,z3};
            rects.push(r);      
            
        }
    }
    draw_rects(&mut rects);    
}



fn draw_rects(rects:&mut [Record]){
    {
        let x=rects.iter().map(|a|a.num_bots as f64);
        let y=rects.iter().map(|a|a.grow);
        let z1=rects.iter().map(|a|a.z1 as f64);
        let z2=rects.iter().map(|a|a.z2 as f64);

        
        let (x2,y2,z3)={

            let ii=rects.iter().filter(|a|a.z3.is_some());
            let x=ii.clone().map(|a|a.num_bots as f64);
            let y=ii.clone().map(|a|a.grow as f64);
            let z3=ii.clone().map(|a|a.z3.unwrap());

            (x,y,z3)
        };
        

        let mut fg = Figure::new();

        fg.axes3d().set_view(110.0,30.0)
            .set_title("Comparison of Sweep and Prune versus Dinotree", &[])
            .set_x_label("Number of Objects", &[])
            .set_y_label("Spareness of Objects", &[])
            .set_z_label("Number of Comparisons", &[Rotate(90.0),TextOffset(-3.0,0.0)])
            .points(x.clone(), y.clone(), z1.clone(), &[Caption("Dinotree"),PointSymbol('O'), Color("violet"), PointSize(1.0)])
            .points(x.clone(), y.clone(), z2.clone(), &[Caption("Sweep and Prune"),PointSymbol('o'), Color("red"), PointSize(1.0)])
            .points(x2.clone(), y2.clone(), z3.clone(), &[Caption("Naive"),PointSymbol('o'), Color("green"), PointSize(0.5)]);


        fg.show();
    }

    {
        let mut fg = Figure::new();

        let x=rects.iter().map(|a|a.num_bots);
        let y=rects.iter().map(|a|a.grow);
        let z=rects.iter().map(|a|a.num_pairs as f64);


        fg.axes3d().set_view(110.0,30.0)
            .set_title("Number of Pair Intersections for Spiral Distribution", &[])
            .set_x_label("Number of Objects", &[])
            .set_y_label("Spareness of Objects", &[])
            .set_z_label("Number of Intersections", &[Rotate(90.0),TextOffset(-3.0,0.0)])
            .points(x, y, z, &[PointSymbol('O'), Color("violet"), PointSize(1.0)]);
            
        fg.show();
    }
}

pub fn handle(){
    handle_spiral();
    handle_spiral_two();    
}

*/