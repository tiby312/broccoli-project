

# Comparison of Tree Height

The below charts show the performance of the tree when manually selecting a height other than the default one chosen.
You can see that the theory is a downward curve, but the benching is more of a bowl. Theory would tell us to have a big enough height such that every leaf node had only one bot in it. But in the real world, this has a lot of overhead with recursive calls and memory. Instead the benching suggested a smaller height where the leaf nodes has a few bots in them.

<img alt="Height heuristic" src="graphs/height_heuristic.svg" class="center" style="width: 100%;" />

The below chart compares the empirically best height against the height that our heuristic tree height function produces. 

<img alt="Height Heuristic vs Optimal" src="graphs/height_heuristic_vs_optimal.svg" class="center" style="width: 100%;" />



## Only pick ODD height trees.

In order to assure that the leaf nodes all map to a relatively square piece of space, we force the tree to always have odd height. Picking an even height would be that one axis would be divided one ore time than the other axis.
