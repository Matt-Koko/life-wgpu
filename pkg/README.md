<style>
  table {
    background-color: white;
    margin-left: auto;
    margin-right: auto;
  }
</style>

<h1 align="center">Life Wgpu</h1>

|    |
|:--:|
| <a href="https://www.rust-lang.org/"><img src="https://simpleicons.org/icons/rust.svg" width="50px" height="50px"/></a> + <a href="https://www.rust-lang.org/what/wasm"><img src="https://simpleicons.org/icons/webassembly.svg" width="50px" height="50px"/></a> + <a href="https://wgpu.rs/"><img src="https://wgpu.rs/logo.min.svg" width="50px" height="50px"/></a> |

### Description:
todo!();

### Licence:
[MIT](./LICENCE.txt). Go nuts.

### Run locally:
Set environment variable to use custom index.html:
```bash
export WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML="./index.html"
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

To serve the web build locally:
```bash
npx serve .
```

### Code References:
- [Learn Wgpu - sotrh](https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/)
- [Wgpu Examples](https://github.com/gfx-rs/wgpu/tree/trunk/examples)
- [Learn Wgpu - chris biscardi](https://www.youtube.com/playlist?list=PLWtPciJ1UMuBs_3G-jFrMJnM5ZMKgl37H)
- [Rust wgpu Graphics Programming Tutorial - Practical Programming with Dr. Xu](https://www.youtube.com/playlist?list=PL_UrKDEhALdJS0VrLPn7dqC5A4W1vCAUT)
- [WGPU For Beginners - GetIntoGameDev](https://www.youtube.com/playlist?list=PLn3eTxaOtL2PNbW4ou-APMV9W9m6nppYl)
