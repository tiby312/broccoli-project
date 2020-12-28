
### Code Layout

At a high level, I wanted all the code related to a specific query algorithm to be contained in one module.
So all the raycast code would be in a raycast module. However, at the same time, I wanted people to be able
to call raycast_mut() on the Tree object itself. One solution would be to have one raycast_mut() function
inside of the raycast module, and then Tree would also have a raycast function that would just be a wrapper
and call the one in the raycast module.

There are some problems with this approach. The obvious problem is added boilerplate of defining wrapper functions.
The other problem is that it raises the question of where should the function
documentation go. Clearly duplicating the documentation is not good. The user would be calling the function
attached to the Tree object which makes adding the documentation to that function seem like a good idea,
but our goal is to contain all the raycast code and documentation into the raycast module. This is why I used traits.

For the naive and assert functions, egronomics is not a concern since they are only used for debugging/testing the tree,
which a regular user of the crate shouldnt care about. So for those, we dont need to to the raycast function to
the tree itself. Instead the user has to call the function directly and pass the tree in as an argument. 

