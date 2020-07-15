use fractal_toy::IsTileBuilder;
use fractal_toy::TileContent;
use fractal_toy::TileParams;
use fractal_toy::TilePos;
use fractal_toy::TileType;
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
    params: Option<TileParams>,
}

impl IsTileBuilder for OCLWorker {
    fn configure(&mut self, p: &TileParams) -> bool {
        self.params = Some(p.clone());
        // TODO: this is blocking and holding the handle lock, check if that is a problem
        self.program = self.compile();
        self.program.is_some()
    }

    fn build(&mut self, pos: TilePos) -> TileContent {
        self.process(pos)
    }
}

impl OCLWorker {
    // will return Err(_) when unavaliable, or verision mismatch
    pub fn new() -> OCLResult<Self> {
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
            params: None,
        })
    }

    pub fn compile(&mut self) -> Option<Program> {
        let params = self.params.as_ref().unwrap();
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

        match params.kind {
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
            _ => return None,
        }

        let new_src = SOURCE_TEMPLATE
            .replace("@TEXTURE_SIZE@", &format!("{}.0", params.resolution))
            .replace("@ALGORITHM@", &alg)
            .replace("@INC@", inc);

        Some(
            Program::builder()
                .src(new_src)
                .devices(self.device)
                .build(&self.context)
                .unwrap(),
        )
    }

    fn process(&mut self, pos: TilePos) -> TileContent {
        let program = self.program.as_ref().unwrap();

        let params = self.params.as_ref().unwrap();

        let texture_size = params.resolution as usize;

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

        let rect = pos
            .square()
            .grow_relative(params.padding as f64 / params.resolution as f64);

        let kernel = Kernel::builder()
            .program(&program)
            .name("add")
            .queue(self.cqueue.clone())
            .global_work_size(&dims)
            .arg(&dst_image)
            .arg(params.iterations as f32)
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
}
