# Some design decisions

## Simple API
The api should be very simple.
Something like this:

```rust
fn main() {
    let mut f = Fractal::new();
    f.add_builder(OCLBuilder())
    f.add_builder(CudaBuilder())
    f.add_builder(WgpuBuilder(device))

    loop {
        f.viewport = ...

        f.update_tiles(some_tile_builder);

        for pos,tile in f.tiles() {
            tile.draw()
        }
    }
}
```

This means:
* no feature flags for builders
* Fractal is an API
* this api should work everywhere


## Web support
* it runs everywhere
* no need to download or thrust anything

## Not using Imgui
* Imgui is nice, however the rust bindings are not nice to use
* No double support
* It uses an old imgui version
* Custom ui is way cooler

## Compilation time
* no more top level generics
* less generics overall
* acutally prifile compile time
