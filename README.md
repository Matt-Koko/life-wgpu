Code References:
- [Learn Wgpu - sotrh](https://sotrh.github.io/learn-wgpu/beginner/tutorial1-window/)
- [Dependencies and the Window | Learn Wgpu - chris biscardi](https://www.youtube.com/watch?v=knmuobQFNmM&list=PLWtPciJ1UMuBs_3G-jFrMJnM5ZMKgl37H)
- [Rust wgpu (3): Create a Colorful Triangle - Practical Programming with Dr. Xu](https://www.youtube.com/watch?v=hOojFOho_lI&list=PL_UrKDEhALdJS0VrLPn7dqC5A4W1vCAUT&index=3)


Compilation error:

```rust
error[E0597]: `window` does not live long enough
   --> src\lib.rs:29:47
    |
22  |     async fn new(window: &Window) -> Self {
    |                  ------ binding `window` declared here
...
29  |         let surface = instance.create_surface(&window).unwrap();
    |                       ------------------------^^^^^^^-
    |                       |                       |
    |                       |                       borrowed value does not live long enough
    |                       argument requires that `window` is borrowed for `'static`
...
103 |     }
    |     - `window` dropped here while still borrowed

error[E0521]: borrowed data escapes outside of associated function
  --> src\lib.rs:29:23
   |
22 |     async fn new(window: &Window) -> Self {
   |                  ------  - let's call the lifetime of this reference `'1`
   |                  |
   |                  `window` is a reference that is only valid in the associated function body
...
29 |         let surface = instance.create_surface(&window).unwrap();
   |                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |                       |
   |                       `window` escapes the associated function body here
   |                       argument requires that `'1` must outlive `'static`
```