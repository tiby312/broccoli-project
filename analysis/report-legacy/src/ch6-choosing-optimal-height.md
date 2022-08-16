

### Comparison of Tree Height

The below charts show the performance of building and querying colliding pairs when manually selecting a height other than the default one chosen.
You can see that the theory is a downward curve, but the benching is more of a bowl. Theory would tell us to have a big enough height such that every leaf node had only one bot in it. But in the real world, this is overhead due to excessive recursive calls. Its not that pronounced, and I think it is because most of the aabbs don't make it to the bottom of the tree anyway. Most will intersect a divider somewhere in the tree. If we used smaller aabbs it might be more pronounced.

<link rel="stylesheet" href="css/poloto.css">

{{#include raw/height_heuristic_theory.svg}}
{{#include raw/height_heuristic_bench.svg}}



### ODD vs Even height trees.

You can see that the even heights are barely better than the odds for sub optimal heights. With odd trees, the direction that the root nodes aabbs are sorted is the same as the leaves. If its even they are different. When the direction's match, we can use sweep and prune to speed things up. When the directions don't match, the sorted property can't be exploited since they are in different dimensions even though some pruning can still be done
based off of the bounding rectangles of the dividers. In 'normal' looking trees where the aabbs arn't extremely clumped up, these two methods seem to be around the same speed. In degenerate cases, not enough aabbs can be excluded using the dividers bounding box for the perpendicular cases. 

The below chart compares the empirically best height against the height that our heuristic tree height function produces. 


<style>
.test .poloto0stroke{
    stroke-width:20;
}
</style>
<div class="test">
{{#include raw/height_heuristic_vs_optimal.svg}}
</div>

