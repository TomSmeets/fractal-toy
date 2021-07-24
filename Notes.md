# Some design decisions


* declarative immediate mode is a nice api on the user side.
* declarative, with full state avaliable is nice on the library side

Apis should be immediate mode, and ensure that the hot pathts are very fast.
Either they use caching, or just pass ID's so that others can cache and request the data when needed.

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

+ functions can return
+ less allocation?
+ in this cace nicer interface, (but lets  make the borrow checker happy first :/)
- more chance of bugs?
+ imgui works like this, and that works very nicely
+ combines lookup and request

TODO: compare with declarative with other types of tasks


### Declarative mode
self.gpu.render(window, &GpuInput {
    resolution: input.resolution,
    viewport: &vp,
    tiles: &tiles,
});

+ Has full knowledge of everything
+ backend  can be anything, gpu just takes a list
- tiles has to be exactly that type. can't just convert between &[A], &[&A]] and BTreeMap<K, A>, etc

# SPEED

 toolchain  | link | full  | inc  | slow factor |
------------+------+-------+------+-------------+
 nightly    | lld  | 28.59 | 0.90 | 1x          |
 stable     | lld  | 30.41 | 1.91 | 2x          |
 nightly    | ld   | 31.69 | 3.65 | 4x          |
 stable     | ld   | 33.46 | 4.48 | 5x          |
