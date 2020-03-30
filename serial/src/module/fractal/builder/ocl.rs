use super::queue::*;
use crate::module::fractal::{builder::TileRequest, tile::TileContent};
use ocl::{
    enums::{AddressingMode, FilterMode, ImageChannelDataType, ImageChannelOrder, MemObjectType},
    flags::CommandQueueProperties,
    Context, Device, Image, Kernel, Program, Queue, Sampler,
};
use std::sync::{Arc, Mutex};

use crate::module::fractal::TEXTURE_SIZE;

static src: &'static str = r#"
    __kernel void add(write_only image2d_t image) {
        int2 coord = (int2)(get_global_id(0), get_global_id(1));
        float2 p = (float2)((float) coord.x / 128.0, (float) coord.y / 128.0);
        // abgr
        float4 pixel = (float4)(1.0, p.x, p.y, 0.0);

        float2 z = (float2)(0, 0);
        float2 c = p*4.0f - 2.0f;

        float n = 0.0f;
        for(int i = 0; i < 1024; ++i) {
            float2 t = z;
            z.x = t.x*t.x - t.y*t.y;
            z.y = 2.0f*t.x*t.y;
            z += c;
            n += 1.0;

            if (z.x*z.x + z.y*z.y > 4.0f) {
                pixel *= 0.0f;
                break;
            }
        }

        write_imagef(image, coord, pixel);
    }
"#;

pub struct OCLTileBuilder {
    context: Context,
    device: Device,
    cqueue: Queue,
    program: Program,
    queue: Arc<Mutex<TileQueue>>,
}

impl OCLTileBuilder {
    pub fn new(queue: Arc<Mutex<TileQueue>>) -> Self {
        let context = Context::builder()
            .devices(Device::specifier().first())
            .build()
            .unwrap();
        let device = context.devices()[0];
        let cqueue = Queue::new(
            &context,
            device,
            Some(CommandQueueProperties::new() /* .out_of_order() */),
        )
        .unwrap();

        let program = Program::builder()
            .src(src)
            .devices(device)
            .build(&context)
            .unwrap();

        OCLTileBuilder {
            context,
            device,
            cqueue,
            program,
            queue,
        }
    }

    fn process(&mut self, p: TileRequest) -> ocl::Result<TileContent> {
        let mut img = vec![0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
        let dims = (TEXTURE_SIZE, TEXTURE_SIZE);

        let dst_image = Image::<u8>::builder()
            .channel_order(ImageChannelOrder::Rgba)
            .channel_data_type(ImageChannelDataType::UnormInt8)
            .image_type(MemObjectType::Image2d)
            .dims(&dims)
            .flags(
                ocl::flags::MEM_WRITE_ONLY
                    | ocl::flags::MEM_HOST_READ_ONLY
                    | ocl::flags::MEM_COPY_HOST_PTR,
            )
            .copy_host_slice(&img)
            .queue(self.cqueue.clone())
            .build()
            .unwrap();

        let kernel = Kernel::builder()
            .program(&self.program)
            .name("add")
            .queue(self.cqueue.clone())
            .global_work_size(&dims)
            .arg(&dst_image)
            .build()
            .unwrap();

        unsafe {
            kernel.enq().unwrap();
        }

        dst_image.read(&mut img).enq().unwrap();

        let t = TileContent {
            pixels: img,
            region: None,
        };
        Ok(t)
    }

    pub fn update(&mut self) {
        let next: Option<TileRequest> = self.queue.lock().unwrap().pop_todo();
        if let Some(p) = next {
            let r = self.process(p).unwrap();
            self.queue.lock().unwrap().push_done(p, r);
        }
    }
}

// fn trivial() -> ocl::Result<()> {
//
// let buffer = pro_que.create_buffer::<f32>()?;
//
// let kernel = pro_que.kernel_builder("add")
// .arg(&buffer)
// .arg(10.0f32)
// .build()?;
//
// unsafe { kernel.enq()?; }
//
// let mut vec = vec![0.0f32; buffer.len()];
// buffer.read(&mut vec).enq()?;
//
// println!("The value at index [{}] is now '{}'!", 200007, vec[200007]);
// Ok(())
// }
