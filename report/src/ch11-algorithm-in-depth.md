
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

#### Profiling Construction + Finding all colliding pairs.

Here are some profiling results finding all intersections on `abspiral(0.2,50_000)` 30 times. 
The image below is a SVG image and is interactive. Hover over blocks to see the full names.
Try this link for a full page view of the image: [SVG](graphs/flamegraph.svg)

<object class="p" data="graphs/flamegraph.svg" type="image/svg+xml" style="width: 100%;">

As you can see all functions are split up into smaller problems.




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


