//! A model expression.

use std::{fs, path::Path};

use crate::error::CubismResult;
use crate::json::expression::{Expression3, ExpressionBlendType, ExpressionParameter};

use cubism_core::Model;

/// A model expression.
#[derive(Clone, Debug)]
pub struct Expression {
    fade_in: f32,
    fade_out: f32,
    parameters: Vec<(usize, ExpressionBlendType, f32)>,
}

impl Expression {
    /// Creates a Expression from a path of .exp3.json file and the
    /// corresponding model.
    pub fn from_exp3_json<P: AsRef<Path>>(model: &Model, path: P) -> CubismResult<Expression> {
        let Expression3 {
            fade_in_time: fade_in,
            fade_out_time: fade_out,
            parameters,
            ..
        } = Expression3::from_reader(fs::File::open(path)?)?;
        Ok(Expression {
            fade_in,
            fade_out,
            parameters: parameters
                .into_iter()
                .flat_map(
                    |ExpressionParameter {
                         id,
                         blend_type,
                         value,
                     }| {
                        model
                            .parameter_ids()
                            .iter()
                            .position(|id2| *id2 == id)
                            .map(|idx| (idx, blend_type, value))
                    },
                )
                .collect::<Vec<_>>(),
        })
    }

    /// Apply an expression to a model.
    pub fn apply(&self, model: &mut Model, mut weight: f32) {
        weight = weight.min(1.0).max(0.0);
        for (id, blend_type, value) in self.parameters.iter().copied() {
            let model_value = &mut model.parameter_values_mut()[id];
            *model_value = match blend_type {
                ExpressionBlendType::Add => value.mul_add(weight, *model_value),
                ExpressionBlendType::Multiply => *model_value * (value - 1.0).mul_add(weight, 1.0),
                ExpressionBlendType::Overwrite => value * weight,
            };
        }
    }
}
