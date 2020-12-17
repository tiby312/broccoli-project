
### Level Comparison

The below charts show the load balance between the different levels of the tree. 
Tree construction is compared against one call to `find_colliding_pairs`.

Some observations:
* The cost of rebalancing the first level is the most erratic. 
    This is because in some cases we're hitting the worst cases of pdqselect.
	I like to think of the algorithm as a sponge and the problem as water seeping through it.
	First you you have coarse filtering, then it gets more precise.
* The load goes from the top levels to the bottom levels as the aabbs spread out more.
* The load on the first few levels is not high unless the aabbs are clumped up. 
* The leaves don't have much work to do since aabbs have a size, they aren't likely to 
  into a leaf.

<img alt="Level Analysis" src="graphs/level_analysis_theory_rebal.svg" class="center" style="width: 100%;" />
<img alt="Level Analysis" src="graphs/level_analysis_theory_query.svg" class="center" style="width: 100%;" />
<img alt="Level Analysis" src="graphs/level_analysis_bench_rebal.svg" class="center" style="width: 100%;" />
<img alt="Level Analysis" src="graphs/level_analysis_bench_query.svg" class="center" style="width: 100%;" />



### Evenness of load

<img alt="Level Analysis" src="graphs/query_evenness_theory.svg" class="center" style="width: 100%;" />

The above chart shows that the work load is pretty even between the left and right recursing of the algorithm.
As aabbs get more clumped up, the right side starts to dominate more. This is not because when we pick a median,
we are using the left most value of the aabb as our median. If you look at the graph below, you can see that each side of the tree still has an roughly even number of aabbs with a grow of 0.007. I think it is something to do with the fact that all the aabbs are sorted using either their left or top sides. To make it more even, bots could be sorted away from the divider to make things more symetric. I mention this in the improvements section. Regardless this imbalance only happens for extremely clumped up sets.

<img alt="Level Analysis" src="graphs/tree_num_per_node_theory.svg" class="center" style="width: 100%;" />

