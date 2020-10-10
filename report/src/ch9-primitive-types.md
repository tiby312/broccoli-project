

# Comparison of primitive types

The below chart shows performance using different primitive types for the aabbs. Notice that once parallelism is brought in, the differences between the types is not as big. It is interesting how much faster integers are than floats. It makes me wonder
if in some cases it might be worth while to convert all floats to integers during construction and then do everything with 
integers.

<img alt="Float vs Integer" src="graphs/float_vs_integer.svg" class="center" style="width: 100%;" />



# When inserting float based AABBs, prefer NotNaN<> over OrderedFloat<>.

If you want to use this for floats, NotNaN has no overhead for comparisions, but has overhead for computation, the opposite is true for OrderedFloat.
For querying the tree colliding pairs, since no calculations are done, just a whole lot of comparisions, prefer NotNaN<>.
Other queries do require arithmatic, like raycast and knearest. So in those cases maybe ordered float is preferred.

