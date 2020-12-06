### Test Setup

Before we can measure and compare performance of this algorithm, we have to come up with a good way to test it. We often want to see how performance degrades as the size of the problem increases, but we also do not want to influence other variables.

For our tests lets use an archimedean spiral distribution. It gives us a lot of variation in how the aabbs intersects, and allows us to grow the size of the problem without affecting density too much. 

If all the aabbs were dstributed along only one dimension then that would also skew our results. For example, sweep and prune will perform very well if all the aabbs are spaced out along the axis we are sweeping.

Lets make a archemdian spiral function that takes 3 inputs and produces an archimedean spiral.: 
* n: the number of aabbs
* separation: the seperation between the aabbs as they are laid out along the spiral.
* grow rate: the rate at which the spiral grow outward from the center.

We increase n to increase the size of the problem.
We can increase the grow rate to decrease the number of aabbs intersecting.

<img alt="Spiral Visualize" src="graphs/spiral_visualize.svg" class="center" style="width: 100%;" />


While those 3 variables change the the distribution of the elements, there is another variable at play.

* aabb size (the bounding box size of each element)

For all the benching in here, we just fix the size such that every element has the same size aabb. There is still a lot of interesting data analysis to do with elements that are all different sizes, but for now we'll just analyse cases where they are all the same.

Lets define a particular scene/distribution just so that it makes are benching simpler.

Let `abspiral(n,grow)` be a distribution of aabbs where:
* n=number of aabbs
* grow=spiral grow rate
* separation=constant (17)
* aabb radius=constant (5)

The constants are just arbitrary values. We just want all the elements to have some bounding box size and to influence how many of them are intersecting. This just makes things simpler since for most of the benches, we can typically show trends what we want to show by only influencing these two variables, so we might as well pick constants for the other variables and imbue that in the meaning of abspiral() itself.

The below chart shows how influencing the spiral_grow affects the number of bot intersections for abspiral(). This shows that we can influence the spiral grow to see how the performance of the tree degrades. We could influence how many aabbs are colliding with changing the separation, but the relationship to the grow rate and the number of intersection pairs makes a nice smooth downward graph.

It is not entirely smooth, but it is smooth enough that we can use this function to change the load on the broccoli without having to sample multiple times.

Its also clearly not linear, but all that really matters is that we have a way to increase/decrease
the number of collisions easily. We just need something that will allow us to definitively see
trends in how the algorithms stack up against each other.

<img alt="Spiral Data" src="graphs/spiral_data.svg" class="center" style="width: 100%;" />

