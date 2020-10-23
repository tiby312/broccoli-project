

### Comparison of Tree Height

The below charts show the performance of the tree when manually selecting a height other than the default one chosen.
You can see that the theory is a downward curve, but the benching is more of a bowl. Theory would tell us to have a big enough height such that every leaf node had only one bot in it. But in the real world, this has a lot of overhead with recursive calls and memory. Instead the benching suggested a smaller height where the leaf nodes has a few bots in them.

<img alt="Height heuristic" src="graphs/height_heuristic.svg" class="center" style="width: 100%;" />

### Only pick ODD height trees.

In the above graph, the even heights are always slower than the odds. This didnt make sense to me since if you have an even height, then each leaf is going to be more square than rectangle. However, you can't fight the data. I think it is faster with odds because the sweep and prune algorithm can take advantage of the more rectangle shaped leafs instead of the square ones.
This might be different if the leafs were not sorted which I mention in the improvements section.




The below chart compares the empirically best height against the height that our heuristic tree height function produces. 

<img alt="Height Heuristic vs Optimal" src="graphs/height_heuristic_vs_optimal.svg" class="center" style="width: 100%;" />

