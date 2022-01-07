### Web Demo

<script type=module>
    import { default as init } from './pkg/demo_web.js';
    var w=await init('pkg/demo_web_bg.wasm');
    await w.main_entry();
</script>
<div>
<canvas id="mycanvas" style="border-style: dotted" width="800" height="600"></canvas>
<button id="nextbutton">next</button>
<button id="shutdownbutton">shutdown</button>
</div>

### For the reader

In this book I'll go over a bunch of design problems/decisions while developing the [broccoli crate](https://crates.io/crates/broccoli) with performance analysis to back it up. 

The source code for this book is the repo [broccoli](https://github.com/tiby312/broccoli) on github.

### Graphs

The graphs are made using [poloto](https://github.com/tiby312/poloto) on github.

### Disclaimer

All the benches in the graphs were generated on my laptop which is a quad-core dell linux laptop and all the commentary was made against those graphs. However, you can pull down [github repo](https://github.com/tiby312/broccoli.git) and generate out the benches and this book again to get results specific to your platform. But keep in mind the commentary might not make sense then. 
