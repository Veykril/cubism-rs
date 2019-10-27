use cubism_core::Model;

use crate::controller::Controller;

#[derive(Copy, Clone, Debug)]
enum EyeState {
    Open,
    Closed,
    Closing,
    Opening,
}

/// An Eye Blink controller. This Controller emulates eye blinking.
// FIXME: sanitize timing inputs
#[derive(Clone, Debug)]
pub struct EyeBlink {
    parameter_ids: Box<[usize]>,
    current_state: EyeState,
    next_cycle: f32,
    blink_interval: f32,
    closed_time: f32,
    opening_time: f32,
    closing_time: f32,
}

impl Default for EyeBlink {
    fn default() -> Self {
        EyeBlink {
            parameter_ids: Box::new([]),
            current_state: EyeState::Open,
            next_cycle: 5.0,
            blink_interval: 5.0,
            closed_time: 0.05,
            opening_time: 0.15,
            closing_time: 0.1,
        }
    }
}

impl EyeBlink {
    /// Creates a new EyeBlink Controller acting on the specified parameter ids
    /// with the given timings.
    ///
    /// The controller assumes that the ids belong to the model that is being
    /// passed on to [`EyeBlink::update_parameters`], meaning that if this
    /// is not the case the application may panic on out of bounds access or
    /// move incorrect parts.
    pub fn new<B: Into<Box<[usize]>>>(
        parameter_ids: B,
        blink_interval: f32,
        closed_time: f32,
        opening_time: f32,
        closing_time: f32,
    ) -> Self {
        EyeBlink {
            parameter_ids: parameter_ids.into(),
            current_state: EyeState::Open,
            blink_interval,
            next_cycle: blink_interval,
            closed_time,
            opening_time,
            closing_time,
        }
    }

    /// Set the parameters that are affected by this controller.
    pub fn set_ids<B: Into<Box<[usize]>>>(&mut self, parameter_ids: B) {
        self.parameter_ids = parameter_ids.into();
    }

    /// Set the timings of this controller.
    pub fn set_timings(
        &mut self,
        blink_interval: f32,
        closed_time: f32,
        opening_time: f32,
        closing_time: f32,
    ) {
        self.blink_interval = blink_interval.max(closed_time + opening_time + closing_time);
        self.next_cycle = self.blink_interval;
        self.closed_time = closed_time;
        self.opening_time = opening_time;
        self.closing_time = closing_time;
    }
}

impl Controller for EyeBlink {
    fn update_parameters(&mut self, model: &mut Model, delta: f32) {
        self.next_cycle -= delta;
        let val = match self.current_state {
            EyeState::Open => {
                if self.next_cycle <= 0.0 {
                    self.current_state = EyeState::Closing;
                    self.next_cycle += self.closing_time;
                }
                1.0
            },
            EyeState::Closed => {
                if self.next_cycle <= 0.0 {
                    self.current_state = EyeState::Opening;
                    self.next_cycle += self.opening_time;
                }
                0.0
            },
            EyeState::Opening => {
                if self.next_cycle <= 0.0 {
                    self.current_state = EyeState::Open;
                    self.next_cycle += self.blink_interval;
                    1.0
                } else {
                    (self.opening_time - self.next_cycle) / self.opening_time
                }
            },
            EyeState::Closing => {
                if self.next_cycle <= 0.0 {
                    self.current_state = EyeState::Closed;
                    self.next_cycle += self.closed_time;
                    0.0
                } else {
                    self.next_cycle / self.closing_time
                }
            },
        };
        for par in self.parameter_ids.iter().copied() {
            model.parameter_values_mut()[par] = val;
        }
    }

    fn priority(&self) -> usize {
        crate::controller::default_priorities::EYE_BLINK
    }
}
