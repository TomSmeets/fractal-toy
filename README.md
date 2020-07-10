Fractal Toy
===========

This is a fractal explorer made with simplicity in mind.


# Building

For now only the `sdl` target works correctly.
This is also only tested on linux.

```
cargo run --bin sdl
```

# Controls

## Keyboard

* `W` `A` `S` and `D` for movement
* `I` and `K` for zooming
* `J` and `L` change the number of iterations
* `N` cycle fractal types 

The following are mostly for debugging

* `1` toggle tile generation
* `2` toggle debug overlay
* `5` save state 
* `6` load state

Saved state is stored in the directory `.fractal-toy/` in the current working directory
