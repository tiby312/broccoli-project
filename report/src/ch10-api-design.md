
### Forbidding the user from violating the invariants of the tree statically

We have an interesting problem with our tree. We want the user to be able to mutate the elements directly in the tree,
but we also do not want to let them mutate the aabb's of each of the elements of the tree. Doing so would
mean we'd need to re-construct the tree.

So when iterating over the tree, we can't return `&mut T`. So to get around that, there is `PMut<T>` that wraps around a `&mut T` and hides it but does exposes the `Aabb` and `HasInner` interfaces. 

So that makes the user unable to get a `&mut T`, but even if we just give the user a `&mut [PMut<T>]` where is another problem. The user could swap two node's slices around. So to get around that we use `PMut<[T]>` and `PMutIter<T>`.

But there is still another problem, we can't return a `&mut Node` either. Because then the user could swap the entire node
between two nodes in the tree. So to get around that we have a `PMut<Node>` that wraps around a `&mut Node`.

So that's alot of work, but now the user is physically unable to violate the invariants of the tree at compile time and at the same time
we do not have a level of indirection. 


### Knowing the axis at compile time

A problem with using recursion on an kd tree is that every time you recurse, you have to access a different axis, so you might have branches in your code. A branch predictor might have problems seeing the pattern that the axis alternate with each call. One way to avoid this would be to handle the nodes in bfs order. Then you only have to alternate the axis once every level. But this way you lose the nice divide and conquer aspect of splitting the problem into two and handling those two problems concurrently. So to avoid, this, the axis of a particular recursive call is known at compile time. Each recursive call, will call the next with the next axis type. This way all branching based off of the axis is known at compile time. 


A downside to this is that the starting axis of the tree
must be chosen at compile time. It is certainly possible to create a wrapper around two specialized versions of the tree, one for each axis, but this would leads to alot of generated code, for little benefit. Not much time is spent handling the root node anyway, so even if the suboptimal starting axis is picked it is not that big of a deal.

### Mutable vs Read-Only api

We could have just exposed read only versions of all the query functions where functions like
`find_all_colliding_pairs` just returned a read only reference instead of a mutable reference.
You could still use this api to mutate things by either inserting indexes or pointers. If you inserted
indicies, then you would need to use unsafe to get mutable referneces to both elements simultaniously
in an array since the fact that both indicies are distinct and don't alias is not known to the compiler.
If you inserted pointers, then you would need to use unsafe to cast them to mutable references.
So having to use unsafe is a downside.

An upside is that you would be able to run multiple raycast() function queries simulaniously, for example.

The main downside is loss of flexibility in cases where you want to store the actual elements inside the tree instead of just pointers or indicies. In those cases, you obviously need mutable references to each element.


### Mutable vs Mutable + Read-Only api

Ideally, there would be both a `find_all_colliding_pairs` and a `find_all_colliding_pairs_mut`. 

A lot of the query algorithms don't actually care what kind of reference is in the tree.
They don't actually mutate the elements, they just retrieve them.

In this way, it would be nice if the query algorithms were generic of the type of reference they held. This way you could have a raycast(&mut Tree) function that allows you to mutate the elements it finds, or you could have a rayacst(&Tree) function that allows you to call it multiple times in parallel.

To do this you can go down one of two paths, macros or generic associated types. GATs [don't exist yet](https://github.com/rust-lang/rfcs/blob/master/text/1598-generic_associated_types.md), and macros are hard to read and can be a head ache. Check out how rust reuses code between Iter and IterMut for slices for an example of macro solution. So for now, we will just support mutable api.

A good article about GATs can be found [here](https://lukaskalbertodt.github.io/2018/08/03/solving-the-generalized-streaming-iterator-problem-without-gats.html).


### Making `Aabb` an unsafe trait vs Not

Making 'Aabb' unsafe allows us to make some assumptions that lets us do interesting things safely. If rust had [trait member fields](https://github.com/rust-lang/rfcs/pull/1546#issuecomment-304033345) we could avoid unsafe.

The key idea is the following:
If two rectangle queries do not intersect, then it is guarenteed that the elements are mutually exclusive.
This means that we can safely return mutable references to all the elements in the first rectangle,
and all the elements in the second rectangle simultaneously. 

For this to work the user must uphold the contract of `Aabb` such that the aabb returned is always the same while it is inserted in the tree.
This is hard for the user not to do since they only have read-only reference to self, but still possible using
RefCell or Mutex. If the user violates this, then despite two query rectangles being mutually exclusive,
the same bot might be in both. So at the cost of making HasAabb unsafe, we can make the MultiRect Api not unsafe.
