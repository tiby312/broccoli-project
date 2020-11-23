use broccoli::{bbox, prelude::*, rect};

fn main() {
    let mut inner1 = 4;
    let mut inner2 = 5;
    let mut inner3 = 6;

    let mut bots = [
        bbox(rect(2isize,3, 2, 3), &mut inner1),
        bbox(rect(7, 8, 7, 8), &mut inner2),
        bbox(rect(4, 7, 4, 7), &mut inner3),
    ];

    let mut tree = broccoli::collections::TreeRef::new(&mut bots);

    let mut m = tree.multi_rect();

    let mut zone1=Vec::new();
    m.for_all_in_rect_mut(rect(0,5,0,5),|a|{
        zone1.push(a);
    }).unwrap();

    let mut zone2=Vec::new();
    m.for_all_in_rect_mut(rect(5,10,5,10),|a|{
        zone2.push(a);
    }).unwrap();

    
    assert_eq!(zone1.len(), 1);
    assert_eq!(zone2.len(), 1);
    assert_eq!(**zone1[0].inner_mut(),4);
    assert_eq!(**zone2[0].inner_mut(),5);
}
