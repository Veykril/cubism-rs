//! A UserModel that represents a functional parsed model3.json.
use std::{fmt, fs, io, ops, path::Path};

use cubism_core::Model;

use crate::controller::{Controller, ControllerMap, ExpressionController, EyeBlink};
use crate::error::CubismResult;
use crate::expression::Expression;
use crate::json::model::{GroupTarget, Model3};

/// A UserModel that represents a functional parsed model3.json.
pub struct UserModel {
    model: Model,
    // registered controllers
    controller_map: ControllerMap,
    // saved snapshot of the models parameter for reloading
    parameter_snapshot: Box<[f32]>,
}

impl UserModel {
    /// Creates a new UserModel backed by the given Model.
    pub fn new(model: Model) -> Self {
        let parameter_snapshot = model.parameter_values().into();
        Self {
            model,
            controller_map: ControllerMap::new(),
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

            let mut expr_con = ExpressionController::new();
            for res in model3.file_references.expressions.iter().map(|exp| {
                Expression::from_exp3_json(&this.model, base.join(&exp.file))
                    .map(|expr| (exp.name.clone(), expr))
            }) {
                let (name, expr) = res?;
                expr_con.register(name, expr);
            }
            this.controller_map.register(expr_con);

            if let Some(eye_blink) = Self::try_create_eye_blink(&this.model, model3) {
                this.controller_map.register(eye_blink);
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

    /// Applies the expression(if set), runs the controllers in order and
    /// updates the model.
    pub fn update(&mut self, delta: f32) {
        self.load_parameters();
        // do motion update here
        self.save_parameters();
        self.controller_map
            .update_enabled_controllers(&mut self.model, delta);
        self.model.update();
    }

    /// The controller map of this model.
    pub fn controllers_map(&self) -> &ControllerMap {
        &self.controller_map
    }

    /// The controller map of this model.
    pub fn controllers_map_mut(&mut self) -> &mut ControllerMap {
        &mut self.controller_map
    }

    /// Returns a reference to the controller of the type if it exists in this
    /// model.
    pub fn controller<C: Controller>(&self) -> Option<&C> {
        self.controller_map.get::<C>()
    }

    /// Returns a mutable reference to the controller of the type if it exists
    /// in this model.
    pub fn controller_mut<C: Controller>(&mut self) -> Option<&mut C> {
        self.controller_map.get_mut::<C>()
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
