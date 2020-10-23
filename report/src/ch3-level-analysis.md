
### Level Comparison

The below charts show the load balance between the different levels of the tree.

Some observations:
* The cost of rebalancing the first level is the most erratic. 
	I like to think of the algorithm as a sponge and the problem as water seeping through it.
	First you you have coarse filtering, then it gets more precise.
* The load goes from the top levels to the bottom levels as the bots spread out more.
* The load on the first few levels is not high unless the bots are clumped up. 
* The leaves don't have much work to do since aabbs have a size, they aren't likely to 
  into a leaf.

<img alt="Level Analysis" src="graphs/level_analysis_theory_rebal.svg" class="center" style="width: 100%;" />
<img alt="Level Analysis" src="graphs/level_analysis_theory_query.svg" class="center" style="width: 100%;" />
<img alt="Level Analysis" src="graphs/level_analysis_bench_rebal.svg" class="center" style="width: 100%;" />
<img alt="Level Analysis" src="graphs/level_analysis_bench_query.svg" class="center" style="width: 100%;" />
