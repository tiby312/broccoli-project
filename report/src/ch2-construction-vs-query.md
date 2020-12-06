
### Rebalancing vs Querying

The below charts show the load balance between the construction and querying through calling
`find_colliding_pairs` on the broccoli.
It's important to note that the comparison isnt really 'fair'. The cost of querying depends a lot on
what you plan on doing with every colliding pair (it could be an expensive user calculation). Here we just use a 'reasonably' expensive calculation that repels the colliding pairs.

Some observations:
* The cost of rebalancing does not change with the density of the objects
* If the aabbs are spread out enough, the cost of querying decreases enough to be about the same as rebalancing.
* The cost of querying is reduced more by parallelizing than the cost of rebalancing.
	
It makes sense that querying in more 'parallelizable' than rebalancing since the calculation that you have to perform for each node before you can divide and conquer the problem is more expensive for rebalancing. For rebalancing you need to find the median and bin the aabbs. For querying you only have to do sweep and prune. 

<img alt="Construction vs Query" src="graphs/construction_vs_query_grow_theory.svg" class="center" style="width: 100%;" />
<img alt="Construction vs Query" src="graphs/construction_vs_query_grow_bench.svg" class="center" style="width: 100%;" />
<img alt="Construction vs Query" src="graphs/construction_vs_query_num_theory.svg" class="center" style="width: 100%;" />
<img alt="Construction vs Query" src="graphs/construction_vs_query_num_bench.svg" class="center" style="width: 100%;" />


### Construction Cost vs Querying Cost

If you are simulating moving elements, it might seem slow to rebuild the tree every iteration. But from benching, most of the time querying is the cause of the slowdown. Rebuilding is always a constant load, but the load of the query can vary wildly depending on how many elements are overlapping.

Any kind of heuristics or caching strategy to speed up construction will result in a sub optimal partitioning of the space which the querying side will pay for. Therefore it is not worth it. Instead focus on speeding up the construction of an optimal partitioning. This can be done via parallelism. Sorting the aabbs that sit on dividing lines may seem slow, but we can get this for 'free' essentially because it can be done after we have already split up the children nodes. So we can finish sorting a node while the children are being worked on.

For example, in a bench where inside of the collision call-back function I do a reasonable collision response with 80_000 aabbs, if there are 0.8 times (or 65_000 ) collisions or more, querying takes longer than rebuilding. For your system, it might be impossible for there to even be 0.8 * n collisions, in which case building the tree will always be the slower part. For many systems, 0.8 * n collisions can happen. For example if you were to simulate a 2d ball-pit, every ball could be touching 6 other balls [Circle Packing](https://en.wikipedia.org/wiki/Circle_packing). So in that system, there are around 3 * n collisions. So in that case, querying is the bottle neck. With liquid or soft-body physics, the number can be every higher. up to n * n.

Rebuilding the first level of the tree does take some time, but it is still just a fraction of the entire building algorithm in some crucial cases, provided that it was able to partition almost all the aabbs into two planes. 

Additionally, we have been assuming that once we build the tree, we are just finding all the colliding pairs of the elements. In reality, there might be many different queries we want to do on the same tree. So this is another reason we want the tree to be built to make querying as fast as possible, because we don't know how many queries the user might want to do on it. In addition to finding all colliding pairs, its quite reasonable the user might want to do some k_nearest querying, some rectangle area querying, or some raycasting.

### Inserting elements after construction

broccoli does not support inserting elements after construction. If you want to add more elements,
you have to rebuild the tree. In most usecases, I think you want to do this anyway. In a dynamic
particle system for example, most of the time, enough particles move about in one step to justify
recreating the whole tree. Trying to avoid this using loose bounding boxes will make querying take
longer. 


### Exploiting Temporal Locality (with loose bounding boxes)

The main reason against exploiting temporal locality is that adding any kind of "memory" to the tree where you save the positions of the dividers to use as good heuristic positions for next iterations will come at a cost of a sub optimal tree layout which will hurt the query algorithm. Our goal is to make the query algorithm as fast as possible since that is what can dominate.

One strategy to exploit temporal locality is by inserting looser bounding boxes into the tree and caching the results of a query for longer than one step. The upside to this is that you only have to build and query the tree every couple of iterations. There are a number of downsides, though:

* Your system performance now depends on the speed of the aabbs. The faster your aabbs move, the bigger their loose bounding boxes, the slower the querying becomes. This isnt a big deal considering the ammount that a bot moves between two frames is expected to be extremely small. But still, there are some corner cases where performance would deteriorate. For example, if every bot was going so fast it would just from one end of you screen to the other between world steps. So you would also need to bound the velocity of your aabbs to a small value.

* You have to implement all the useful geometry tree functions all over again, or you can only use the useful geometry functions at the key world steps where the tree actually is constructed. For example, if you want to query a rectanlge area, the tree provides a nice function to do this, but you only have the tree every couple of iterations. The result is that you have to somehow implement a way to query all the aabbs in the rectangle area using your cached lists of colliding aabbs, or simply only query on the world steps in which you do have the built tree. Those queries will also be slower since you are working on a tree with loose boxes.

* Every bot needs to have a member variable that is its index. This isnt ideal to have since its redundant information. You can figure out a aabbs index from its position within the list fed into the tree. So it is just wasted space. For a very intestive algorithms like collision querying, having the memory footprint being operated being small is crucial. We can't rely on pointer offsets to determine the indicies of which aabbs are colliding when using the tree since we reordered the aabbs directly to make the tree to avoid a level of indirection. This increases the separation between the other fields.

* The maximum load on a world step is greater. Sure amortised, this caching system may computation, but the times you do construct and query the tree, you are doing so with loose bounding boxes. On top of that, while querying, you also have to build up a seperate data structure that caches the colliding pairs you find. 

* The api of the broccoli is flexible enough that you can implement loose bounding box + caching on top of it (without sacrificing parallelism) if desired.

So in short, this system doesnt take advantage of temporal locality, but the user can still take advantage of it by inserting loose bounding boxes and then querying less frequently to amortize the cost. I didnt explore this since I need to construct the tree every iteration anyway in my android demo, because I wanted the feedback of the user moving his finger around to be imeddiate. So to find all the aabbs touching the finger i need the tree to be up to date every single iteration. This is because I have no way of know where the user is going to put his finger down. I cant bound it by velocity or acceleration or anything. If I were to bound the touches "velocity", it would feel more slugish i think. It would also delay the user putting a new touch down for one iteration possibly.

### Expoiting Temporal Locality (caching medians)

I would love to try the following: Instead of finding the median at every level, find an approximate median. Additionally, keep a weighted average of the medians from previous tree builds and let it degrade with time. Get an approximate median using median of medians. This would ensure worst case linear time when building one level of the tree. This would allow the rest of the algorithm to be parallelized sooner.

This would mean that query would be slower since we are not using heuristics and not using the true median, but this might be a small slowdown and it might speed of construction significatly.

For a while I had the design where the dividers would move as those they had mass. They would gently be pushed to which ever side had more aabbs. The problem with this approach is that the divider locations will mostly of the time be sub optimial. And the cost saved in rebalancing just isnt enough for the cost added to querying with a suboptimal partitioning. By always partitioning optimally, we get guarentees of the maximum number of aabbs in a node. Remember querying is the bottleneck, not rebalancing.



