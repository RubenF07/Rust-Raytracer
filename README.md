## Rust Raytracer

A small, focused raytracer implemented in Rust.

- **Adapted implementation of:** [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html)
- **STL support:** personally implemented STL mesh loader with BVH optimization for fast ray intersections.

## Features

- Raytracing core written in Rust
- Per-mesh STL loading (custom implementation)
- Bounding Volume Hierarchy (BVH) optimization for faster rendering

## Build & Run

```bash
cargo run --release
```
