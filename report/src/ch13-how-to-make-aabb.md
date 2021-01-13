#### How to make a Aabb

So far we've talked about how aabbs can be stored in a Tree. For example we recommended
populating it with `BBox<N,&mut T>`.
But we havent talked about how you would generate such a struct.

In some dynamic systems, every particle has a position and maybe a radius, and then from the position and radius
an aabb can be generated. So what you store in your tree might look something like this:

`BBox<f32,&mut Particle>`

where `Particle` might look like this:
```
struct Particle{
    pos:[f32;2],
    vel:[f32;2]
}

```

and then in your main loop you might have something like this:
```
    tree.for_every_colliding_pair(|a,b|{
        a.repel(b)
    })
```

#### An optimization idea

Provided all the particles are the same size, in order to save on space, your particle could just be :
```
struct Particle{
    vel:[f32;2]
}
```

And you could instead use one of the corners of the aabb:

```
    tree.for_every_colliding_pair_pmut(|a,b|{
        let apos=[a.get().x.start,a.get().y.start];
        let bpos=[b.get().x.start,b.get().y.start];
        repel(apos,a.inner_mut(),bpos,b.inner_mut())
    })
```

This works because for the repel() function we just need the relative offset position
to determine the direction and magnitude of the repel force. So it doesnt matter that
we used the top left corner instead of the center.

This optimization might still work if your aabb was converted from floats to u32 provided 
that you convert them back right before repelling. However if the aabbs are u16 converted from float,
then the offset vectors might be too inaccurate.


