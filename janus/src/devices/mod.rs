pub mod amp;
pub mod env;
pub mod filt;
pub mod mixosc;
pub mod modfilt;
pub mod osc;

use super::{fixedmath, EnvParamFxP, NoteFxP, SampleFxP, ScalarFxP, USampleFxP};

use super::BufferT;
use super::STATIC_BUFFER_SIZE;

//TODO: Support multiple sample rates
const SAMPLE_RATE: u16 = 44100;
const FRAC_4096_2PI_SR: fixedmath::U0F32 = fixedmath::U0F32::lit("0x0.9565925d");

pub trait Float: num_traits::Float + num_traits::FloatConst {
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const THREE: Self;
    const ONE_HALF: Self;
    const POINT_NINE_EIGHT: Self;
    const RES_MAX: Self;
    const NOTE_MAX: Self;
}

impl Float for f32 {
    const ZERO: f32 = 0.0f32;
    const ONE: f32 = 1.0f32;
    const TWO: f32 = 2.0f32;
    const THREE: f32 = 3.0f32;
    const ONE_HALF: f32 = 0.5f32;
    const POINT_NINE_EIGHT: f32 = 0.98f32;
    const RES_MAX: f32 = 0xF000 as f32 / 0xFFFF as f32;
    const NOTE_MAX: f32 = 127.0f32 * (0xFFFF as f32 / 0x10000 as f32);
}

impl Float for f64 {
    const ZERO: f64 = 0.0f64;
    const ONE: f64 = 1.0f64;
    const TWO: f64 = 2.0f64;
    const THREE: f64 = 3.0f64;
    const ONE_HALF: f64 = 0.5f64;
    const POINT_NINE_EIGHT: f64 = 0.98f64;
    const RES_MAX: f64 = 0xF000 as f64 / 0xFFFF as f64;
    const NOTE_MAX: f64 = 127.0f64 * (0xFFFF as f64 / 0x10000 as f64);
}

fn midi_note_to_frequency<T: Float>(note: T) -> T {
    let c69 = T::from(69).unwrap();
    let c12 = T::from(12).unwrap();
    let c440 = T::from(440).unwrap();
    c440 * ((note - c69) / c12).exp2()
}

pub use amp::{Amp, AmpFxP};
pub use env::{Env, EnvFxP, EnvParams, EnvParamsFxP};
pub use filt::{Filt, FiltFxP, FiltOutput, FiltOutputFxP, FiltParams, FiltParamsFxP};
pub use mixosc::{MixOsc, MixOscFxP, MixOscParams, MixOscParamsFxP};
pub use modfilt::{ModFilt, ModFiltFxP, ModFiltParams, ModFiltParamsFxP};
pub use osc::{Osc, OscFxP, OscOutput, OscOutputFxP, OscParams, OscParamsFxP};
