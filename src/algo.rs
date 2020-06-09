type V2 = cgmath::Vector2<f64>;

pub trait FractalAlg {
    fn iterations_per_step(&self) -> f64 { 1.0 }

    fn steps(&self) -> GenericAlg;
    fn step_cpu(&self, z: V2, c: V2) -> V2; 
}

#[derive(Clone, Copy)]
pub enum AlgStep {
    Pow2, // z*z   +  c
    Pow3, // z*z*z +  c
    Abs,  // |re z| - i|im z|
}

#[derive(Clone)]
pub struct GenericAlg {
    pub steps: Vec<AlgStep>,
}

pub struct Debug { }
impl FractalAlg for Debug {
    fn steps(&self) -> GenericAlg { GenericAlg { steps: Vec::new() } }
    fn iterations_per_step(&self) -> f64 { 1.0 }
    fn step_cpu(&self, z: V2, c: V2) -> V2 {
        z + c
    }
}

impl FractalAlg for AlgStep {
    fn iterations_per_step(&self) -> f64 {
        match self {
            AlgStep::Pow2 => 1.0,
            AlgStep::Pow3 => 1.5,
            AlgStep::Abs  => 0.0,
        }
    }

    fn steps(&self) -> GenericAlg {
        GenericAlg { steps: vec![ *self ] }
    }

    fn step_cpu(&self, z: V2, c: V2) -> V2 {
        match self {
            Self::Pow2 => cpx_sqr(z) + c,
            Self::Pow3 => cpx_cube(z) + c,
            Self::Abs  => cpx_abs(z),
        }
    }
}

impl FractalAlg for GenericAlg {
    fn iterations_per_step(&self) -> f64 {
        self.steps.iter().map(AlgStep::iterations_per_step).sum::<f64>().max(1.0)
    }

    fn steps(&self) -> GenericAlg {
        self.clone()
    }

    fn step_cpu(&self, mut z: V2, c: V2) -> V2 {
        for s in self.steps.iter() {
            z = s.step_cpu(z, c);
        }
        z
    }
}

pub struct Mandelbrot { }
impl FractalAlg for Mandelbrot {
    fn steps(&self) -> GenericAlg {
        GenericAlg { steps : vec! [ AlgStep::Pow2 ] }
    }

    fn step_cpu(&self, mut z: V2, c: V2) -> V2 {
        z = cpx_sqr(z) + c;
        z
    }
}

pub struct BurningShip { }
impl FractalAlg for BurningShip {
    fn steps(&self) -> GenericAlg {
        GenericAlg { steps : vec! [ AlgStep::Abs, AlgStep::Pow2 ] }
    }

    fn step_cpu(&self, mut z: V2, c: V2) -> V2 {
        z = cpx_abs(z);
        z = cpx_sqr(z) + c;
        z
    }
}

pub struct ShipHybrid { }
impl FractalAlg for ShipHybrid {
    fn steps(&self) -> GenericAlg {
        GenericAlg { steps : vec! [ AlgStep::Pow3, AlgStep::Abs, AlgStep::Pow2 ] }
    }

    fn iterations_per_step(&self) -> f64 { 2.5 }

    fn step_cpu(&self, mut z: V2, c: V2) -> V2 {
        z = cpx_cube(z) + c; // 1.5
        z = cpx_abs(z);
        z = cpx_sqr(z) + c; // 1.0
        z
    }
}

fn cpx_mul(a: V2, b: V2) -> V2 {
    V2 {
        x: a.x * b.x - a.y * b.y,
        y: a.x * b.y + a.y * b.x,
    }
}

fn cpx_cube(a: V2) -> V2 {
    cpx_mul(cpx_sqr(a), a)
}

fn cpx_sqr(a: V2) -> V2 {
    V2 {
        x: a.x * a.x - a.y * a.y,
        y: 2.0 * a.x * a.y,
    }
}

fn cpx_abs(a: V2) -> V2 {
    V2 {
        x: a.x.abs(),
        y: -a.y.abs(),
    }
}

