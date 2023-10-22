mod fixedmath;
pub mod util;

pub mod devices;

pub mod voice;
pub use voice::VoiceFxP;

const STATIC_BUFFER_SIZE : usize = 256;
type BufferT<T> = [T; STATIC_BUFFER_SIZE];

pub use fixedmath::Sample as SampleFxP;
pub use fixedmath::USample as USampleFxP;
pub use fixedmath::Note as NoteFxP;
pub use fixedmath::Scalar as ScalarFxP;
pub use fixedmath::U3F13 as EnvParamFxP;

