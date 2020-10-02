use crate::inner_prelude::*;
use crate::Aabb;

///The results of the binning process.
pub struct Binned<'a, T: 'a> {
    pub middle: &'a mut [T],
    pub left: &'a mut [T],
    pub right: &'a mut [T],
}

unsafe fn swap_unchecked<T>(arr: &mut [T], a: usize, b: usize) {
    let a = &mut *(arr.get_unchecked_mut(a) as *mut T);
    let b = &mut *(arr.get_unchecked_mut(b) as *mut T);
    core::mem::swap(a, b)
}

/// Sorts the bots into three bins. Those to the left of the divider, those that intersect with the divider, and those to the right.
/// They will be laid out in memory s.t.  middile < left < right
pub fn bin_middle_left_right<'b, A: Axis, X: Aabb>(
    axis: A,
    med: &X::Num,
    bots: &'b mut [X],
) -> Binned<'b, X> {
    let bot_len = bots.len();

    let mut left_end = 0;
    let mut middle_end = 0;

    //     |    middile   |   left|              right              |---------|
    //
    //                ^           ^                                  ^
    //              middile_end    left_end                      index_at

    for index_at in 0..bot_len {
        match bots[index_at].get().get_range(axis).contains_ext(*med) {
            //If the divider is less than the bot
            core::cmp::Ordering::Equal => {
                //left

                bots.swap(index_at, left_end);
                bots.swap(left_end, middle_end);
                middle_end += 1;
                left_end += 1;
            }
            //If the divider is greater than the bot
            core::cmp::Ordering::Greater => {
                //middile
                bots.swap(index_at, left_end);
                left_end += 1;
            }
            core::cmp::Ordering::Less => {
                //right
            }
        }
    }

    let (rest, right) = bots.split_at_mut(left_end);
    let (middle, left) = rest.split_at_mut(middle_end);
    //println!("middile left right={:?}",(middle.len(),left.len(),right.len()));
    debug_assert!(left.len() + right.len() + middle.len() == bot_len);
    Binned {
        left,
        middle,
        right,
    }
}

/// Sorts the bots into three bins. Those to the left of the divider, those that intersect with the divider, and those to the right.
/// They will be laid out in memory s.t.  middile < left < right
pub(crate) unsafe fn bin_middle_left_right_unchecked<'b, A: Axis, X: Aabb>(
    axis: A,
    med: &X::Num,
    bots: &'b mut [X],
) -> Binned<'b, X> {
    let bot_len = bots.len();

    let mut left_end = 0;
    let mut middle_end = 0;

    //     |    middile   |   left|              right              |---------|
    //
    //                ^           ^                                  ^
    //              middile_end    left_end                      index_at

    for index_at in 0..bot_len {
        match bots
            .get_unchecked(index_at)
            .get()
            .get_range(axis)
            .contains_ext(*med)
        {
            //If the divider is less than the bot
            core::cmp::Ordering::Equal => {
                //left

                swap_unchecked(bots, index_at, left_end);
                swap_unchecked(bots, left_end, middle_end);
                middle_end += 1;
                left_end += 1;
            }
            //If the divider is greater than the bot
            core::cmp::Ordering::Greater => {
                //middile
                swap_unchecked(bots, index_at, left_end);
                left_end += 1;
            }
            core::cmp::Ordering::Less => {
                //right
            }
        }
    }

    let (rest, right) = bots.split_at_mut(left_end);
    let (middle, left) = rest.split_at_mut(middle_end);
    //println!("middile left right={:?}",(middle.len(),left.len(),right.len()));
    debug_assert!(left.len() + right.len() + middle.len() == bot_len);
    Binned {
        left,
        middle,
        right,
    }
}

#[inline(always)]
pub fn compare_bots<T: Aabb>(axis: impl Axis, a: &T, b: &T) -> core::cmp::Ordering {
    let (p1, p2) = (a.get().get_range(axis).start, b.get().get_range(axis).start);
    if p1 > p2 {
        core::cmp::Ordering::Greater
    } else {
        core::cmp::Ordering::Less
    }
}

///Sorts the bots based on an axis.
#[inline(always)]
pub fn sweeper_update<I: Aabb, A: Axis>(axis: A, collision_botids: &mut [I]) {
    let sclosure = |a: &I, b: &I| -> core::cmp::Ordering { compare_bots(axis, a, b) };

    collision_botids.sort_unstable_by(sclosure);
}

/*
#[cfg(test)]
mod test{
    use test_support;
    use test_support::Bot;
    use test_support::create_unordered;
    use super::*;
    use axgeom;
    //use Blee;
    use support::BBox;
    use *;
    use ordered_float::NotNaN;
    #[test]
    fn test_get_section(){
        for _ in 0..100{
            let world=test_support::create_word();
            let axis=axgeom::XAXIS;
            let rr=Range{start:100.0,end:110.0};


              let mut vec1:Vec<BBox<NotNaN<f32>,Bot>>=(0..1000).map(|a|
            {
                let rect=test_support::get_random_rect(&world);
                let bot=Bot::new(a);
                BBox::new(bot,rect)
            }
                ).collect();

            //let mut vec1:Vec<Bot>=(0..500).map(|a|Bot{id:a,rect:support::get_random_rect(&world)}).collect();



            let src:Vec<usize>={
                let mut src_temp=Vec::new();

                for a in vec1.iter(){

                    if rr.intersects(a.rect.get_range(axis)){
                        src_temp.push(a.val.id);
                    }

                }
                src_temp
            };


            let mut sw=Sweeper::new();
            let a=Blee::new(axis);
            Sweeper::update(&mut vec1,&a);

            /*
            println!("Bots:");
            for b in vec1.iter(){
                println!("{:?}",(b.id,b.rect.get_range(axis)));
            }
            */
let target=sw.get_section(&mut vec1,&rr,&a);

match target{
Some(x)=>{

//Assert that all bots that intersect the rect are somewhere in the list outputted by get_setion().
for k in src.iter(){
let mut found=false;
for j in x.iter(){
if *k==j.val.id{
found=true;
break;
}
}
assert!(found);
}

//Assert that the first bot in the outputted list intersects with get_section().
let first=x.first().unwrap();
let mut found=false;
for j in src.iter(){
if first.val.id==*j{
found=true;
break;
}
}
assert!(found);

//Assert that the last bot in the outputted list intersects with get_section().
let last=&x[x.len()-1];
let mut found=false;
for j in src.iter(){
if last.val.id==*j{
found=true;
break;
}
}
assert!(found);
},
None=>{
assert!(src.len()==0);
}
}

}
}

#[test]
fn test_bijective_parallel(){
for _ in 0..100{
let world=test_support::create_word();
//let mut vec1:Vec<BBox<Bot>>=(0..5).map(|a|Bot{id:a,rect:support::get_random_rect(&world)}).collect();
//let mut vec2:Vec<BBox<Bot>>=(0..5).map(|a|Bot{id:vec1.len()+a,rect:support::get_random_rect(&world)}).collect();

let mut vec1:Vec<BBox<NotNaN<f32>,Bot>>=(0..5).map(|a|
{
let rect=test_support::get_random_rect(&world);
let bot=Bot::new(a);
BBox::new(bot,rect)
}
).collect();

let mut vec2:Vec<BBox<NotNaN<f32>,Bot>>=(0..5).map(|a|
{
let rect=test_support::get_random_rect(&world);
let bot=Bot::new(vec1.len()+a);
BBox::new(bot,rect)
}
).collect();

let axis=axgeom::XAXIS;

let mut src:Vec<(usize,usize)>={
let mut src_temp=Vec::new();

for i in vec1.iter(){
for j in vec2.iter(){
let (a,b):(&BBox<NotNaN<f32>,Bot>,&BBox<NotNaN<f32>,NotNaN<f32>,Bot>)=(i,j);

if a.rect.get_range(axis).intersects(b.rect.get_range(axis)){
src_temp.push(create_unordered(&a.val,&b.val));
}
}
}
src_temp
};

let mut sw=Sweeper::new();
let a=Blee::new(axis);
Sweeper::update(&mut vec1,&a);
Sweeper::update(&mut vec2,&a);

let mut val=Vec::new();
//let rr=world.get_range(axis);

{
let mut f=|cc:ColPair<BBox<NotNaN<f32>,Bot>>|{
val.push(create_unordered(cc.a.1,cc.b.1));
};
let mut bk=BleekSF::new(&mut f);
sw.find_bijective_parallel((&mut vec1,&mut vec2),&a,&mut bk);
}
src.sort_by(&test_support::compair_bot_pair);
val.sort_by(&test_support::compair_bot_pair);

/*
println!("naive result:\n{:?}",(src.len(),&src));
println!("sweep result:\n{:?}",(val.len(),&val));

println!("Bots:");
for b in vec1{
    println!("{:?}",(b.id,b.rect.get_range(axis)));
}
println!();

for b in vec2{
    println!("{:?}",(b.id,b.rect.get_range(axis)));
}
*/
assert!(src==val);
}
}

#[test]
fn test_find(){

//let world=axgeom::Rect::new(-1000.0,1000.0,-1000.0,1000.0);
let world=test_support::create_word();

let mut vec:Vec<BBox<NotNaN<f32>,Bot>>=(0..500).map(|a|
{
let rect=test_support::get_random_rect(&world);
let bot=Bot::new(a);
BBox::new(bot,rect)
}
).collect();

//Lets always order the ids smaller to larger to make it easier to look up.
// let mut map:HashMap<(usize,usize),()>=HashMap::new();
let mut src:Vec<(usize,usize)>=Vec::new();

let axis=axgeom::XAXIS;
for (e,i) in vec.iter().enumerate(){
for j in vec[e+1..].iter(){
let (a,b):(&BBox<NotNaN<f32>,Bot>,&BBox<NotNaN<f32>,Bot>)=(i,j);

if a.rect.get_range(axis).intersects(b.rect.get_range(axis)){
src.push(create_unordered(&a.val,&b.val));
}
}
}

let mut sw=Sweeper::new();

let a=Blee::new(axis);
Sweeper::update(&mut vec,&a);

let mut val=Vec::new();

{
let mut f=|cc:ColPair<BBox<NotNaN<f32>,Bot>>|{
val.push(create_unordered(cc.a.1,cc.b.1));
};
let mut bk=BleekSF::new(&mut f);
sw.find(&mut vec,&a,&mut bk);
}
src.sort_by(&test_support::compair_bot_pair);
val.sort_by(&test_support::compair_bot_pair);

//println!("{:?}",(src.len(),val.len()));
//println!("{:?}",val);
assert!(src==val);
}
}

*/
