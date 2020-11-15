use crate::sdl::Sdl;
use fractal_toy::Viewport;
use fractal_toy::TEXTURE_SIZE;

pub enum Tile {
    Todo,
    Doing,
    Done(Vec<u8>),
    Used,
}

pub struct Config {}
pub struct TileMap {
    pub tiles: tilemap::TileMap<Tile>,
}
struct BuilderOCL {}
struct BuilderCPU {}

impl Config {
    fn new() -> Self {
        Self {}
    }
}

impl BuilderCPU {
    fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, map: &mut TileMap) {
        use fractal_toy::IsTileBuilder;
        use fractal_toy::TileParams;
        use fractal_toy::TileType;

        let mut b = fractal_toy::CPUBuilder::new();
        let p = TileParams {
            kind: TileType::Mandelbrot,
            iterations: 64,
            resolution: TEXTURE_SIZE as u32,
            padding: 1,
        };

        b.configure(&p);
        for (p, t) in map.tiles.iter_mut() {
            if let Tile::Todo = t {
                // *t = Tile::Done(vec![0, 255, 0, 0]);
                *t = Tile::Done(b.build(*p).pixels);
                break;
            }
        }
    }
}

impl BuilderOCL {
    fn new() -> Self {
        Self {}
    }
    pub fn update(&mut self, map: &mut TileMap) {
        // for (_, t) in map.tiles.iter_mut() {
        //     if let Tile::Todo = t {
        //         *t = Tile::Done(vec![0, 0, 255, 0]);
        //         break;
        //     }
        // }
    }
}

pub fn run() {
    let mut sdl = Sdl::new();

    let mut tile_map = TileMap::new();
    let mut config = Config::new();
    let mut viewport = Viewport::new(sdl.output_size());

    let mut builder_ocl = BuilderOCL::new();
    let mut builder_cpu = BuilderCPU::new();

    loop {
        let input = sdl.events();
        input.move_viewport(&mut viewport);
        input.update_config(&mut config);

        if input.is_quit() {
            break;
        }

        tile_map.update(&viewport);

        builder_ocl.update(&mut tile_map);
        builder_cpu.update(&mut tile_map);

        sdl.render(&tile_map, &viewport)
    }
}

impl TileMap {
    fn new() -> Self {
        Self {
            tiles: tilemap::TileMap::new(),
        }
    }

    fn update(&mut self, vp: &Viewport) {
        // Free textures
        let new_iter = vp.get_pos_all().map(|x| (x, ()));
        self.tiles
            .update_with(new_iter, |_, _| (), |_, _| Some(Tile::Todo));
    }
}
