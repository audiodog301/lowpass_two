#![allow(incomplete_features)]
#![feature(generic_associated_types)]

struct LPF {
    former: f32,
    a: f32,
}

impl LPF {
    fn next_sample(&mut self, input: &f32) -> f32 {
        let out = ((5.0 - self.a) * self.former) + (*input * self.a);
        self.former = *input;
        out
    }

    fn set_a(&mut self, value: &f32) {
        self.a = *value;
    }
}

use serde::{Serialize, Deserialize};

use baseplug::{
    ProcessContext,
    Plugin,
};


baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct LowpassModel {
        #[model(min = 4.0, max = 5.0)]
        #[parameter(name = "cutoff", gradient = "Linear")]
        cutoff: f32
    }
}

impl Default for LowpassModel {
    fn default() -> Self {
        Self {
            cutoff: 1.0
        }
    }
}

struct Lowpass {
    leftDistortion: LPF,
    rightDistortion: LPF,
}

impl Plugin for Lowpass {
    const NAME: &'static str = "ruin";
    const PRODUCT: &'static str = "ruin";
    const VENDOR: &'static str = "audiodog301";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = LowpassModel;

    #[inline]
    fn new(_sample_rate: f32, _model: &LowpassModel) -> Self {
        Self {
            leftDistortion: LPF { former: 0.0, a: 4.0},
            rightDistortion: LPF { former: 0.0, a: 4.0},
        }
    }

    #[inline]
    fn process(&mut self, model: &LowpassModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        for i in 0..ctx.nframes {
            if model.cutoff.is_smoothing() {
                self.leftDistortion.set_a(&(model.cutoff[i]));
                self.rightDistortion.set_a(&(model.cutoff[i]));
            }
            
            output[0][i] = self.leftDistortion.next_sample(&(input[0][i]));
            output[1][i] = self.leftDistortion.next_sample(&(input[1][i]));
        }
    }
}

baseplug::vst2!(Lowpass, b"lpft");