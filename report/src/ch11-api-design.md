TODO talk about general usecase. 
Different types of entities. collision checking groups of entities instead of
collision checking all at once.

## Mutable vs Mutable + Read-Only api

Most of the query algorithms only provide an api where they operate on a mutable reference
to the broccoli and return mutable reference results.
Ideally there would be sibling query functions that take a read-only broccoli
and produce read-only reference results. 
The main benefit would be that you could run different types of query algorithms on the tree
simulatiously safely. 
In rust right now, there isn't a easy way to re-use code
between two versions of an algorithm where one uses mutable references and one uses read-only references.
In the rust source code, you can see they do this using macros. From rust core's slice code:

```ignore
// The shared definition of the `Iter` and `IterMut` iterators
macro_rules! iterator {
	...
}
```

But I didnt want to make these already complicated query algorithms even harder to read by
wrapping them all in macros. The simplest query algorithm, rect querying, does provide read-only versions.
I'm hoping that eventualy you will be able to 'parameterize over mutability' in rust so that i dont have to use macros.

In any-case, there arn't many use-cases that I can think of where you'd want read-only versions of certain 
query algorithms. For example, finding all colliding pairs. What would the user do with only read-only references
to all the colliding pairs? There may be some cases where the user would just want to draw or list all the colliding pairs,
but most use-cases I can think of, you want to actually mutate the pairs in some way.

Other query algorithms I can see some benefits. For example, raycasting. In raycasting the user might just want to
draw a line to the element that is returned from the raycast, in which case the user wouldn't need a mutable reference
to the result. So if I were to provide a read-only raycast api, and the user wanted to raycast many times, then that could
be easily be parallelized. For now, the user must use the mutable raycast api and handle each raycast sequentially. This
is simplier to implement (no macros), and the raycast algorithm is already very fast compared to other algrithms such
as constructing the tree and finding colliding pairs.


## Making HasAabb an unsafe trait vs Not

Making a trait unsafe is something nobody wants to do, but in this instance it lets as make some simple assumptions
that lets us do interesting things safely. 

The key idea is the following:
If two rectangle queries do not intersect, then it is guarenteed that the elements are mutually exclusive.
This means that we can safely return mutable references to all the elements in the first rectangle,
and all the elements in the second rectangle simultaneously. 

For this to work the user must uphold the contract of HasAabb such that the aabb returned is always the same
while it is inserted in the tree.
This is hard for the user not to do since they only have read-only reference to self, but still possible using
RefCell or Mutex. If the user violates this, then despite two query rectangles being mutually exclusive,
the same bot might be in both. So at the cost of making HasAabb unsafe, we can make the MultiRect Api not unsafe.



## Forbidding the user from violating the invariants of the tree

We have an interesting problem with our tree. We want the user to be able to mutate the elements directly in the tree,
but we also do not want to let them mutate the aabb's of each of the elements of the tree. Doing so would
mean we'd need to re-construct the tree.

So when iterating over the tree, we can't return &mut T. So to get around that, there is ProtectedBBox that wraps
around a &mut T that also implements HasAabb. 

So that makes the user unable to get a &mut T, but even if we just give the user a &mut [ProtectedBBox<T>] where is another problem. The user could swap two node's slices around. So to get around that we have ProtectedBBoxSlice that wraps
around a &mut [ProtectedBBox<T>] and provides an interator who Item is a ProtectedBBox<T>.

But there is still another problem, we can't return a &mut Node either. Because then the user could swap the entire node
between two nodes in the tree. So to get around that we have a ProtectedNode that wraps around a &mut Node.

So that's alot of work, but now the user is physically unable to violate the invariants of the tree and at the same time
we do not have a level of indirection. It is tempting
to just let it be the user's responsibility to not violate the invariants of the tree, and that would be a fine design
choice if it weren't for the fact that HasAabb is unsafe (See Making HasAabb an unsafe trait vs Not Section).
