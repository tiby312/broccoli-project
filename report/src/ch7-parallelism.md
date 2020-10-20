
 
# Comparison of Parallel Height

The below chart shows the performance of the broccoli tree for different levels at which to switch to sequential.
Obviously if you choose to switch to sequential straight away, you have sequential tree performance.

This was benched on a laptop with 4 physical cores. This means that if you just parallelize one level of the kdtree, you're only taking advantage of two of the 4 cores. This explains the time it took when we switched at level 8 vs 9. 

<img alt="Parallel Height Heuristic" src="graphs/parallel_height_heuristic.svg" class="center" style="width: 100%;" />


## Multithreading

Evenly dividing up work into chuncks for up to N cores is the name of the game here. You don't want to make assumptions about how many cores the user has. Why set up your system to only take up advantage of 2 cores if 16 are available. So here, using a thread pool library like rayon is useful. 

In big complicated system, it can be tempting to keep sub-systems sequential and then just allow multiple sub-systems to happen in parallel if they are computation heavy. But this is not fully taking advantage of multithreading. You want each subsystem to itself be parallizable. So this collision detection system is parallizable. 

