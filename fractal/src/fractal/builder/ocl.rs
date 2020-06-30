use crate::fractal::builder::{TileRequest, TileType};
use crate::fractal::queue::QueueHandle;
use crate::fractal::queue::TileResponse;
use crate::fractal::TileContent;
use ocl::enums::{ImageChannelDataType, ImageChannelOrder, MemObjectType};
use ocl::flags::CommandQueueProperties;
use ocl::Result as OCLResult;
use ocl::{Context, Device, Image, Kernel, Program, Queue};

static SOURCE_TEMPLATE: &str = include_str!("kernel.cl");

pub struct OCLWorker {
    context: Context,
    device: Device,
    cqueue: Queue,
    program: Option<Program>,

    handle: QueueHandle,

    kind: TileType,
}

impl OCLWorker {
    // will return Err(_) when unavaliable, or verision mismatch
    pub fn new(handle: QueueHandle) -> OCLResult<Self> {
        let context = Context::builder()
            .devices(Device::specifier().first())
            .build()?;
        let device = context.devices()[0];
        let cqueue = Queue::new(
            &context,
            device,
            Some(CommandQueueProperties::new() /* .out_of_order() */),
        )?;

        Ok(OCLWorker {
            context,
            device,
            cqueue,
            program: None,
            handle,
            kind: TileType::Empty,
        })
    }

    pub fn compile(&mut self) -> Program {
        let pow2 = r#"
            tmp = z;
            z.x = tmp.x*tmp.x - tmp.y*tmp.y + c.x;
            z.y = 2.0*tmp.x*tmp.y + c.y;
        "#;

        let pow3 = r#"
            tmp = z;
            z.x = tmp.x*tmp.x*tmp.x - tmp.y*tmp.y*tmp.x - 2*tmp.x*tmp.y*tmp.y + c.x;
            z.y = 2.0*tmp.x*tmp.x*tmp.y + tmp.x*tmp.x*tmp.y - tmp.y*tmp.y*tmp.y + c.y;
        "#;

        let abs = r#"
            z = fabs(z);
            z.y = -z.y;
        "#;

        let mut alg = String::new();
        let mut inc = "1.0";

        match self.kind {
            TileType::Mandelbrot => {
                alg.push_str(pow2);
            },
            TileType::ShipHybrid => {
                alg.push_str(pow3);
                alg.push_str(abs);
                alg.push_str(pow2);
                inc = "2.5";
            },
            TileType::BurningShip => {
                alg.push_str(abs);
                alg.push_str(pow2);
            },
            TileType::Empty => {},
        }

        let new_src = SOURCE_TEMPLATE
            .replace(
                "@TEXTURE_SIZE@",
                &format!("{}.0", super::super::TEXTURE_SIZE),
            )
            .replace("@ALGORITHM@", &alg)
            .replace("@INC@", inc);

        Program::builder()
            .src(new_src)
            .devices(self.device)
            .build(&self.context)
            .unwrap()
    }

    fn process(&mut self, p: &TileRequest) -> TileContent {
        if p.params.kind != self.kind || self.program.is_none() {
            self.kind = p.params.kind;
            self.program = Some(self.compile());
        }

        let program = self.program.as_ref().unwrap();

        let texture_size = p.params.resolution as usize;

        let mut img = vec![0; texture_size * texture_size * 4];
        let dims = (texture_size, texture_size);

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

        let rect = p
            .pos
            .square()
            .grow_relative(p.params.padding as f64 / p.params.resolution as f64);

        let kernel = Kernel::builder()
            .program(&program)
            .name("add")
            .queue(self.cqueue.clone())
            .global_work_size(&dims)
            .arg(&dst_image)
            .arg(p.params.iterations as f32)
            .arg(rect.x)
            .arg(rect.y)
            .arg(rect.w)
            .build()
            .unwrap();

        unsafe {
            kernel.enq().unwrap();
        }

        dst_image.read(&mut img).enq().unwrap();

        TileContent::new(img)
    }

    pub fn run(&mut self) {
        loop {
            let h = match self.handle.tiles.upgrade() {
                Some(h) => h,
                None => break,
            };

            let mut h = h.lock();

            if h.params.kind == TileType::Empty {
                drop(h);
                self.handle.wait();
                continue;
            }

            let next = match h.recv() {
                None => {
                    drop(h);
                    self.handle.wait();
                    continue;
                },
                Some(next) => next,
            };

            let next = TileRequest {
                params: h.params.clone(),
                version: h.params_version,
                pos: next,
            };

            // make sure the lock is freed
            drop(h);

            let tile = self.process(&next);

            let ret = self.handle.send(TileResponse {
                pos: next.pos,
                version: next.version,
                content: tile,
            });

            if ret.is_err() {
                break;
            }
        }
    }
}
