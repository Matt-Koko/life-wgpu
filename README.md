<h1 align="center">Life Wgpu
</br>
<a href="https://www.rust-lang.org/"><img src="https://simpleicons.org/icons/rust.svg" width="50px" height="50px"/></a>
+
<a href="https://www.rust-lang.org/what/wasm"><img src="https://simpleicons.org/icons/webassembly.svg" width="50px" height="50px"/></a>
+
<a href="https://wgpu.rs/"><img src="https://wgpu.rs/logo.min.svg" width="50px" height="50px"/></a>
</h1>

### Description:
todo!();

### Run locally:
Set environment variable to use custom index.html:
```bash
export WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML="./src/in_dev.html"
```

In browser:
```bash
cargo run --target wasm32-unknown-unknown
```

In browser with hot reload:
```bash
cargo watch -x "run --target wasm32-unknown-unknown"
```

In desktop:
```bash
cargo run
```

### Build for web:
```bash
wasm-pack build --target web
```
This will create a `pkg` folder with the web build. See `index.html` for usage.


### Code References:
- [Learn Wgpu - sotrh](https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/)
- [Dependencies and the Window | Learn Wgpu - chris biscardi](https://www.youtube.com/watch?v=knmuobQFNmM&list=PLWtPciJ1UMuBs_3G-jFrMJnM5ZMKgl37H)
- [Rust wgpu (3): Create a Colorful Triangle - Practical Programming with Dr. Xu](https://www.youtube.com/watch?v=hOojFOho_lI&list=PL_UrKDEhALdJS0VrLPn7dqC5A4W1vCAUT&index=3)
- [Wgpu Examples](https://github.com/gfx-rs/wgpu/tree/trunk/examples)
