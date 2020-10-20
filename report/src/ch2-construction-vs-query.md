
# Rebalancing vs Querying

The below charts show the load balance between the construction and querying on the broccoli.
It's important to note that the comparison isnt really 'fair'. The cost of querying depends a lot on
what you plan on doing with every colliding pair (it could be an expensive user calculation). Here we just use a 'reasonably' expensive calculation that repels the colliding pairs.

Some observations:
* The cost of rebalancing does not change with the density of the objects
* The cost of querying does change with the density.
* If the bots are spread out enough, the cost of querying decreases enough to cost less than the cost of rebalancing.
* The cost of querying is reduced more by parallelizing than the cost of rebalancing.
	
It makes sense that querying in more 'parallelilable' than rebalancing since the calculation that you have to perform for each node before you can divide and conquer the problem is more expensive for rebalancing. For rebalancing you need to find the median and bin the bots. For querying you only have to do sweep and prune. 

<img alt="Construction vs Query" src="graphs/construction_vs_query_grow_theory.svg" class="center" style="width: 100%;" />
<img alt="Construction vs Query" src="graphs/construction_vs_query_grow_bench.svg" class="center" style="width: 100%;" />
<img alt="Construction vs Query" src="graphs/construction_vs_query_num_theory.svg" class="center" style="width: 100%;" />
<img alt="Construction vs Query" src="graphs/construction_vs_query_num_bench.svg" class="center" style="width: 100%;" />


## Construction Cost vs Querying Cost


If you are simulating moving elements, it might seem slow to rebuild the tree every iteration. But from benching, most of the time querying is the cause of the slowdown. Rebuilding is always a constant load, but the load of the query can vary wildly depending on how many elements are overlapping.

For example, in a bench where inside of the collision call-back function I do a reasonable collision response with 80_000 bots, if there are 0.8 times (or 65_000 ) collisions or more, querying takes longer than rebuilding. For your system, it might be impossible for there to even be 0.8 * n collisions, in which case building the tree will always be the slower part. For many systems, 0.8 * n collisions can happen. For example if you were to simulate a 2d ball-pit, every ball could be touching 6 other balls [Circle Packing](https://en.wikipedia.org/wiki/Circle_packing). So in that system, there are around 3 * n collisions. So in that case, querying is the bottle neck. With liquid or soft-body physics, the number can be every higher. up to n * n.

Rebuilding the first level of the tree does take some time, but it is still just a fraction of the entire building algorithm in some crucial cases, provided that it was able to partition almost all the bots into two planes. 

Additionally, we have been assuming that once we build the tree, we are just finding all the colliding pairs of the elements. In reality, there might be many different queries we want to do on the same tree. So this is another reason we want the tree to be built to make querying as fast as possible, because we don't know how many queries the user might want to do on it. In addition to finding all colliding pairs, its quite reasonable the user might want to do some k_nearest querying, some rectangle area querying, or some raycasting.
