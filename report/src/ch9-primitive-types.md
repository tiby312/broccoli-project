

# Comparison of primitive types

The below chart shows performance using different primitive types for the aabbs. Notice that once parallelism is brought in, the differences between the types is not as big. It is interesting how much faster integers are than floats. 
The black line shows that it is faster to spend the time to convert the floats to integers, than
to use floats. This might not be true on all CPU architectures that might have better floating point hardware acceleration. However, intuitively, it makes sense for integer comparisons to be faster as there is just less work to do. A float is made up of a exponent and a mantisa, both of which are numbers that need to be compared against another float. An integer is just one number.

If you do convert your floats to integers, make sure to normalize it over all possible values of the integer to make it as accurate as possible. If it is important to you to not miss any interesections, then you'd also have make sure that the rouding is conservative always producing a bounding box that is slightly bigger than it needs to be.

One thing that surprised me is that ordered_float::OrderedFloat<T> is only slightly slower than ordered_float::NotNan<T>. I thought that because we are doing a lot of comparisons, the extra
overhead of the comparison function for OrderedFlot would add up, but it only slightly affects it. 

<img alt="Float vs Integer" src="graphs/float_vs_integer.svg" class="center" style="width: 100%;" />
