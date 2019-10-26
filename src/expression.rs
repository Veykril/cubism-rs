//! A model expression.

use std::{fs, path::Path};

use crate::{
    error::CubismResult,
    json::expression::{Expression3, ExpressionBlendType, ExpressionParameter},
};
use cubism_core::Model;

/// A model expression.
#[derive(Clone, Debug)]
pub struct Expression {
    parameters: Vec<(usize, ExpressionBlendType, f32)>,
}

impl Expression {
    /// Creates an Expression from a (id, blend_type, value) collection.
    pub fn new<I: Into<Vec<(usize, ExpressionBlendType, f32)>>>(parameters: I) -> Self {
        Self {
            parameters: parameters.into(),
        }
    }

    /// Creates a Expression from a path of .exp3.json file and the
    /// corresponding model.
    pub fn from_exp3_json<P: AsRef<Path>>(model: &Model, path: P) -> CubismResult<Expression> {
        let json = Expression3::from_reader(fs::File::open(path)?)?;
        Ok(Expression::new(
            json.parameters
                .into_iter()
                .flat_map(
                    |ExpressionParameter {
                         id,
                         blend_type,
                         value,
                     }| {
                        Some((
                            model.parameter_ids().iter().position(|id2| *id2 == id)?,
                            blend_type,
                            value,
                        ))
                    },
                )
                .collect::<Vec<_>>(),
        ))
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
