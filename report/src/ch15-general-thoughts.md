
#### Multithreading

Evenly dividing up work into chunks for up to N cores is preferable. You don't want to make assumptions about how many cores the user has. Why set up your system to only take up advantage of 2 cores if 16 are available. So here, using a thread pool library like rayon is useful. 

In a big complicated system, it can be tempting to keep sub-systems sequential and then just allow multiple sub-systems to happen in parallel if they are computation heavy. But this is not fully taking advantage of multithreading. You want each subsystem to itself be parallelizable. You want to parallelize in a platform independent way exploiting as many cores as are available.


#### Testing correctness

A good test is a test that tests with good certainty that a large portion of code is working properly and that is itself short.
Maintaining tests comes at the cost of anchoring down the design of the production code in addition to having to be maintained themselves. As a result, making good abstractions between your crates and modules that have simple and well defined apis is very important. Then you can have a few simple tests to fully exercise an api and verify large amounts of code.

This crate's sole purpose is to provide a method of providing collision detection that is faster than the naive method. So a good high level test would be to compare the query results from using this crate to the naive method (which is much easier to verify is correct). This one test can be performed on many different inputs of lists of aabbs to try to expose any corner cases. So this one test when fed with both random and specifically hand tailored inputs to expose corner cases can show with a lot of certainty that the crate is satisfying the api. 

Simply using rust has a big impact on testing. Because of its heavy use of static typing, many bugs are caught at compile time. This translates to less testing as there are fewer possible paths that the produced program can take. 

The fact that the api is generic over the underlying number type used is useful. This means that we can test the system using integers and we can expect it to work for floats. It is easier to test with integers since we can more easily construct specific scenarios where one number is one value more or less than another. So in this way we can expose corner cases.


#### Benching

Always measure code before investing time in optimizing. As you design your program. You form in your mind ideas of what you think the bottle necks in your code are. When you actually measure your program, your hunches can be wildly off.

Platform dependance. Rust is a great language that strives for platform independent code. But at the end of the day, even though rust programs will behave the same on multiple platforms, their performance might be wildly different. And as soon as you start optimizing for one platform you have to wonder whether or not you are actually de-optimizing for another platform. For example, rebalancing is much slower on my android phone than querying. On my dell xps laptop, querying is the bottle neck instead. I have wondered why there is this disconnect. I think part of it is that rebalancing requires a lot of sorting, and sorting is something where it is hard to predict branches. So my laptop probably has a superior branch predictor. Another possible reason is memory writing. Rebalancing involves a lot of memory swapping, whereas querying does not involve any major writing to memory outside of what the user decides to do for each colliding pair. In any case, my end goal in creating this algorithm was to make the querying as fast as possible so as to get the most consistent performance regardless of how many aabbs were colliding.

When dealing with parallelism, benching small units can give you a warped sense of performance. Onces the units are combined, there may be more contention for resources like work stealing. With small units, you have a false sense that the cpu's are not busy doing other things. For example, I parallelized creating the container range for each node. Benchmarks showed that it was faster. But when I benched the rebalancing as a whole, it was slower with the parallel container creation. So in this way, benching small units isn't quite as useful as testing small units is. That said, if you know that your code doesn't depend on some global resource like a threadpool, then benching small units is great.

#### Level of Indirection

Sometimes slow things can be a tool to make things fast. Normally people thinking adding a level of indirection is an automatic performance hit, but a level of indirection can be seen as a tool that can speed up your program. If you have a data structure composed of (X,Y), and X is accessed frequently but not Y, then
if you add a level of indirection such that you have (X,&mut Y), then your data structure is composed
of smaller elements making more of it fit in memory at once. This of course only makes sense if Y is big enough.

#### Dynamic Allocation

Similarly you can use dynamic allocation as a tool to speed up your program. It has a cost, but the idea is that with that allocated memory you can get more performance gains. The problem is that everybody has to buy into the system for it to work. Anybody who allocated a bunch of memory and doesn't return it because they want to avoid allocating it again is hogging that space for longer than it needs it.

Often times, its not the dynamic allocation that is slow, but some other residual of doing it in the first place. For example, dynamically allocating a bunch of pointers to an array, and then sorting that list of pointers. The allocation is fast, its just that there is no cache coherency. Two pointers in your list of pointers could very easily point to memory locations that are very far apart.

Writing apis that don't do dynamic allocation is tough and can be cumbersome, since you probably have to have the user give you a slice of a certain size, but typically you need to first get the problem size from the user to figure out the amount of memory you want to request.
On one hand the level of explicitness is great and users don't have to put any faith in allocation system. But on the other hand it adds a lot of logic to your api that makes it harder to see what your library actually does. 
