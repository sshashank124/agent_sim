#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Agent {
    position: [f32; 2],
    heading: f32, // radians
    _pad0: u32,
}

impl Agent {
    pub fn new(mut rand_unit: impl FnMut() -> f32) -> Self {
        let mut rand_signed_unit = || rand_unit() * 2. - 1.;
        Self {
            position: core::array::from_fn(|_| rand_signed_unit()),
            heading: rand_signed_unit() * core::f32::consts::PI,
            ..Default::default()
        }
    }
}
