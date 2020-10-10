



## Liquid.

In liquid simulations the cost of querying dominates EVEN MORE than rebalancing since as opposed to particles that always repel, liquid particles repel if close to each other, but all attract if somewhat close. This means that the aabb's will intersect more often as the system tends to have overlapping aabbs.


## Rigid Bodies

If you want to use this tree for rigid bodies you have to overcome an obvious problem. You cannot move the bounding boxes once the tree it constructed. So while you are querying the tree to find bots that collide, you cannot move them then and there. An option is to insert loose bounding boxes and allow the bots to be moved within the loose bounding boxes. And then if the move need to be moved so much that they have to be moved out of the loose bounding boxes, re-construct the tree and query again until all the bots have finished moving, or you have done a set number of physics iterations, at which point you have some soft-body collision resolution fallback.

Ironically, even though to have a rigid body system you would need to have looser bounding boxes, performance of the system overall could improve. This is because rigid body systems enforce a level of spareness. In soft body, it is possible for literally every bot to be on touching each other causing many colliding pairs to be handled. A rigid body physics system would not allow this state to happen.

## Continuous Collision detection

In order to use dinotree for continuous collision detection (suitable for very fast objects, for example), the aabbs that you insert into it must be big enough to contain the position of a bot before and after the time step. This way, upon aabb collisions, you can do fine grained contiuous collision detection and resolution.

## 3D

What about 3d? Making this library multi dimensional would have added to the complexity, so the design desision was made to only target 2d. Its much easier for me as a developer to visualize 2d. So as a good first iteration of this library, targeting just 2d simplifies things. Expanding it to 3d, shouldnt take too much effort. The hard part would be over. Code architecture would hopefully not need to be changed much.  


That's not to say one couldn't still take advantage of this system in a 3d simulation. Every bot could store a height field that you do an extra check against in the collision function. The downside is that imagine if there were many bots stacked on top of each other, but you only wanted to query a small cube. Then doing it this way, your query function would have to consider all those bots that were stacked. If there are only a few different height values, one could maintain a seperte 2d dinotree for each level. Looking at the real world though, and most usecases, your potential z values are much less than our potetial x and y values. So for many cases, it probably actually better to use the tree for 2 dimentions, and then naively handling the 3rd. Then you dont suffer from the "curse of dimensionality"? An even better 3d solution might be to use sweep-and-prune in the z dimesion, and the 2d dinotree for x and y.

## Pipelining

It might be possible to pipeline the process so that rebalancing and querying happen at the same time with the only downside being that bots react to their collisions one step later. To account for that the aabb's could be made slightly bigger and predict what they will hit the next step. This system could almost double the performance if the system wasnt using all its cores for each stage (rebalancing/querying), but we're talking a lot of added complexity, but probably more systems don't have that many cores.
