
## Pipelining

It might be possible to pipeline the process so that rebalancing and querying happen at the same time with the only downside being that bots react to their collisions one step later. To account for that the aabb's could be made slightly bigger and predict what they will hit the next step. 
However, the construction and querying phase are already parallelized. Making those happen in parallel will probably confused the rayon's work stealer.

## Liquid.

In liquid simulations the cost of querying dominates even more than construction since as opposed to particles that repel when touching, liquid particles react even when just close to each other. This means that the aabb's will intersect more often as the system tends to have overlapping aabbs.

## Rigid Bodies

If you want to use this tree for rigid bodies you have to deal with an obvious problem. You cannot move the bounding boxes once the tree it constructed. So while you are querying the tree to find bots that collide, you cannot move them then and there. An option is to insert loose bounding boxes and allow the bots to be moved within the loose bounding boxes. And then if the move need to be moved so much that they have to be moved out of the loose bounding boxes, re-construct the tree and query again until all the bots have finished moving, or you have done a set number of physics iterations, at which point you have some soft-body collision resolution fallback.

Ironically, even though to have a rigid body system you would need to have looser bounding boxes, performance of the system overall could improve. This is because rigid body systems enforce a level of spareness. In soft body, it is possible for literally every bot to be on touching each other causing many colliding pairs to be handled. A rigid body physics system would not allow this state to happen.

However, you should ignore everything I just said. A much better solution is to not move the bots at all. You can still have rigid body physics by just doing many passes on the velocities. Check out Erin Catto's Box2D library, as well as some of his [talks](https://www.youtube.com/watch?v=SHinxAhv1ZE&t=2042s).

## Continuous Collision detection

In order to use broccoli for continuous collision detection (suitable for very fast objects, for example), the aabbs that you insert into it must be big enough to contain the position of a bot before and after the time step. This way, upon aabb collisions, you can do fine grained contiuous collision detection and resolution. broccoli is well suited for this use-case because it makes no assumptions about the sizes of the aabbs. There is no rule that they must all be the same size or have a maximum size.

## 3D

What about 3D? Making this library multi dimensional would have added to the complexity, so I only targeted 2D. 

That said, you could certainly use broccoli to partition 2 dimensions, and then use another method to partition the last dimension (perhaps a 1D version of sweep and prune).