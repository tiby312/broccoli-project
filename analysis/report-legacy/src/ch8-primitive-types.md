

### Comparison of primitive types

The below chart shows performance using different primitive types for the aabbs. 

You can see that it can be slightly faster to spend the time to convert the floats to integers, than to use floats. This might not be true on all CPU architectures.

You could also covert floats to `u16`. This way an entire `Rect<u16>` is only 64bits big. You loose some precision, but provided that you round up such that your bounding boxes are only ever slightly too big, this likely isnt a problem. We are using this tree for broad-phase collision detection - its already not precise. So why not use an imprecise number type if we are just trying to broadly determine colliding pairs.

If you do convert your floats to integers, make sure to normalize it over all possible values of the integer to make it as accurate as possible. If it is important to you to not miss any intersections, then you'd also have make sure that the rounding is conservative always producing a bounding box that is slightly bigger than it needs to be.



<link rel="stylesheet" href="css/poloto.css">
{{#include raw/float_vs_integer.svg}}

