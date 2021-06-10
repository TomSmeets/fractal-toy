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


## Perturbation theory

```
https://fractalwiki.org/wiki/Perturbation_theory

Z_n => cpu computed mandelbrot value
z_n => gpu computed offset

Z_(n+1) = Z_n^2 + C
Z_(n+1) + z_(n+1) = (Z_n+z_n)^2 + C + c
Z_(n+1) + z_(n+1) = Z_n^2 + z_n^2 + 2*Z_n*z_n + C + c
Z_(n+1) + z_(n+1) = (Z_n^2 + C) + (z_n^2 + c) + 2*Z_n*z_n
Z_(n+1) + z_(n+1) = Z_(n+1) + (z_n^2 + c) + 2*Z_n*z_n
          z_(n+1) = z_n^2 + c + 2*Z_n*z_n

```

TODO: test first with 64 bit cpu and 32 bit gpu!

we calculate one cpu Z for every tile. makes sense.


## Style

### Immediate mode
for t in viewport.visible_tiles() {
    let img = builder.build(p);
    gpu.tile(p, img);
}

gpu.draw(&viewport);


### Declarative mode
self.gpu.render(window, &GpuInput {
    resolution: input.resolution,
    viewport: &vp,
    tiles: &tiles,
});

