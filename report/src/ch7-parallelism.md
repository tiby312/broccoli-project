### Comparison of Parallel Height

The below chart shows the performance of the broccoli tree for different levels at which to switch to sequential.
Obviously if you choose to switch to sequential straight away, you have sequential tree performance.

This was benched on a laptop with 4 physical cores. This means that if you just parallelize one level of the kdtree, you're only taking advantage of two of the 4 cores. This explains the time it took when we switched at level 8 vs 9. 

<img alt="Parallel Height Heuristic" src="graphs/parallel_height_heuristic.svg" class="center" style="width: 100%;" />


