
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
but our goal is to contain all the raycast code and documentation into the raycast module.

The soltuion I went with was making each query module define a trait. Then the Tree would implement all of them.
This way all the code and documentaiton for a query algorith is all contained inside of one module and there are no wrapper functions.  

For the naive and assert functions, egronomics is not a concern since they are only used for debugging/testing the tree,
which a regular user of the crate shouldnt care about. So for those, we dont need to to the raycast function to
the tree itself. Instead the user has to call the function directly and pass the tree in as an argument. 

A downside to the current approach is that there is a kind of circular dpenendency with the current setup. The knearest module depends on Tree and Tree depends on the knearest module. This can be fixed by introducing another type TreeCore. Then you could have Tree depending on knearest module depending on TreeCore. Tree could deref to TreeCore. So TreeCore would provide the data structure and visitor functions, and then Tree would just be a wrapper around TreeCore providing high level query functions like raycast(). However, this didnt seem worth introducing a new type.
