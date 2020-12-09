

### Comparison of Tree Height

The below charts show the performance of building and querying colliding pairs when manually selecting a height other than the default one chosen.
You can see that the theory is a downward curve, but the benching is more of a bowl. Theory would tell us to have a big enough height such that every leaf node had only one bot in it. But in the real world, this is overhead due to excessive recursive calls. Its not that pronounced, and I think it is because most of the aabbs don't make it to the bottom of the tree anyway. Most will intersect a divider somewhere in the tree. If we used smaller aabbs it might be more pronounced.

<img alt="Height heuristic" src="graphs/height_heuristic.svg" class="center" style="width: 100%;" />

### ODD vs Even height trees.

In the above graph, the even heights are barely better than the odds. This is because with odd trees, the direction that the root nodes aabbs are sorted is the same as the leaves. If its even the are different. When the direction's match, we can use sweep and prune to speed things up. When the directions don't match, its hopefuly. The sorted property can't be exploited since they are in different dimensions.


The below chart compares the empirically best height against the height that our heuristic tree height function produces. 

<img alt="Height Heuristic vs Optimal" src="graphs/height_heuristic_vs_optimal.svg" class="center" style="width: 100%;" />

