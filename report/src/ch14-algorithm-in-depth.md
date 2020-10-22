
### In depth algorithm overview:


#### Construction

Construction works as follows, Given: a list of elements.

For every node we do the following:
1. First we find the median of the remaining elements (using pattern defeating quick select) and use its position as this nodes divider.
2. Then we bin the bots into three bins. Those strictly to the left of the divider, those strictly to the right, and those that intersect.
3. Then we sort the bots that intersect the divider along the opposite axis that was used to finding the median. These bots now live in this node.
4. Now this node is fully set up. Recurse left and right with the bots that were binned left and right. This can be done in parallel.


#### Finding all intersecting pairs

Done via divide and conquer. For every node we do the following:
1) First we find all intersections with bots in that node using sweep and prune..
2) We recurse left and right finding all bots that intersect with bots in the node.
	Here we can quickly rule out entire nodes and their decendants if a node's aabb does not intersect
	with this nodes aabb.
3) At this point the bots in this node have been completely handled. We can safely move on to the children nodes 
   and treat them as two entirely seperate trees. Since these are two completely disjoint trees, they can be handling in
   parallel.


#### Nbody (experimental)

Here we use divide and conquer.

The nbody algorithm works in three steps. First a new version tree is built with extra data for each node. Then the tree is traversed taking advantage of this data. Then the tree is traversed again applying the changes made to the extra data from the construction in the first step.

The extra data that is stored in the tree is the sum of the masses of all the bots in that node and all the bots under it. The idea is that if two nodes are sufficiently far away from one another, they can be treated as a single body of mass.

So once the extra data is setup, for every node we do the following:
	Gravitate all the bots with each other that belong to this node.
	Recurse left and right gravitating all bots encountered with all the bots in this node.
		Here once we reach nodes that are sufficiently far away, we instead gravitate the node's extra data with this node's extra data, and at this point we can stop recursing.
	At this point it might appear we are done handling this node the problem has been reduced to two smaller ones, but we are not done yet. We additionally have to gravitate all the bots on the left of this node with all the bots on the right of this node.
    For all nodes encountered while recursing the left side,
    	Recurse the right side, and handle all bots with all the bots on the left node.
    	If a node is suffeciently far away, treat it as a node mass instead and we can stop recursing.
    At this point we can safely exclude this node and handle the children and completely independent problems.



## Raycasting


TODO explain

## Knearest

TODO explain


## Rect

TODO explain.


