

### Bounds checking vs no bounds checking

This shows the difference between using array indexing with and without bounds checking / unsafe. As you can see there is basically no difference. So there is literally no point in trying to avoid indexing because of the possibility of panicking.


<link rel="stylesheet" href="css/poloto.css">
{{#include raw/checked_vs_unchecked_binning.svg}}
