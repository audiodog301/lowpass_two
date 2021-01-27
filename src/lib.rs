#![allow(incomplete_features)]
#![feature(generic_associated_types)]

struct LPF {
    former: f32,
    a: f32,
    former_magnitude: f32,
}

impl LPF {
    fn next_sample(&mut self, input: &f32) -> f32 {
        let out = ((self.former_magnitude - self.a) * self.former) + (*input * self.a);
        self.former = out;
        out
    }

    fn set_a(&mut self, value: &f32) {
        self.a = *value;
    }
}

fn clip(input: &f32, clip_at: &f32) -> f32 {
    if *input > *clip_at {
        *clip_at
    } else if *input < (-1.0 * *clip_at) {
        -1.0 * *clip_at
    } else {
        *input
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
        cutoff: f32,
        #[model(min = 0.0, max = 1.0)]
        #[parameter(name = "cutoff_two", gradient = "Linear")]
        cutoff_two: f32
    }
}

impl Default for LowpassModel {
    fn default() -> Self {
        Self {
            cutoff: 5.0,
            cutoff_two: 1.0
        }
    }
}

struct Lowpass {
    left_distortion: LPF,
    right_distortion: LPF,
    left_filter: LPF,
    right_filter: LPF,
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
            left_distortion: LPF { former: 0.0, a: 4.0, former_magnitude: 5.0 },
            right_distortion: LPF { former: 0.0, a: 4.0, former_magnitude: 5.0 },
            left_filter: LPF {former: 0.0, a: 1.0, former_magnitude: 1.0 },
            right_filter: LPF { former: 0.0, a: 1.0, former_magnitude: 1.0 }
        }
    }

    #[inline]
    fn process(&mut self, model: &LowpassModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        for i in 0..ctx.nframes {
            if model.cutoff.is_smoothing() {
                self.left_distortion.set_a(&(model.cutoff[i]));
                self.right_distortion.set_a(&(model.cutoff[i]));
            }

            if model.cutoff_two.is_smoothing() {
                self.left_filter.set_a(&(model.cutoff_two[i]));
                self.right_filter.set_a(&(model.cutoff_two[i]));
            }
            
            output[0][i] = self.left_filter.next_sample(&(0.5*clip(&(self.left_distortion.next_sample(&(input[0][i]))), &1.7)));
            output[1][i] = self.right_filter.next_sample(&(0.5*clip(&(self.right_distortion.next_sample(&(input[1][i]))), &1.7)));
        }
    }
}

baseplug::vst2!(Lowpass, b"lpft");