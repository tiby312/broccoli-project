
### Rebalancing vs Querying

The below charts show the load balance between the construction and querying through calling
`find_colliding_pairs` on the broccoli.
It's important to note that the comparison isnt really 'fair'. The cost of querying depends a lot on
what you plan on doing with every colliding pair (it could be an expensive user calculation). Here we just use a 'reasonably' expensive calculation that repels the colliding pairs.
```
//Called on each colliding pair.
fn repel(p1: Vec2<f32>, p2: Vec2<f32>, res1: &mut Vec2<f32>, res2: &mut Vec2<f32>) {
    let offset = p2 - p1;
    let dis = (offset).magnitude2();
    if dis < RADIUS * RADIUS {
        *res1 += offset * 0.0001;
        *res2 -= offset * 0.0001;
    }
}
````

Some observations:
* The cost of rebalancing does not change with the density of the objects
* If the aabbs are spread out enough, the cost of querying decreases enough to be about the same as rebalancing.
* The cost of querying is reduced more by parallelizing than the cost of rebalancing.
	
It makes sense that querying in more 'parallelizable' than rebalancing since the calculation that you have to perform for each node before you can divide and conquer the problem is more expensive for rebalancing. For rebalancing you need to find the median and bin the aabbs. For querying you only have to do sweep and prune. 

<img alt="Construction vs Query" src="graphs/construction_vs_query_grow_theory.svg" class="center" style="width: 100%;" />
<img alt="Construction vs Query" src="graphs/construction_vs_query_grow_bench.svg" class="center" style="width: 100%;" />
<img alt="Construction vs Query" src="graphs/construction_vs_query_num_theory.svg" class="center" style="width: 100%;" />
<img alt="Construction vs Query" src="graphs/construction_vs_query_num_bench.svg" class="center" style="width: 100%;" />



### Collect Performance

Sometimes you need to iterate over all colliding pairs multiple times even though the elements havent moved.
You could call `find_colliding_pairs()` multiple times, but it is slow.
Broccoli provides functions to save off query results so that they can be iterated on though `TreeInd`.

<img alt="Construction vs Query" src="graphs/broccoli_query.svg" class="center" style="width: 100%;" />

The above graph shows the performance of `collect_colliding_pairs()` and `collect_colliding_pairs_par()`. These functions generate lists of colliding pairs. The graph shows only the time taken to construct the lists.

<img alt="Construction vs Query" src="graphs/optimal_query.svg" class="center" style="width: 100%;" />

The above graph shows the performance of iterating over the pairs collected from calling `collect_colliding_pairs()` and `collect_colliding_pairs_par()`. The parallel version returns multiple disjoint pairs that can be iterated on in parallel. Notice that it is much faster to iterate over the pre-found colliding pairs when compared to the earlier chart. The graph makes it obvious that there are gains to iterating over disjoint pairs in parallel, but keep in mind that the times we are looking at are extremely small to begin with.

So as you can see, you pay a small cost, for collecting the colliding pairs, but then you are able to iterate 
over them over and over again faster.



### Construction Cost vs Querying Cost

If you are simulating moving elements, it might seem slow to rebuild the tree every iteration. But from benching, most of the time querying is the cause of the slowdown. Rebuilding is always a constant load, but the load of the query can vary wildly depending on how many elements are overlapping.


We don't want to speed up construction but produce a worse partitioning as a result because that would slow down querying and that is what can dominate easily. Instead we can focus on speeding up the construction of an optimal partitioning. This can be done via parallelism. Sorting the aabbs that sit on dividing lines may seem slow, but we can get this for 'free' essentially because it can be done after we have already split up the children nodes. So we can finish sorting a node while the children are being worked on. Rebuilding the first level of the tree does take some time, but it is still just a fraction of the entire building algorithm in some crucial cases, provided that it was able to partition almost all the aabbs into two planes. 

Additionally, we have been assuming that once we build the tree, we are just finding all the colliding pairs of the elements. In reality, there might be many different queries we want to do on the same tree. So this is another reason we want the tree to be built to make querying as fast as possible, because we don't know how many queries the user might want to do on it. In addition to finding all colliding pairs, its quite reasonable the user might want to do some k_nearest querying, some rectangle area querying, or some raycasting.

### Inserting elements after construction

broccoli does not support inserting elements after construction. If you want to add more elements,
you have to rebuild the tree. However, broccoli provides a `intersect_with()` function that lets you
find collisions between two groups. This way you can have one group of static objects and another group of dynamic objects, and update the static objects less frequently. In a dynamic
particle system for example, most of the time, enough particles move about in one step to justify
recreating the whole tree. 

### Exploiting Temporal Locality (with loose bounding boxes)

One strategy to exploit temporal locality is by inserting looser bounding boxes into the tree and caching the results of a query for longer than one step. The upside to this is that you only have to build and query the tree every couple of iterations. There are a number of downsides, though:

* Your system performance now depends on the speed of the aabbs. The faster your aabbs move, the bigger their loose bounding boxes, the slower the querying becomes. This isnt a big deal considering the ammount that a bot moves between two frames is expected to be extremely small. But still, there are some corner cases where performance would deteriorate. For example, if every bot was going so fast it would just from one end of you screen to the other between world steps. So you may also need to bound the velocity of your aabbs to a small value.

* You have to implement all the useful geometry tree functions all over again, or you can only use the useful geometry functions at the key world steps where the tree actually is constructed. For example, if you want to query a rectangle area, the tree provides a nice function to do this, but you only have the tree every couple of iterations. The result is that you have to somehow implement a way to query all the aabbs in the rectangle area using your cached lists of colliding aabbs, or simply only query on the world steps in which you do have the built tree. Those queries will also be slower since you are working on a tree with loose boxes.

* The maximum load on a world step is greater. Sure amortised, this caching system may save computation, but the times you do construct and query the tree, you are doing so with loose bounding boxes. On top of that, while querying, you also have to build up a seperate data structure that caches the colliding pairs you find. 

* The api of the broccoli is flexible enough that you can implement loose bounding box + caching on top of it (without sacrificing parallelism) if desired.

So in short, this system doesnt take advantage of temporal locality, but the user can still take advantage of it by inserting loose bounding boxes and then querying less frequently to amortize the cost. 

