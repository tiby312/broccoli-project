
### Algorithm overview:


#### Construction

Construction works like this. Given: a list of aabbs.

For every node we do the following:
1. First we find the median of the remaining elements (using pattern defeating quick select) and use its position as this nodes divider.
2. Then we bin the aabbs into three bins. Those strictly to the left of the divider, those strictly to the right, and those that intersect.
3. Then we sort the aabbs that intersect the divider along the opposite axis that was used to finding the median. These aabbs now live in this node.
4. Now this node is fully set up. Recurse left and right with the aabbs that were binned left and right. This can be done in parallel.


#### Finding all colliding pairs

Done via divide and conquer. For every node we do the following:
1) First we find all intersections with aabbs in that node using sweep and prune..
2) We recurse left and right finding all aabbs that intersect with aabbs in the node.
	Here we can quickly rule out entire nodes and their descendants if a node's aabb does not intersect
	with this nodes aabb.
3) At this point the aabbs in this node have been completely handled. We can safely move on to the children nodes 
   and treat them as two entirely separate trees. Since these are two completely disjoint trees, they can be handling in
   parallel.


#### Allocations

There are some very fast allocators out
there, but not all allocators are created equal. If you want your code to be as platform independent as possible,
you should try to minimize allocations even if in benches on your local machine, there is no performance hit. For example, currently the rust webassembly target using a very simple allocator that is pretty slow. The colliding pair
finding algorithm requires a stack at each level of recursion. Each level of recursion, we could allocate a new stack,
but reusing the preallocated stack is better. It just turns out that this requires some unsafe{} since we are populating the stack with lifetimed mutable references. 



#### How to handle parallel cases

Part of the colliding pair finding algorithm requires that we find all colliding pairs between two nodes. 
Some times the aabbs between two nodes are sorted along the same dimension and sometimes not. When they are
we have three options that all sound pretty good:

* Option 1:
	* Use just one stack of active list
	* Aabbs are kept in the active list longer than normal
	* More comparisons, but simple and only uses one vec
* Option 2:
	* Use two active lists. 
	* Fewest comparisons
	* Needs two vecs.
* Option 3:
	* Use two active lists, but implement it as one vec under the hood.
	* Fewest allocations
	* Fewest comparisons
	* Slow to push and truncate each vec since it requires shuffling things around.


I went with option 3. The performance hit from pushing and truncating can be made up with a big allocation up front.
Doing two big allocations upfront for option2 is wasteful.


#### How to speed up perpendicular cases

Its slow to naively find intersections between the aabbs in two nodes that are sorted along different dimensions.
There are a couple of options:

* Option 1:
	* Cut off each list by using the other node's bounding box to deal with smaller lists.
	* Now iterate over each element, and perform parallel sweep where one list has size one.

* Option 2:
	* Cut off each list by using the other node's bounding box to deal with smaller list.
	* Collect a list of pointers of one list. 
	* Sort that list along the other lists axis
	* Perform parallel sweep 

* Option 3:
	* Cut off each list as always
	* do a simple nested loop and check if the aabbs intersect

* Option 4:
	* Cut off each list as always
	* For each element in node A, iterate over each element in node B.
		* Exit early if the B element is completely to the right of the A element.
		* Now we only have to check if the B element's right side is touching A.
		
Option 4 is the fastest. It exploits the sorted property of the aabbs, but also does not require
any kind of storage of an active list.

#### Profiling Construction + Finding all colliding pairs.

Here are some profiling results finding all intersections on `abspiral(0.2,50_000)` 30 times. 
The image below is a SVG image and is interactive. Hover over blocks to see the full names.
Try this link for a full page view of the image (or if it is not rendering correctly): [SVG](graphs/flamegraph.svg)


<object>
<style>
  .ayu{
	  --pplot_color0:yellow;
  }
  .rust{
	   --pplot_color0:green;
  }
  .coal{
	  --pplot_color0:brown
  }
  .light{
	  --pplot_color0:black;
  }
  .navy{
	  --plot_color0:blue;
  }
  #splot{
   	--fg_color:var(--fg);
	--bg_color:var(--bg);
	--plot_color0:var(--pplot_color0);
	--plot_color1:blue;
	--plot_color2:green;
	--plot_color4:red;
	--plot_color5:red;
	--plot_color6:red;
  }
</style>
{{#include graphs/image.svg}}
</object>

<object class="p" data="graphs/flamegraph.svg" type="image/svg+xml" style="width: 100%;">
</object>

The flame graph shows a very insightful map of how much time is spent in which sections of the algorithm.
You can clearly see how the rebalancing and the querying are each individually broken down.
You can see how at each recursive step, a piece of the problem is broken off.



#### nbody (experimental)

Here we use divide and conquer.

The nbody algorithm works in three steps. First a new version tree is built with extra data for each node. Then the tree is traversed taking advantage of this data. Then the tree is traversed again applying the changes made to the extra data from the construction in the first step.

The extra data that is stored in the tree is the sum of the masses of all the aabbs in that node and all the aabbs under it. The idea is that if two nodes are sufficiently far away from one another, they can be treated as a single body of mass.

So once the extra data is setup, for every node we do the following:
	Gravitate all the aabbs with each other that belong to this node.
	Recurse left and right gravitating all aabbs encountered with all the aabbs in this node.
		Here once we reach nodes that are sufficiently far away, we instead gravitate the node's extra data with this node's extra data, and at this point we can stop recursing.
	At this point it might appear we are done handling this node and the problem has been reduced to two smaller ones, but we are not done yet. We additionally have to gravitate all the aabbs on the left of this node with all the aabbs on the right of this node.
    For all nodes encountered while recursing the left side,
    	Recurse the right side, and handle all aabbs with all the aabbs on the left node.
    	If a node is sufficiently far away, treat it as a node mass instead and we can stop recursing.
    At this point we can safely exclude this node and handle the children and completely independent problems.



#### Raycasting


TODO explain now

TODO improvement:
right now the raycast algorithm naively checks all the elements that belong to a node provided
it decides to look at a node. In reality, we could do better. We could figure out where the ray
hits the divider line, and then only check AABBs that intersect that divider line. The problem
with this improvement is that we can't just rely on the `Num` trait since we need to do some math.
You lose the nice property of the algorithm not doing any number arithmetic. Therefore I didn't implement
it. However it might be a good idea. That said, before any element is checked using the expensive raycast function, it will first check the abb
raycast function to determine if it is even worth going further. This is probably a good enough speed up.

#### Knearest

TODO explain


#### Rect

TODO explain.


