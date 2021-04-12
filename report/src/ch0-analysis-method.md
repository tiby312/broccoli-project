### Test Setup

Before we can measure and compare performance of broccoli to other broad-phase strategies, we have to come up with a good way to test it. We often want to see how performance degrades as the size of the problem increases, but we also do not want to influence other variables.

For our tests lets use the same distribution that sunflowers use. This distribution gives us a good consistent packing of aabbs, and also gives us a easy way to grow the size of the problem. It also gives us a lot of variation in how the aabbs intersects. (If all the aabbs were distributed along only one dimension then that would also skew our results. For example, sweep and prune will perform very well if all the aabbs are spaced out along the axis we are sweeping.)

Lets use [Vogel's model](https://en.wikipedia.org/wiki/Fermat%27s_spiral#The_golden_ratio_and_the_golden_angle) that takes 2 inputs and produces an sunflower spiral.: 
* n: the number of aabbs
* grow rate: the rate at which the spiral grow outward from the center.

We increase n to increase the size of the problem.
We can increase the grow rate to decrease the number of aabbs intersecting.

<link rel="stylesheet" href="css/poloto.css">

{{#include raw/spiral_visualize.svg}}


While those 2 variables change the distribution of the elements, there is another variable at play.

* aabb size (the bounding box size of each element)

For all the benching in here, we just fix the size such that every element has the same size aabb. There is still a lot of interesting data analysis to do with elements that are all different sizes, but for now we'll just analyse cases where they are all the same.

Lets define a particular scene / distribution just so that it makes are benching simpler.

Let `abspiral(n,grow)` be a distribution of aabbs where:
* n=number of aabbs
* grow=spiral grow rate
* aabb radius= `2`

The `2` constant is just an arbitrary value. We just want all the elements to have some bounding box size and to influence how many of them are intersecting. This just makes things simpler since for most of the benches, we can show trends by only influencing `n` and `grow`, so we might as well pick constants for the other variables and imbue that in the meaning of `abspiral()` itself.

The below chart shows how influencing the spiral_grow affects the number of bot intersections for abspiral(). This shows that we can influence the spiral grow to see how the performance of the tree degrades. It is not entirely smooth, but it is smooth enough that we can use this function to change the load on the broccoli without having to sample multiple times.

Its also clearly not linear, but all that really matters is that we have a way to increase/decrease
the number of collisions easily. We just need something that will allow us to definitively see
trends in how the algorithms stack up against each other.



{{#include raw/spiral_data_grow.svg}}

Throughout this writeup, we use a grow of `1.5` a lot as a "typical" amount of colliding pairs.
So for `20,000` aabbs, a grow rate of `1.5` gives you around `3 * 20,000` intersections. This is no where
near the worst case of `20,000 * 20,000` aabbs, which is `400000000`. But this worst case, doesn't really 
happen in a lot of use cases.

So a `abspiral(n,grow=1.5)` produces around `3*n` collisions. If you were to simulate a 2d ball-pit, every ball could be touching 6 other balls ([Circle Packing](https://en.wikipedia.org/wiki/Circle_packing)). So in that system, there are `(6 * n)/2 = 3*n` collisions. That said with liquid or soft-body physics, the number can be much higher.


Here are some inputs into our abspiral function and a ballparck of how many intersections occur:

```
abspiral( 20_000, 2.1 ) ~= 20_000           // SPARSE
abspiral( 20_000, 1.5 ) ~= 3 * 20_000       // DEFAULT
abspiral( 20_000, 0.6 ) ~= 20 * 20_000      // DENSE
abspiral( 20_000, 0.2 ) ~= 180 * 20_000     // MEGA DENSE
```


The below graph shows that as we increase the number of elements, so does the number of collisions in a nice
linear way.



<link rel="stylesheet" href="css/poloto.css">


{{#include raw/spiral_data_num.svg}}

