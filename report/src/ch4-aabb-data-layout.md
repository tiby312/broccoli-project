
# Default vs Direct vs Indirect

Below are a bunch of diagrams that highlight differences between a couple variable:
Whether the elements inserted into the tree are made up of:

* `(Rect<Num>,&mut T)` (Default)
* `(Rect<Num>,T)` (Direct)
* `&mut (Rect<Num>,T)` (Indirect)

We also vary the size of `T` (8,32,128,or 256 bytes).
We do not bother varying the size of `Num` since we assume the user is using a
'normal' sized number type like a float or an integer.

We define a more specialized abspiral(), abspiral-isize() that takes an additonal
argument which influnces the size of `T`.

There are a couple of observations to make.
* Direct is the faster at querying, but the slowest at construction
* Default is the best all-around.
* Indirect isn't far behind Default.
* Direct is greatly influenced by the size of `T`.
 
Default in many cases beats Direct showing that sometimes a level of
indirection actually speeds things up. This is because in most cases,
the query algorithm just needs to check the aabb and doesnt need
to derefence. It also shows that it is in most
cases worth copying the aabb.


<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_query_0.1_128_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_query_1_128_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_query_0.1_32_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_query_0.1_8_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_query_0.1_256_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_query_0.01_128_bytes.svg" class="center" style="width: 100%;" />


<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_rebal_0.1_256_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_rebal_1_128_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_rebal_0.1_128_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_rebal_0.1_8_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_rebal_0.01_128_bytes.svg" class="center" style="width: 100%;" />
<img alt="Direct vs Indirect Query" src="graphs/dinotree_direct_indirect_rebal_0.1_32_bytes.svg" class="center" style="width: 100%;" />



## Different Data Layouts


There are three main datalayouts for each of the elements in a broccoli that are interesting:
`(Rect<isize>,&mut T)`
`(Rect<isize>,T)`
`&mut (Rect<isize>,T)`


Because the tree construction code is generic over the elements that are inserted (as long as they implement HasAabb),
The user can easily try all three data layouts.

The default layout is almost always the fastest. 
There are a few corner cases where if T is very small, and the bots are very dense, direct is faster, but it is marginal.

The default layout is good because during broccoli construction and querying we make heavy use of aabb's, but don't actually need T all that much. We only need T when we actually detect a collision, which doesnt happen that often. Most of the time we are just
ruling out possible colliding pairs by checking their aabbs.

Yes the times we do need T could lead to a cache miss, but this happens infrequently enough that that is ok.
Using the direct layout we are less likely to get a cache miss on access of T, but now the aabbs are further apart from each other
because we have all these T's in the way. It also took us longer to put the aabb's and T's all together in one contiguous memory.

One thing that is interesting to note is that if T has its aabb already inside of it, then `(Rect<isize>,&mut T)` duplicates that memory. This is still faster than `&mut (Rect<isize>,T)`, but it still feels wasteful. To get around this, you can make it so that T doesnt have the aabb inside of it, but it just has the information needed to make it. Then you can make the aabbs as you make the `(Rect<isize>,&mut T)` in memory. So for example T could have just in it the position and radius. This way you're using the very fast tree data layout of `(Rect<isize>,&mut T)`, but at the same time you don't have two copies of every objects aabb in memory. 

If we were inserting references into the tree, then the original order of the bots is preserved during construction/destruction of the tree. However, for the direct layout, we are inserting the actual bots to remove this layer of indirection. So when are done using the tree, we want to return the bots to the user is the same order that they were put in. This way the user can rely on indicies for other algorithms to uniquely identify a bot. To do this, during tree construction, we also build up a Vec of offsets to be used to return the bots to their original position. We keep this as a seperate data structure as it will only be used on destruction of the tree. If we were to put the offset data into the tree itself, it would be wasted space and would hurt the memory locality of the tree query algorithms. We only need to use these offsets once, during destruction. It shouldnt be the case that all querying algorithms that might be performed on the tree suffer performance for this.


## AABB vs Point + radius

Point+radius pros:
less memory (just 3 floating point values)
cons:
can only represent a circle (not an oval)
have to do more floating point calculations durying querying

AABB pros:
no floating point calculations needed during querying.
can represent any rectangle
cons:
more memory (4 floating point values)

Note, if the size of the elements is the same then the Point+radius only takes up 2 floating point values, so that might be better in certain cases. But even in this case, I think the cost of having to do floating point calculations when comparing every bot with every other bot in the query part of the algorithm is too much. With a AABB system, absolutely no floating point calculations need to be done to find all colliding pairs.

## AABB data layout

The aabb we use is made up of ranges that look like : start,end instead of start,length.  If you define them as a start and a length then querying intersections between rectangles requires that you do floating point arithmatic. The advantage of the start,end data layout is that all the broccoli query algorithms don't need to do a single floating point calculation. It is all
just comparisons. The downside, is that if you want the dimentions on the aabb, you have to calculate them, how this isnt something that any of the tree algorithms need. 
