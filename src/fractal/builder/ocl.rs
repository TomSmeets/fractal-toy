use crate::fractal::builder::{TileRequest, TileType};
use crate::fractal::tile::TileContent;
use crate::fractal::TEXTURE_SIZE;
use crossbeam_channel::{Receiver, Sender};
use ocl::enums::{ImageChannelDataType, ImageChannelOrder, MemObjectType};
use ocl::flags::CommandQueueProperties;
use ocl::{Context, Device, Image, Kernel, Program, Queue};
use std::thread;

static SOURCE_TEMPLATE: &str = include_str!("kernel.cl");

pub struct OCLTileBuilder {
    handle: Option<std::thread::JoinHandle<()>>,
}

impl OCLTileBuilder {
    pub fn new(rx: Receiver<TileRequest>, tx: Sender<(TileRequest, TileContent)>) -> Self {
        let handle = thread::spawn(|| {
            let mut w = OCLWorker::new(rx, tx);
            w.run();
        });
        OCLTileBuilder {
            handle: Some(handle),
        }
    }
}

impl Drop for OCLTileBuilder {
    fn drop(&mut self) {
        self.handle.take().unwrap().join().unwrap();
    }
}

pub struct OCLWorker {
    context: Context,
    device: Device,
    cqueue: Queue,
    program: Option<Program>,

    rx: Receiver<TileRequest>,
    tx: Sender<(TileRequest, TileContent)>,

    kind: TileType,
}

impl OCLWorker {
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

    pub fn new(rx: Receiver<TileRequest>, tx: Sender<(TileRequest, TileContent)>) -> Self {
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

        OCLWorker {
            context,
            device,
            cqueue,
            program: None,
            rx,
            tx,
            kind: TileType::Empty,
        }
    }

    fn process(&mut self, p: TileRequest) -> Option<TileContent> {
        if p.params.kind != self.kind || self.program.is_none() {
            self.kind = p.params.kind;
            self.program = Some(self.compile());
        }

        let program = self.program.as_ref().unwrap();

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

        let [offset_x, offset_y, zoom] = p.pos.to_f64_with_padding();

        let kernel = Kernel::builder()
            .program(&program)
            .name("add")
            .queue(self.cqueue.clone())
            .global_work_size(&dims)
            .arg(&dst_image)
            .arg(p.params.iterations as f32)
            .arg(offset_x)
            .arg(offset_y)
            .arg(zoom)
            .build()
            .unwrap();

        unsafe {
            kernel.enq().unwrap();
        }

        dst_image.read(&mut img).enq().unwrap();

        let t = TileContent::new(img);
        Some(t)
    }

    pub fn run(&mut self) {
        while let Ok(next) = self.rx.recv() {
            if let Some(r) = self.process(next) {
                self.tx.send((next, r)).unwrap();
            }
        }
    }
}