## Mutable vs Mutable + Read-Only api

A lot of the query algorithms don't actually care what kind of reference is in the tree.
They don't actually mutate the elements, they just retrieve them.

In this way, it would be nice if the query algorithms were generic of the type of reference they held. This way you could have a raycast(&mut Tree) function that allows you to mutate the elements it finds, or you could have a rayacst(&Tree) function that allows you to call it multiple times in parallel.

To do this you can go down one of two paths, macros or generic associated types. GATs [don't exist yet](https://github.com/rust-lang/rfcs/blob/master/text/1598-generic_associated_types.md), and macros are hard to read and can be a head ache. Check out our rust reuses code between Iter and IterMut for slices for an example of macro solution.

So for now, we will just support mutable api.


## Making `Aabb` an unsafe trait vs Not

Making a trait unsafe is something nobody wants to do, but in this instance it lets as make some simple assumptions that lets us do interesting things safely. If rust had [trait member fields](https://github.com/rust-lang/rfcs/pull/1546#issuecomment-304033345) we could avoid unsafe.

The key idea is the following:
If two rectangle queries do not intersect, then it is guarenteed that the elements are mutually exclusive.
This means that we can safely return mutable references to all the elements in the first rectangle,
and all the elements in the second rectangle simultaneously. 

For this to work the user must uphold the contract of `Aabb` such that the aabb returned is always the same while it is inserted in the tree.
This is hard for the user not to do since they only have read-only reference to self, but still possible using
RefCell or Mutex. If the user violates this, then despite two query rectangles being mutually exclusive,
the same bot might be in both. So at the cost of making HasAabb unsafe, we can make the MultiRect Api not unsafe.

## Forbidding the user from violating the invariants of the tree statically

We have an interesting problem with our tree. We want the user to be able to mutate the elements directly in the tree,
but we also do not want to let them mutate the aabb's of each of the elements of the tree. Doing so would
mean we'd need to re-construct the tree.

So when iterating over the tree, we can't return `&mut T`. So to get around that, there is `PMut<T>` that wraps around a `&mut T` and hides it but does exposes the `Aabb` and `HasInner` interfaces. 

So that makes the user unable to get a `&mut T`, but even if we just give the user a `&mut [PMut<T>]` where is another problem. The user could swap two node's slices around. So to get around that we use `PMut<[T]>` and `PMutIter<T>`.

But there is still another problem, we can't return a `&mut Node` either. Because then the user could swap the entire node
between two nodes in the tree. So to get around that we have a `PMut<Node>` that wraps around a `&mut Node`.

So that's alot of work, but now the user is physically unable to violate the invariants of the tree at compile time and at the same time
we do not have a level of indirection. 
