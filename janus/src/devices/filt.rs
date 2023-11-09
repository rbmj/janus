use super::*;

/// The output of a [Filt], consisting of low-, band-, and high-pass signals
pub struct FiltOutput<'a, Smp> {
    /// Low-Pass
    pub low: &'a [Smp],
    /// Band-Pass
    pub band: &'a [Smp],
    /// High-Pass
    pub high: &'a [Smp],
}

/// Parameters for a [Filt]
pub struct FiltParams<'a, Smp> {
    /// The cutoff frequency, expressed as a MIDI note number
    pub cutoff: &'a [Smp],
    /// The resonance, expressed as a value between zero and one
    pub resonance: &'a [Smp],
}

impl<'a, Smp> FiltParams<'a, Smp> {
    /// The length of the input parameters, defined as the length of the shortest
    /// input slice.
    pub fn len(&self) -> usize {
        std::cmp::min(self.cutoff.len(), self.resonance.len())
    }
}

/// A 2-pole, floating-point state variable filter, with low, band, and high
/// pass signal outputs.
pub struct Filt<Smp> {
    low: BufferT<Smp>,
    band: BufferT<Smp>,
    high: BufferT<Smp>,
    low_z: Smp,
    band_z: Smp,
}

impl<Smp: Float> Filt<Smp> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            low: [Smp::ZERO; STATIC_BUFFER_SIZE],
            band: [Smp::ZERO; STATIC_BUFFER_SIZE],
            high: [Smp::ZERO; STATIC_BUFFER_SIZE],

            low_z: Smp::ZERO,
            band_z: Smp::ZERO,
        }
    }
    /// Helper function to prewarp the gain of the analog equivalent filter:
    fn prewarped_gain(f: Smp) -> Smp {
        let f_c = midi_note_to_frequency(f);
        Smp::tan(Smp::PI() * f_c / Smp::from(SAMPLE_RATE).unwrap())
    }
    /// Run the filter on the provided input and parameters.
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(&mut self, input: &[Smp], params: FiltParams<Smp>) -> FiltOutput<Smp> {
        let cutoff = params.cutoff;
        let resonance = params.resonance;
        let numsamples =
            std::cmp::min(STATIC_BUFFER_SIZE, std::cmp::min(input.len(), params.len()));
        for i in 0..numsamples {
            let res = Smp::ONE
                - if resonance[i] < Smp::RES_MAX {
                    resonance[i]
                } else {
                    Smp::RES_MAX
                };
            let gain = Self::prewarped_gain(cutoff[i]);
            let denom = gain * gain + Smp::TWO * res * gain + Smp::ONE;
            self.high[i] = (input[i] - (Smp::TWO * res + gain) * self.band_z - self.low_z) / denom;
            let band_gain = gain * self.high[i];
            self.band[i] = band_gain + self.band_z;
            self.band_z = self.band[i] + band_gain;

            let low_gain = gain * self.band[i];
            self.low[i] = low_gain + self.low_z;
            self.low_z = self.low[i] + low_gain;
        }
        FiltOutput {
            low: &self.low[0..numsamples],
            band: &self.band[0..numsamples],
            high: &self.high[0..numsamples],
        }
    }
}

impl<Smp: Float> Default for Filt<Smp> {
    fn default() -> Self {
        Self::new()
    }
}

/// The output of a [FiltFxP], consisting of low-, band-, and high-pass signals.
pub struct FiltOutputFxP<'a> {
    /// Low-Pass
    pub low: &'a [SampleFxP],
    /// Band-Pass
    pub band: &'a [SampleFxP],
    /// High-Pass
    pub high: &'a [SampleFxP],
}

/// Parameters for a [FiltFxP]
pub struct FiltParamsFxP<'a> {
    /// The cutoff frequency of the filter, expressed as a fixed-point MIDI
    /// note number (see [NoteFxP])
    pub cutoff: &'a [NoteFxP],
    /// The resonance of the filter, expressed as a number in `[0, 1)`
    pub resonance: &'a [ScalarFxP],
}

impl<'a> FiltParamsFxP<'a> {
    /// The length of the parameters, defined as the length of the shortest slice.
    pub fn len(&self) -> usize {
        std::cmp::min(self.cutoff.len(), self.resonance.len())
    }
}

/// A 2-pole, fixed-point, state variable filter with low, band, and high pass
/// output signals.
pub struct FiltFxP {
    low: BufferT<SampleFxP>,
    band: BufferT<SampleFxP>,
    high: BufferT<SampleFxP>,
    low_z: fixedmath::I12F20,
    band_z: fixedmath::I12F20,
}

impl FiltFxP {
    const RES_MAX: ScalarFxP = ScalarFxP::lit("0x0.F000");
    /// Constructor
    pub fn new() -> Self {
        Self {
            low: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            band: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            high: [SampleFxP::ZERO; STATIC_BUFFER_SIZE],
            low_z: fixedmath::I12F20::ZERO,
            band_z: fixedmath::I12F20::ZERO,
        }
    }
    /// A helper function to calculate the prewarped gain of the equivalent analog circuit.
    /// Note that the use of [fixedmath::tan_fixed] will cause this to be fairly inaccurate
    /// at high frequencies (approximately half Nyquist, or 11kHz at 44.1kHz sample rate)
    fn prewarped_gain(n: NoteFxP) -> fixedmath::U1F15 {
        let f_c = fixedmath::U14F2::from_num(fixedmath::midi_note_to_frequency(n));
        let omega_d = ScalarFxP::from_num(
            f_c.wide_mul(FRAC_4096_2PI_SR)
                .unwrapped_shr(13),
        );
        fixedmath::tan_fixed(omega_d)
    }
    /// Run the filter on the provided input and parameters.
    ///
    /// Note: The output slice from this function may be shorter than the
    /// input slices.  Callers must check the number of returned samples and
    /// copy them into their own output buffers before calling this function
    /// again to process the remainder of the data.
    pub fn process(&mut self, input: &[SampleFxP], params: FiltParamsFxP) -> FiltOutputFxP {
        let cutoff = params.cutoff;
        let resonance = params.resonance;
        let numsamples = std::cmp::min(
            std::cmp::min(input.len(), cutoff.len()),
            std::cmp::min(resonance.len(), STATIC_BUFFER_SIZE),
        );
        for i in 0..numsamples {
            let res = ScalarFxP::MAX - std::cmp::min(resonance[i], Self::RES_MAX);
            // include type annotations to make the fixed point logic more explicit
            let gain: fixedmath::U1F15 = Self::prewarped_gain(cutoff[i]);
            let gain2 = fixedmath::U3F29::from_num(gain.wide_mul(gain));
            // resonance * gain is a U1F31, so this will only lose the least significant bit
            // and provides space for the shift left below (should be optimized out)
            let gain_r = fixedmath::U3F29::from_num(res.wide_mul(gain));
            let k = gain2 + gain_r.unwrapped_shl(1);
            let (denom_inv, shift) = fixedmath::one_over_one_plus(k);

            let gain_plus_2r =
                fixedmath::U3F29::from_num(res).unwrapped_shl(1) + fixedmath::U3F29::from_num(gain);
            let band_high_feedback: fixedmath::I7F25 = fixedmath::U3F13::from_num(gain_plus_2r)
                .wide_mul_signed(SampleFxP::saturating_from_num(self.band_z));
            let high_num = SampleFxP::saturating_from_num(
                fixedmath::I12F20::from_num(input[i])
                    - fixedmath::I12F20::from_num(band_high_feedback)
                    - self.low_z,
            );
            let high_unshifted: fixedmath::I5F27 = high_num.wide_mul_unsigned(denom_inv);
            self.high[i] = SampleFxP::saturating_from_num(high_unshifted.unwrapped_shr(shift));

            let band_gain = fixedmath::I12F20::from_num(gain.wide_mul_signed(self.high[i]));
            let band = band_gain + self.band_z;
            self.band[i] = SampleFxP::saturating_from_num(band_gain + self.band_z);
            self.band_z = band + band_gain;

            let low_gain = fixedmath::I12F20::from_num(gain.wide_mul_signed(self.band[i]));
            let low = low_gain + self.low_z;
            self.low[i] = SampleFxP::saturating_from_num(low);
            self.low_z = low + low_gain;
        }
        FiltOutputFxP {
            low: &self.low[0..numsamples],
            band: &self.band[0..numsamples],
            high: &self.high[0..numsamples],
        }
    }
}

impl Default for FiltFxP {
    fn default() -> Self {
        Self::new()
    }
}

mod bindings {
    use super::*;

    #[no_mangle]
    pub extern "C" fn janus_filt_u16_new() -> *mut FiltFxP {
        Box::into_raw(Box::new(FiltFxP::new()))
    }

    #[no_mangle]
    pub extern "C" fn janus_filt_u16_free(p: *mut FiltFxP) {
        if !p.is_null() {
            let _ = unsafe { Box::from_raw(p) };
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_filt_u16_process(
        p: *mut FiltFxP,
        samples: u32,
        input: *const i16,
        cutoff: *const u16,
        resonance: *const u16,
        low: *mut *const i16,
        band: *mut *const i16,
        high: *mut *const i16,
        offset: u32,
    ) -> i32 {
        if p.is_null()
            || input.is_null()
            || cutoff.is_null()
            || resonance.is_null()
            || low.is_null()
            || band.is_null()
            || high.is_null()
        {
            return -1;
        }
        unsafe {
            let i = std::slice::from_raw_parts(
                input.offset(offset as isize).cast::<SampleFxP>(),
                samples as usize,
            );
            let c = std::slice::from_raw_parts(
                cutoff.offset(offset as isize).cast::<NoteFxP>(),
                samples as usize,
            );
            let r = std::slice::from_raw_parts(
                resonance.offset(offset as isize).cast::<ScalarFxP>(),
                samples as usize,
            );
            let params = FiltParamsFxP {
                cutoff: c,
                resonance: r,
            };
            let out = (*p).process(i, params);
            *low = out.low.as_ptr().cast();
            *band = out.band.as_ptr().cast();
            *high = out.high.as_ptr().cast();
            out.low.len() as i32
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_filt_f32_new() -> *mut Filt<f32> {
        Box::into_raw(Box::new(Filt::new()))
    }

    #[no_mangle]
    pub extern "C" fn janus_filt_f32_free(p: *mut Filt<f32>) {
        if !p.is_null() {
            let _ = unsafe { Box::from_raw(p) };
        }
    }

    #[no_mangle]
    pub extern "C" fn janus_filt_f32_process(
        p: *mut Filt<f32>,
        samples: u32,
        input: *const f32,
        cutoff: *const f32,
        resonance: *const f32,
        low: *mut *const f32,
        band: *mut *const f32,
        high: *mut *const f32,
        offset: u32,
    ) -> i32 {
        if p.is_null()
            || input.is_null()
            || cutoff.is_null()
            || resonance.is_null()
            || low.is_null()
            || band.is_null()
            || high.is_null()
        {
            return -1;
        }
        unsafe {
            let i = std::slice::from_raw_parts(input.offset(offset as isize), samples as usize);
            let c = std::slice::from_raw_parts(cutoff.offset(offset as isize), samples as usize);
            let r = std::slice::from_raw_parts(resonance.offset(offset as isize), samples as usize);
            let params = FiltParams::<f32> {
                cutoff: c,
                resonance: r,
            };
            let out = (*p).process(i, params);
            *low = out.low.as_ptr().cast();
            *band = out.band.as_ptr().cast();
            *high = out.high.as_ptr().cast();
            out.low.len() as i32
        }
    }
}
