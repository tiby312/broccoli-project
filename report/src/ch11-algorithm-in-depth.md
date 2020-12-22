
### Algorithm overview:


#### Construction

Construction works as follows, Given: a list of elements.

For every node we do the following:
1. First we find the median of the remaining elements (using pattern defeating quick select) and use its position as this nodes divider.
2. Then we bin the aabbs into three bins. Those strictly to the left of the divider, those strictly to the right, and those that intersect.
3. Then we sort the aabbs that intersect the divider along the opposite axis that was used to finding the median. These aabbs now live in this node.
4. Now this node is fully set up. Recurse left and right with the aabbs that were binned left and right. This can be done in parallel.


#### Finding all colliding pairs

Done via divide and conquer. For every node we do the following:
1) First we find all intersections with aabbs in that node using sweep and prune..
2) We recurse left and right finding all aabbs that intersect with aabbs in the node.
	Here we can quickly rule out entire nodes and their decendants if a node's aabb does not intersect
	with this nodes aabb.
3) At this point the aabbs in this node have been completely handled. We can safely move on to the children nodes 
   and treat them as two entirely seperate trees. Since these are two completely disjoint trees, they can be handling in
   parallel.


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

Turns out these both appear to be about the same, if we adjust the target number of aabbs for node. Option2 prefers like 64 aabbs, while option2 prefers a smaller amount like 32. Because of this I chose option2 since it does
not require any special dynamic allocation.


#### Profiling Construction + Finding all colliding pairs.

Here are some profiling results finding all intersections on `abspiral(0.2,50_000)` 30 times. 
The image below is a SVG image and is interactive. Hover over blocks to see the full names.
Try this link for a full page view of the image (or if it is not rendering correctly): [SVG](graphs/flamegraph.svg)

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
    	If a node is suffeciently far away, treat it as a node mass instead and we can stop recursing.
    At this point we can safely exclude this node and handle the children and completely independent problems.



#### Raycasting


TODO explain now

TODO improvement:
right now the raycast algorithm naively checks all the elements that belong to a node provided
it decides to look at a node. In reality, we could do better. We could figure out where the ray
hits the divider line, and then only check AABBs that intersect that divider line. The problem
with this improvement is that we can't just rely on the `Num` trait since we need to do some math.
You lose the nice property of the algorithm not doing any number arithmatic. Therefore I didnt implement
it. However it might be a good idea. That said, before any element is checked using the expensive raycast function, it will first check the abb
raycast function to determine if it is even worth going further. This is probably a good enough speed up.

#### Knearest

TODO explain


#### Rect

TODO explain.


