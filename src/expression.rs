#![deny(missing_docs)]
//! Expression.

use std::{fs, path::Path};

use crate::{
    error::CubismResult,
    json::expression::{Expression3, ExpressionBlendType, ExpressionParameter},
};
use cubism_core::Model;

#[derive(Clone, Debug)]
/// Expression.
pub struct Expression {
    json: Expression3,
}

impl Expression {
    /// Creates an Expression from .exp3.json data.
    pub fn new(exp3: Expression3) -> Self {
        Self { json: exp3 }
    }

    /// Creates a Expression from a path of .exp3.json file.
    pub fn from_exp3_json<P: AsRef<Path>>(path: P) -> CubismResult<Expression> {
        let json = Expression3::from_reader(fs::File::open(path)?)?;
        Ok(Expression::new(json))
    }

    /// Apply an expression to a model.
    pub fn apply(&self, model: &mut Model) -> CubismResult<()> {
        for ExpressionParameter {
            id,
            value,
            blend_type,
        } in &self.json.parameters
        {
            // look up for a parameter by ID
            let param = if let Some(param) = model.parameter_mut(id) {
                param
            } else {
                // skip if missing
                continue;
            };

            let value = *value;
            let orig_value = *param.value;
            let new_value = match blend_type {
                ExpressionBlendType::Add => value + orig_value,
                ExpressionBlendType::Multiply => value * orig_value,
                ExpressionBlendType::Overwrite => value,
            };

            *param.value = new_value.min(param.max_value).max(param.min_value);
        }

        Ok(())
    }
}
