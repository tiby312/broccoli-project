
### Semi-Direct vs Direct vs Indirect

Below are a bunch of diagrams that highlight differences between a couple variables:
Whether the elements inserted into the tree are made up of:

* `(Rect<Num>,&mut T)` (Semi-Direct)
* `(Rect<Num>,T)` (Direct)
* `&mut (Rect<Num>,T)` (Indirect)

We also vary the size of `T` (8,16,32,128,or 256 bytes).
We do not bother varying the size of `Num` since we assume the user is using a
'normal' sized number type like a float or an integer.

We bench construction as well as one call to `find_colliding_pairs`.

We define a more specialized `abspiral()`, `abspiral-isize()` that takes an additional
argument which influences the size of `T`.

There are a couple of observations to make.
* Semi-Direct is the best all-around.
* Direct is sometimes slightly faster then Semi-Direct at querying, but the slowest at construction
* Indirect isn't far behind Semi-Direct, but suffers in some high density distributions.
* Direct is greatly influenced by the size of `T`.
 
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_query_0.1_128_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_query_1_128_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_query_0.1_32_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_query_0.1_8_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_query_0.1_16_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_query_0.1_256_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_query_0.01_128_bytes.svg" class="center" style="width: 100%;" />


<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_rebal_0.1_256_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_rebal_1_128_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_rebal_0.1_128_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_rebal_0.1_8_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_rebal_0.1_16_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_rebal_0.01_128_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/tree_direct_indirect_rebal_0.1_32_bytes.svg" class="center" style="width: 100%;" />



### Different Data Layouts

Because the tree construction code is generic over the elements that are inserted (as long as they implement Aabb),
The user can easily try all three data layouts.

The semi-direct layout is almost always the fastest. 
There are a few corner cases where if T is very small, and the aabbs are very dense, direct is slightly faster, but it is marginal.

The semi-direct layout is good because during broccoli construction and querying we make heavy use of aabb's, but don't actually need T all that much. We only need T when we actually detect a collision, which doesn't happen that often. Most of the time we are just
ruling out possible colliding pairs by checking their aabbs.

Yes the times we do need T could lead to a cache miss, but this happens infrequently enough that that is ok.
Using the direct layout we are less likely to get a cache miss on access of T, but now the aabbs are further apart from each other
because we have all these T's in the way. It also took us longer to put the aabb's and T's all together in one contiguous memory.

One thing that is interesting to note is that in these benches T has its aabb already inside of it, and so `(Rect<isize>,&mut T)` duplicates that memory. This is still faster than direct and indirect, but it does use more memory. The direct method's main advantage is memory usage which is the lowest.

If we were inserting references into the tree, then the original order of the aabbs is preserved during construction/destruction of the tree. However, for the direct layout, we are inserting the actual aabbs to remove this layer of indirection. So when are done using the tree, we may want to return the aabbs to the user is the same order that they were put in. This way the user can rely on indices for other algorithms to uniquely identify a bot. To do this, during tree construction, you would have to also build up a Vec of offsets to be used to return the aabbs to their original position. You would have to keep this as a separate data structure as it will only be used on destruction of the tree. If we were to put the offset data into the tree itself, it would be wasted space and would hurt the memory locality of the tree query algorithms. We only need to use these offsets once, during destruction. It shouldnt be the case that all querying algorithms that might be performed on the tree suffer performance for this.


### AABB vs Point + radius

Point+radius pros:
less memory (just 3 floating point values)
cons:
can only represent a circle (not an oval)
have to do more floating point calculations during querying

AABB pros:
no floating point calculations needed during querying.
can represent any rectangle
cons:
more memory (4 floating point values)

Note, if the size of the elements is the same then the Point+radius only takes up 2 floating point values, so that might be better in certain cases. But even in this case, I think the cost of having to do floating point calculations when comparing every bot with every other bot in the query part of the algorithm is too much. With a AABB system, absolutely no floating point calculations need to be done to find all colliding pairs.

### AABB data layout

The aabb we use is made up of ranges that look like : start,end instead of start,length.  If you define them as a start and a length then querying intersections between rectangles requires that you do floating point arithmetic. The advantage of the start,end data layout is that all the broccoli query algorithms don't need to do a single floating point calculation. It is all
just comparisons. The downside, is that if you want the dimensions on the aabb, you have to calculate them, but this isnt something that any of the tree algorithms need. 
