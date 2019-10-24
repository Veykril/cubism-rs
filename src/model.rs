//! A UserModel that represents a functional parsed model3.json.
use fixedbitset::FixedBitSet;

use std::{collections::HashMap, fmt, fs, io, ops, path::Path};

use cubism_core::Model;

use crate::{
    controller::Controller,
    effect::EyeBlink,
    error::CubismResult,
    expression::Expression,
    json::model::{GroupTarget, Model3},
};

/// A UserModel that represents a functional parsed model3.json.
pub struct UserModel {
    model: Model,
    // registered controllers
    controllers: Vec<Box<dyn Controller>>,
    // mapping from name to index into the controllers array
    controllers_rev_map: HashMap<String, usize>,
    controllers_enabled: FixedBitSet,
    // loaded expressions
    expressions: HashMap<String, Expression>,
    current_expression: Option<String>,
    expression_weight: f32,
    // saved snapshot of the models parameter for reloading
    parameter_snapshot: Box<[f32]>,
}

impl UserModel {
    /// Creates a new UserModel backed by the given Model.
    pub fn new(model: Model) -> Self {
        let parameter_snapshot = model.parameter_values().into();
        Self {
            model,
            controllers: Vec::new(),
            controllers_rev_map: HashMap::new(),
            controllers_enabled: FixedBitSet::with_capacity(0),
            expressions: HashMap::new(),
            current_expression: None,
            expression_weight: 1.0,
            parameter_snapshot,
        }
    }

    /// Creates a UserModel from a path of a model3.json file
    #[inline]
    pub fn from_model3_json<P: AsRef<Path>>(path: P) -> CubismResult<Self> {
        let path = path.as_ref();
        let model3 = Model3::from_reader(fs::File::open(path)?)?;
        Self::from_model3(path.parent().unwrap(), &model3)
    }

    /// Creates a UserModel from a Model3 and the parent path of the file it was
    /// loaded from.
    pub fn from_model3(base: &Path, model3: &Model3) -> CubismResult<Self> {
        if let Some(moc_path) = model3.file_references.moc.as_ref() {
            let model = Model::from_bytes(&fs::read(base.join(moc_path))?)?;
            let mut this = Self::new(model);

            this.expressions = model3
                .file_references
                .expressions
                .iter()
                .map(|exp| {
                    Ok((
                        exp.name.clone(),
                        Expression::from_exp3_json(&this.model, base.join(&exp.file))?,
                    ))
                })
                .collect::<CubismResult<_>>()?;

            if let Some(eye_blink) = Self::try_create_eye_blink(&this.model, model3) {
                this.add_controller(crate::id::groups::EYE_BLINK.to_owned(), eye_blink);
            }

            Ok(this)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "no moc file has been specified").into())
        }
    }

    fn try_create_eye_blink(model: &Model, model3: &Model3) -> Option<EyeBlink> {
        let eye_blink_ids: Box<[usize]> = model3
            .groups
            .iter()
            .find(|g| g.target == GroupTarget::Parameter && g.name == "EyeBlink")?
            .ids
            .iter()
            .flat_map(|id| model.parameter_ids().iter().position(|id2| *id2 == *id))
            .collect();

        let mut eb = EyeBlink::default();
        eb.set_ids(eye_blink_ids);
        Some(eb)
    }

    /// Saves the current parameter values of this model in a hidden snapshot.
    pub fn save_parameters(&mut self) {
        self.parameter_snapshot
            .copy_from_slice(self.model.parameter_values());
    }

    /// Loads the parameters of the hidden snapshot into the current parameter
    /// values of this model.
    pub fn load_parameters(&mut self) {
        self.model
            .parameter_values_mut()
            .copy_from_slice(&self.parameter_snapshot);
    }

    /// Swaps the parameters of this model and the hidden snapshot.
    pub fn swap_parameters(&mut self) {
        self.parameter_snapshot
            .swap_with_slice(self.model.parameter_values_mut());
    }

    /// Adds a new controller to this model with the given name. Returns the
    /// slot it has been moved into that determines the execution order.
    /// The controller is toggled off by default
    pub fn add_controller<S: Into<String>, C: Controller + 'static>(
        &mut self,
        name: S,
        controller: C,
    ) -> usize {
        let idx = self.controllers.len();
        self.controllers.push(Box::new(controller));
        self.controllers_rev_map.insert(name.into(), idx);
        self.controllers_enabled.grow(idx + 1);
        idx
    }

    /// Enables the controller at the specified index.
    pub fn controller_enable(&mut self, idx: usize, enabled: bool) {
        if idx < self.controllers.len() {
            self.controllers_enabled.set(idx, enabled);
        }
    }

    /// Loads an expression from a json file at the given path and adds it with
    /// the given name. If an expression with the given name already exists it
    /// will be replaced by the new one, returning the previous expression.
    pub fn load_expressions<S: Into<String>, P: AsRef<Path>>(
        &mut self,
        name: S,
        path: P,
    ) -> CubismResult<Option<Expression>> {
        Expression::from_exp3_json(&self.model, path)
            .map(|exp| self.expressions.insert(name.into(), exp))
    }

    /// Set the current expression, if an expression by the given name doesnt
    /// exist it will be set to apply no expression.
    pub fn set_expression<S: Into<String>>(&mut self, exp: S) {
        let exp = exp.into();
        self.current_expression = if self.expressions.contains_key(&exp) {
            Some(exp)
        } else {
            None
        };
    }

    /// Sets the expression weight to apply.
    /// Note: Weight will be bound between [0.0,1.0].
    pub fn set_expression_weight(&mut self, weight: f32) {
        self.expression_weight = weight.min(1.0).max(0.0);
    }

    /// The currently loaded expressions.
    pub fn expressions(&self) -> &HashMap<String, Expression> {
        &self.expressions
    }

    /// Applies the expression(if set), runs the controllers in order and
    /// updates the model.
    pub fn update(&mut self, delta: f32) {
        self.load_parameters();
        // do motion update here
        self.save_parameters();
        if let Some(exp) = self.current_expression.as_ref() {
            if let Some(exp) = self.expressions.get(exp) {
                exp.apply(&mut self.model, self.expression_weight);
            }
        }
        for con in self.controllers_enabled.ones() {
            self.controllers[con].update_parameters(&mut self.model, delta);
        }
        self.model.update();
    }

    /// The underlying core model.
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// The underlying core model.
    pub fn model_mut(&mut self) -> &mut Model {
        &mut self.model
    }
}

impl ops::Deref for UserModel {
    type Target = Model;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl fmt::Debug for UserModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserModel").finish()
    }
}
