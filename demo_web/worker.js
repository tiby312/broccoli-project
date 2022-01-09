import { default as init } from './pkg/demo_web.js';
var w=await init('pkg/demo_web_bg.wasm');
await w.worker_entry();
