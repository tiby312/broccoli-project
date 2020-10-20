

# Comparison of primitive types

The below chart shows performance using different primitive types for the aabbs. Notice that once parallelism is brought in, the differences between the types is not as big. It is interesting how much faster integers are than floats. 
The black line shows that it is faster to spend the time to convert the floats to integers, than
to use floats. This might not be true on all CPU architectures that might have better floating point hardware acceleration. However, intuitively, it makes sense for integer comparisons to be faster as there is just less work to do. A float is made up of a exponent and a mantisa, both of which are numbers that need to be compared against another float. An integer is just one number.

If you do convert your floats to integers, make sure to normalize it over all possible values of the integer to make it as accurate as possible. If it is important to you to not miss any interesections, then you'd also have make sure that the rouding is conservative always producing a bounding box that is slightly bigger than it needs to be.

<img alt="Float vs Integer" src="graphs/float_vs_integer.svg" class="center" style="width: 100%;" />



# When inserting float based AABBs, prefer NotNaN<> over OrderedFloat<>.

If you want to use this for floats, NotNaN has no overhead for comparisions, but has overhead for computation, the opposite is true for OrderedFloat.
For querying the tree colliding pairs, since no calculations are done, just a whole lot of comparisions, prefer NotNaN<>.
Other queries do require arithmatic, like raycast and knearest. So in those cases maybe ordered float is preferred.

