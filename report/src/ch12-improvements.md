
#### Temporal Coherence between tree constructions

It would be interesting to use last tree constructions dividers as pivots 
for the next tree construction. This would require touching the pdqselect 
crate to accept custom pivots. Not sure what the gains here would be, though
considering the level balance charts indicate that even in the best case,
rebalancing does get that much faster (in cases where good pivots are chosen).


#### Don't sort the leafs

If you don't sort the leafs, there could be some potential speed up. By the time you get to the leafs, there are so few aabbs in a leaf that it may not be worth it. The aabbs also would not be strewn along a dividing line so sweep and prune would not be as fast.  However, this can only hurt the query algorithm so I didn't do it. However, if you want to make one (construct+query) sequence as fast as possible it might be better. But again, my goal was to make querying as fast as possible.

#### Sort away from the divider.

Currently, all elements are sorted using the left or top side of the aabb. It would be interesting if depending on the direction you recurse, you sorted along the left or right side of the aabb. This might help pruning elements from nodes on perpendicular axis. It also make the algorithm have a nice symmetry of behaving exactly the same in each half of a partition. The downside is more code generated and complexity.

#### Pointer Compression

The broccoli tree data structure can be very pointer heavy. There may be some gains from using pointer compression if only during construction. During the query phase, i'm certain that using pointer compression would be slow given the extra overhead of having to unpack each pointer. However, if the tree was constructed with `BBox<N,u16>` which was then converted to `BBox<N,&mut T>` then maybe construction would be faster provided the conversion isnt too slow.
A cleaner solution would just be to target a 32bit arch instead of 64bit. (Sidenode: The webassembly arch is 
32bit)

#### Continuous Collision detection

In order to use broccoli for continuous collision detection (suitable for very fast objects, for example), the aabbs that you insert into it must be big enough to contain the position of a bot before and after the time step. This way, upon aabb collisions, you can do fine grained contiuous collision detection and resolution. broccoli is well suited for this use-case because it makes no assumptions about the sizes of the aabbs. There is no rule that they must all be the same size or have a maximum size.

#### Colliding every other frame

If you populate the tree with loose bounding boxes that is big enough to cover all the places
a bot could end up in one iteration, you could save finding the colliding pairs every other iteration. To do this the `collect` functions are useful for saving query results for the intermediate steps.

#### Pipelining

It might be possible to pipeline the process so that rebalancing and querying happen at the same time with the only downside being that aabbs react to their collisions one step later. To account for that the aabb's could be made slightly bigger and predict what they will hit the next step. 
However, the construction and querying phase are already parallelized. Making those happen in parallel will probably confuse the rayon's work stealer. However maybe if there are somehow two independant threadpools this could get you the speed up. However, its unclear to me if it would be faster because you'd have to insert slightly bigger aabbs which would slow down querying. 

#### Liquid.

In liquid simulations the cost of querying dominates even more than construction since as opposed to particles that repel when touching, liquid particles react even when just close to each other. This means that the aabb's will intersect more often as the system tends to have overlapping aabbs.

#### Rigid Bodies

If you want to use this tree for true rigid bodies you have to deal with an obvious problem. You cannot move the bounding boxes once the tree it constructed. So while you are querying the tree to find aabbs that collide, you cannot move them then and there. An option is to insert loose bounding boxes and allow the aabbs to be moved within the loose bounding boxes. And then if the move need to be moved so much that they have to be moved out of the loose bounding boxes, re-construct the tree and query again until all the aabbs have finished moving, or you have done a set number of physics iterations, at which point you have some soft-body collision resolution fallback.

Ironically, even though to have a rigid body system you would need to have looser bounding boxes, performance of the system overall could improve. This is because rigid body systems enforce a level of spareness. In soft body, it is possible for literally every bot to be on touching each other causing many colliding pairs to be handled. A rigid body physics system would not allow this state to happen.

However, you should ignore everything I just said. A much better solution is to not move the aabbs at all. You can still have rigid body physics by just doing many passes on the velocities. Check out Erin Catto's Box2D library, as well as some of his [talks](https://www.youtube.com/watch?v=SHinxAhv1ZE&t=2042s).

#### 3D

What about 3D? Making this library multi dimensional would have added to the complexity, so I only targeted 2D. That said, you could certainly use broccoli to partition 2 dimensions, and then use another method to partition the last dimension (perhaps a 1D version of sweep and prune). Infact, in situations where things are more likely to intersect on the x and y plane and less so on the z plane, it might be faster to use a system where you have a broccoli Tree for the x and y dimensions, ad then store the z dimension in a level of indirection. This way each element is smaller.