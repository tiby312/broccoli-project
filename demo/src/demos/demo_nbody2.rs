use crate::support::prelude::*;

#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    force: Vec2<f32>,
    mass: f32,
}

#[derive(Copy, Clone)]
struct NodeMass {
    center: Vec2<f32>,
    mass: f32,
    force: Vec2<f32>,
}

use core::default::Default;
impl Default for NodeMass{
    fn default()->NodeMass{
        NodeMass{
            center:vec2(Default::default(),Default::default()),
            mass:Default::default(),
            force:vec2(Default::default(),Default::default())
        }
    }
}


use core::marker::PhantomData;
use broccoli::pmut::*;
use broccoli::query::nbody2::*;


#[derive(Clone, Copy)]
struct Bla<'a> {
    _num_pairs_checked: usize,
    _p: PhantomData<&'a usize>,
}
impl<'b> broccoli::query::nbody2::NNN for &Bla<'b> {
    type Mass = NodeMass;
    type T = BBox<f32, &'b mut Bot>;
    type N = f32;


    //return the position of the center of mass
    fn compute_center_of_mass(&mut self,a:&[Self::T])->Self::Mass{
        unimplemented!();
    }

    fn are_close(&mut self,a:&Self::Mass,b:&Self::Mass)->bool{
        unimplemented!();
    }


    fn gravitate(&mut self,a:GravEnum<Self::T,Self::Mass>,b:GravEnum<Self::T,Self::Mass>){
        unimplemented!();
    }

    fn gravitate_self(&mut self,a:PMut<[Self::T]>){
        unimplemented!();
    }

    fn apply_a_mass(&mut self,mass:Self::Mass,b:PMut<[Self::T]>){
        unimplemented!();
    }

    fn combine_two_masses(&mut self,a:&Self::Mass,b:&Self::Mass)->Self::Mass{
        unimplemented!();
    }
}



