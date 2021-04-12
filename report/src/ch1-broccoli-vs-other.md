
### Trends as N increases


The broccoli crate's goal is to provide broad-phase queries such as "find all elements that intersect". Its data structure is basically a kdtree with the added feature that elements that belong to a node are sorted along the divider axis so that we can use sweep and prune during the query phase. Lets compare the complexity of finding all intersecting pairs using four different methods: kdtree, sweep and prune, naive, and broccoli.

<link rel="stylesheet" href="css/poloto.css">

{{#include raw/colfind_theory_default.svg}}

As you can see, broccoli is a clear winner in terms of minimizing the number of comparisons. The jumps that you see in the kdtree line are the points at which the trees height grows. It is a complete binary tree so a slight increase in the height causes a doubling of nodes so it is a drastic change. As the number of aabbs increases it is inevitable that sometimes the tree will be too tall or too short. Like kdtree, broccoli is also a complete binary tree so it also have jumps, but they are less pronounced. I think this is because the second strategy of sweep and prune can "absorb" the jumps.

Lets make sure that broccoli is still the winner if the aabbs are more clumped up.

{{#include raw/colfind_theory_dense.svg}}

Okay good broccoli is still the winner against its building blocks in terms of comparisons.

So thats great that we've found a strategy that minimizes the comparisons, but that doesn't really mean anything unless the real world performance is also just as fast. Lets look at how it benches against the same set of strategies.

{{#include raw/colfind_bench_default.svg}}

Here we have also plotted the parallel versions that uses the  `rayon` work-stealing crate under the hood.
This is benched on my quad-core laptop. It is interesting to note that the real world bench times follow the same trend as the theoretical number of comparisons. Again, we can see that brococli wins. Parallel broccoli beats parallel kdtree, and ditto for sequential versions. Lets make sure things don't change when elements are more clumped up.

{{#include raw/colfind_bench_dense.svg}}

Okay good its still the same results. In fact you can see that the parallel kdtree algorithm is suffering a bit with the extra clumpiness. Earlier it was on par with sequential broccoli, but now it is definitively worse.

It's worth noting that the difference between `sweep and prune`/`kdtree` and `naive` is much bigger than the difference between `sweep and prune`/`kdtree` and `broccoli`. So using these simpler algorithms gets you big gains as it is. The gains you get from using `broccoli` are not as pronounced, but are noticeable with more elements.

In the same vein, you can see that there aren't many gains to use `broccoli_par` over `broccoli`. It can double/quadruple your performance, but as you can see those gains pale in comparison to the gains from simply using the a better sequential algorithm. Thats not to say multiplying your performance by the number of cores you have isn't great, it's just that it isn't as big a factor. This to me means that typically, allocating effort on
investigating if your algorithm is optimal sequentially may be better than spending effort in parallelizing what you have.

### Trends as Grow increases

Up until now, we have been looking at trends of how the algorithms preform as we increase the number of aabbs that it has to find intersections for. Lets instead look at trends of what happens when we change how clumped the aabbs are instead.

{{#include raw/colfind_theory_grow_wide.svg}}

Okay broccoli is still the best. We didn't include naive because it dwarfed the other algorithms so that you couldn't see the differences between the non naive algorithms. Lets look at bench times:


{{#include raw/colfind_bench_grow_wide.svg}}

Same trends. Again naive was excluded since it just dwarfed everything.



### Extremely clumped up case

So far we've looked at "reasonable" levels of clumpiness. When things get extremely clumped up, weird things happen.

{{#include raw/colfind_theory_grow.svg}}

Okay so there are some weird things going on here. For one thing naive isnt a straight line. For another broccoli and sweep cross over each other.

The first weird thing is explained by the fact that the naive implementation I used is not 100% naive. While it does check
every possible pair, it first checks if a pair of aabb's collides in one dimension. If it does not collide in that dimension, it does not even check the next dimension. So because of this "short circuiting", there is a slight increase in comparisons when the aabbs are clumped up. If there were no short-circuiting, it would be flat all across. It is clear from the graph that this short-circuiting optimization does not gain you all that much.

The second weird thing of `sweep and prune` seemingly having a better worst case than the `broccoli`: This makes sense since in the worst case, `sweep and prune` will sort all the elements, and then sweep. In the worst case for `broccoli`, it will first find the median, and then sort all the elements, and then sweep. So `broccoli` is slower since it redundantly found the median, and then sorted everything. However, it is easy to see that this only happens when the aabbs are extremely clumped up. So while `sweep and prune` has a better worst-cast, the worst-cast scenario of `broccoli` is rare and it is not much worse (median finding + sort versus just sort). 

Now lets look at the benches.

{{#include raw/colfind_bench_grow.svg}}

Above we benched for a smaller number of elements since it simply takes too long at this density to bench 30_000 elements like we did in the non-extremely-clumped-up case earlier. This graph looks extremely weird!

I really can't fathom why its faster to find collisions when everything is touching everything when everything is almost touching everything. My only guess is that in the former case, branch prediction is very straight forward. You just assume you're also going down the path of a collision hit. So its very weird to say, but the actual worst case when it comes to real-world performance is the almost-worst theoretical case.

### Fairness

It's important to note that these comparisons aren't really fair. With broccoli, we are focused on optimising the finding colliding pairs portion, but these comparisons are comparing construct + one call to finding colliding pairs. However, we can't really show a graph of just the query portion, because other algorithms can't be easily split up into a construction and query part. Perhaps a better test would be to compare multiple query calls. So for each algorithm with one set of elements, find all the colliding pairs, then also find all the elements in a rectangle, then also find knearest, etc.


### Sweep vs Kd-Tree vs Both

Sweep and prune is a nice and simple AABB collision finding system, but it degenerates as there are more and more "false-positives" (objects that intersect on one axis, but not both). Kd Trees have a great recursive way of pruning out elements, but non-point objects that can't be inserted into children are left in the parent node and those objects must be collision checked with everybody else naively. A better solution is to use both. 

The basic idea is that you use a tree up until a specific tree height, and then switch to sweep and prune, and then additionally use sweep and prune for elements stuck in higher up tree nodes. The sweep and prune algorithm is a good candidate to use since it uses very little memory (just a stack that can be reused as you handle descendant nodes). But the real reason why it is good is the fact that the aabbs that belong to a non-leaf node in a kd tree are likely to be strewn across the divider in a line. Sweep and prune degenerates when the active list that it must maintain has many aabbs that end up not intersecting. This isn't likely to happen for the aabbs that belong to a node since the aabbs that belong to a node are guaranteed to touch the divider. If the divider partitions aabbs based off their x value, then the aabbs that belong to that node will all have x values that are roughly close together (they must intersect divider), but they y values can be vastly different (all the aabbs will be scattered up and down the dividing line). So when we do sweep and prune, it is important that we sweep and prune along axis that is different from the axis along which the divider is partitioning, otherwise it will degenerate to practically the naive algorithm.


### KD tree vs Quad Tree

I think the main benefit of a quad tree is that tree construction is fast since we don't need to find the median at each level. They also have a interesting relationship with [z order curves](https://en.wikipedia.org/wiki/Z-order_curve).

But that comes at a cost of a potentially not great partitioning of the physical elements. Our goal is to make the querying as fast as possible as this is the part that can vary and dominate very easily in dense/clumped up situations. The slow construction time of the kdtree is not ideal, but it is a very consistent load (doesn't vary from how clumped the elements are). 

KD trees are also great in a multi-threaded setting. With a kd tree, you are guaranteed that for any parent, there are an equal number of objects if you recurse the left side and the right side since you specifically chose the divider to be the median. 

This means that during the query phase, the work-load will be fairly equal on both sides. It might not be truly equal because even though for a given node, you can expect both the left and right sub-trees to have an equal number of elements, they most likely will be distributed differently within each sub-tree. For example the left sub-tree might have all of its elements stuck in just one node, while the right sub-tree has all of its elements in its leaf nodes. However, the size of each sub-tree is a somewhat good estimate of the size of the problem of querying it. So this is a good property for a divide and conquer multi-threaded style algorithm. With a quad tree, the load is not as likely to be even between the four children nodes. 


### Tree space partitioning vs grid 

I wanted to make a collision system that could be used in the general case and did not need to be fine-tuned. Grid based collision systems suffer from the teapot-in-a-stadium problem. They also degenerate more rapidly as objects get more clumped up. If, however, you have a system where you have strict rules with how evenly distributed objects will be among the entire space you're checking collisions against, then I think a grid system can be better. But I think these systems are few and far in between. I think in most systems, for example, its perfectly possible for all the objects to exist entirely on one half of the space being collision checked leaving the other half empty. In such a case, half of the data structure of the grid system is not being used to partition anything. There are also difficulties in how to represent the data structure since every grid cell could have a variable number of aabbs in side of it. Having a Vec in each cell, for example, would hardly be efficient.

The way [liquid fun](https://google.github.io/liquidfun/) does collisions by using grids in one dimension and sweep and prune in the other. 

### broccoli vs BVT

I'm not sure how broccoli stacks up against a bounding volume tree. This is something I would like to investigate in the future. It would be interesting to compare against bottom up and top down constructions of BVT seeing as KD Tree are top down by nature.