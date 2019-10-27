use fxhash::FxHashMap;

use cubism_core::Model;

use crate::controller::Controller;
use crate::expression::Expression;
use crate::util::SimpleSlab;

/// An ExpressionController is responsible for properly registering and
/// switching between expressions of a model.
pub struct ExpressionController {
    expressions: SimpleSlab<Expression>,
    name_map: FxHashMap<String, usize>,
    current_expr: Option<usize>,
    weight: f32,
}

impl ExpressionController {
    /// Creates a new empty ExpressionController.
    pub fn new() -> Self {
        Self {
            expressions: SimpleSlab::new(),
            name_map: FxHashMap::default(),
            current_expr: None,
            weight: 1.0,
        }
    }

    /// Inserts a new expression into the map with the name, unregistering
    /// and returning back the previous expression under the same name if it
    /// exists.
    pub fn register(&mut self, name: impl Into<String>, exp: Expression) -> Option<Expression> {
        let index = self.expressions.push(exp);
        self.name_map
            .insert(name.into(), index)
            .and_then(|old| self.expressions.take(old))
    }

    /// Set the current expression, if an expression by the given name doesnt
    /// exist it will be set to apply no expression.
    pub fn set_expression(&mut self, name: &str) {
        self.current_expr = self.name_map.get(name).copied();
    }

    /// Sets the expression weight to apply.
    /// Note: Weight will be bound between [0.0,1.0].
    pub fn set_expression_weight(&mut self, weight: f32) {
        self.weight = weight.min(1.0).max(0.0);
    }

    /// The names of all currently registered expressions.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.name_map.keys().map(|s| &**s)
    }

    /// An iterator over the currently registered expressions
    pub fn expressions(&self) -> impl Iterator<Item = &Expression> {
        self.expressions.iter().flatten()
    }
}

impl Controller for ExpressionController {
    fn update_parameters(&mut self, model: &mut Model, _: f32) {
        self.current_expr.map(|expr| {
            self.expressions
                .get(expr)
                .map(|expr| expr.apply(model, self.weight))
        });
    }

    fn priority(&self) -> usize {
        crate::controller::default_priorities::EXPRESSION
    }
}

impl Default for ExpressionController {
    fn default() -> Self {
        Self::new()
    }
}
