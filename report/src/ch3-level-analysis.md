
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




<link rel="stylesheet" href="css/poloto.css">

{{#include raw/level_analysis_theory_rebal.svg}}
{{#include raw/level_analysis_theory_query.svg}}
{{#include raw/level_analysis_bench_rebal.svg}}
{{#include raw/level_analysis_bench_query.svg}}



### Evenness of load


{{#include raw/query_evenness_theory_default.svg}}
{{#include raw/query_evenness_theory_dense.svg}}
{{#include raw/query_evenness_theory_sparse.svg}}


The above charts show that the work load is pretty even between the left and right recursing of the algorithm.
As aabbs get more clumped up, the right side starts to dominate more. I'm not sure why this is, but I do know that picking the median based off of the right and bottom of the aabbs instead of left and top makes no difference in this case. I think it is just that this particular spiral distribution becomes not so uniform when the aabbs are extremely clumped up.



{{#include raw/query_num_per_node_theory_default.svg}}
{{#include raw/query_num_per_node_theory_dense.svg}}
{{#include raw/query_num_per_node_theory_sparse.svg}}

The above charts show that the number of elements per node becomes very saturated / evenly distributed when elements are not as clumped up.




