

# Bounds checking vs no bounds checking

This shows the difference between using array indexing with and without bounds checking / unsafe.
As you can see, the no bounds checking version is faster, but it is by a pretty negligible amount.
The scale of the xaxis shows that the difference isn't really noticeable until x is very big. That said,
you can still notice a clear difference between the two in the graph.

<img alt="Bounds Checking" src="graphs/checked_vs_unchecked_binning.svg" class="center" style="width: 100%;" />
