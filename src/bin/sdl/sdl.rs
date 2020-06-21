use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use serial::atlas::AtlasTextureProvider;
use serial::fractal::TEXTURE_SIZE;
use serial::math::Rect as MRect;
use serial::math::*;

pub struct Sdl {
    /// ~~SDL_Quit is called when dropped, so it has to be kept alive~~
    /// Never mind, that is not true, the context is only dropped when all SDL
    /// elements are dropped. So it is not necessary to keep the context or
    /// subsystem in memory. I will however keep these fields. as to make it
    /// explicit that we are using this.
    pub ctx: sdl2::Sdl,
    pub video: sdl2::VideoSubsystem,
    pub controller: sdl2::GameControllerSubsystem,
    pub event: sdl2::EventPump,
    pub canvas: Canvas<Window>,
}

impl Sdl {
    pub fn new() -> Self {
        let ctx = sdl2::init().unwrap();
        let video = ctx.video().unwrap();

        let window = video
            .window("rust-sdl2 demo", 800, 600)
            .resizable()
            .opengl()
            .position_centered()
            .build()
            .unwrap();

        // IMPOTANT: This causes some issues in older sdl2 versions
        // For some reason this causes horrible lag spikes when polling events
        // see https://stackoverflow.com/a/53658644
        // however i have also noticed the same issue when on version '355a4f94a782' of sdl2
        // not sure which commit fixed it, but with the stable sdl-2.0.12 it seems to work
        let controller = ctx.game_controller().unwrap();

        let event = ctx.event_pump().unwrap();
        let mut canvas = window.into_canvas().present_vsync().build().unwrap();

        canvas.set_blend_mode(BlendMode::Blend);

        unsafe {
            sdl2::sys::SDL_SetHint(
                sdl2::sys::SDL_HINT_RENDER_SCALE_QUALITY.as_ptr() as *const i8,
                (b"0").as_ptr() as *const i8,
            );
        }

        Sdl {
            ctx,
            video,
            controller,
            event,
            canvas,
        }
    }

    pub fn canvas_copy(
        &mut self,
        texture: &sdl2::render::Texture,
        src: Option<Rect>,
        dst: Option<Rect>,
    ) {
        self.canvas.copy(texture, src, dst).unwrap();
    }

    pub fn output_size(&self) -> Vector2<u32> {
        let (x, y) = self.canvas.output_size().unwrap();
        Vector2::new(x, y)
    }

    pub fn create_texture_static_rgba8(&mut self, w: u32, h: u32) -> sdl2::render::Texture {
        self.canvas
            .texture_creator()
            .create_texture_static(PixelFormatEnum::ABGR8888, w, h)
            .unwrap()
    }
}

impl Default for Sdl {
    fn default() -> Self {
        Self::new()
    }
}

impl AtlasTextureProvider for Sdl {
    type Texture = Texture;

    fn alloc(&mut self, width: u32, height: u32) -> Texture {
        self.create_texture_static_rgba8(width, height)
    }

    fn free(&mut self, t: Texture) {
        unsafe {
            t.destroy();
        }
    }

    fn update(&mut self, texture: &mut Texture, rect: MRect, pixels: &[u8]) {
        texture
            .update(Some(rect.to_sdl()), pixels, 4 * TEXTURE_SIZE as usize)
            .unwrap();
    }
}
