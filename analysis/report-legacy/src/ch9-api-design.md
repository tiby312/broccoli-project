### Ord vs Partial-Ord

As any rust user eventually learns, the floating point standard doesn't provide a total ordering of floats.
This makes it impossible to implement a true `max()` or `min()` function. Rust's floating point primitive types
only implement `PartialOrd` and not `Ord`, requiring that the user specify what to do in cases where there is no
clear comparison when using functions like max. 

broccoli construction and query requires sorting or finding the max and min at various times. There are basically
3 ways to handle floating points.

* Enforce `Ord`
    * Impossible for the user to incorrectly use the tree.
    * Cumbersome for the user to use wrapper types like `NotNan` or `OrderedFloat`
    * Wrapper types can incur performance hits. 
        * `OrderedFloat` has a slightly more expensive comparison
        * `NotNan` can introduce error paths making auto-vectorization not possible.

* Require `PartialOrd` and panic on failure
    * Impossible for the user to incorrectly use the tree.
    * Performance hit from extra checks and error paths in the query algorithms themselves.

* Require `PartialOrd` and silently fail.
    * No performance hit
    * Flexible to user.
        * User can still opt into using `Ord` types by passing their own wrapper types. They can
          define their own `from_ord()->Tree` tree building function that requires Ord.
    * User could mis-use the tree. They need to be cognizant that they are not passing NaN values
      if they are opting out of using a wrapper type.

For a long time I used the first option, but have since moved to the last option. Mainly because
it makes the code using this crate much more ergonomic and easier to read since there is no need
to juggle a wrapper type.


### Forbidding the user from violating the invariants of the tree statically

We want the user to be able to mutate the elements directly in the tree,
but we also do not want to let them mutate the aabb's of each of the elements of the tree. Doing so would
mean we'd need to re-construct the tree.

So when iterating over the tree, we can't return `&mut T`. So to get around that, there is `PMut<T>` that wraps around a `&mut T` and hides it but allows the user to access a mutable inner part, and a read-only aabb part.

So that makes the user unable to get a `&mut T`, but even if we just give the user a `&mut [PMut<T>]` where is another problem. The user could swap two node's slices around. So to get around that we use `PMut<[T]>` and `PMutIter<T>`.

But there is still another problem, we can't return a `&mut Node` either. Because then the user could swap the entire node
between two nodes in the tree. So to get around that we have a `PMut<Node>` that wraps around a `&mut Node`.

So that's a lot of work, but now the user is physically unable to violate the invariants of the tree at compile time and at the same time
we do not have a level of indirection. 

The downside to this static protection is the loss of the nice syntactic sugar using a regular `&mut T` would provide. The user has to manually extract the mutable inner part by calling `unpack_inner()`. 


### Knowing the axis at compile time

A problem with using recursion on an kd tree is that every time you recurse, you have to access a different axis, so you might have branches in your code. A branch predictor might have problems seeing the pattern that the axis alternate with each call. One way to avoid this would be to handle the nodes in bfs order. Then you only have to alternate the axis once every level. But this way you lose the nice divide and conquer aspect of splitting the problem into two and handling those two problems concurrently. So to avoid this, the axis of a particular recursive call is known at compile time. Each recursive call, will call the next with the next axis type. This way all branching based off of the axis is known at compile time. 


A downside to this is that the starting axis of the tree
must be chosen at compile time. It is certainly possible to create a wrapper around two specialized versions of the tree, one for each axis, but this would leads to a lot of generated code, for little benefit. Not much time is spent handling the root node anyway, so even if the suboptimal starting axis is picked it is not that big of a deal.

For this reason I hardcoded the starting divider to be a vertical line, partitioning aabbs based off of their x axis. This is completely arbitrary. In some cases, users might have more information about their particular distribution of aabbs to want a different starting axis, but the loss of choosing the not optimal starting axis isn't that big in most cases, I think. 

### Mutable vs Read-Only api

We could have just exposed read only versions of all the query functions where functions like
`find_all_colliding_pairs` just returned a read only reference instead of a mutable reference.
You could still use this api to mutate things by either inserting indexes or pointers. If you inserted
indices, then you would need to use unsafe to get mutable references to both elements simultaneously
in an array since the fact that both indices are distinct and don't alias is not known to the compiler.
If you inserted pointers, then you would need to use unsafe to cast them to mutable references.
So having to use unsafe is a downside.

An upside is that you would be able to run multiple raycast() function queries simultaneously, for example.

The main downside is loss of flexibility in cases where you want to store the actual elements inside the tree instead of just pointers or indices. In those cases, you obviously need mutable references to each element.


### Mutable vs Mutable + Read-Only api

Ideally, there would be both a `find_all_colliding_pairs` and a `find_all_colliding_pairs_mut`. 

A lot of the query algorithms don't actually care what kind of reference is in the tree.
They don't actually mutate the elements, they just retrieve them, and forward them on to the user
who may want to mutate them or not.

In this way, it would be nice if the query algorithms were generic of the type of reference they held. This way you could have a raycast(&mut Tree) function that allows you to mutate the elements it finds, or you could have a raycast(&Tree) function that allows you to call it multiple times in parallel.

To do this you can go down one of two paths, macros or generic associated types. GATs [don't exist yet](https://github.com/rust-lang/rfcs/blob/master/text/1598-generic_associated_types.md), and macros are hard to read and can be a head ache. Check out how rust reuses code between Iter and IterMut for slices for an example of macro solution. So for now, we will just support mutable api.

A good article about GATs can be found [here](https://lukaskalbertodt.github.io/2018/08/03/solving-the-generalized-streaming-iterator-problem-without-gats.html).
