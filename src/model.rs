//! A UserModel that represents a functional parsed model3.json.
use core::{fmt, ops};
use std::{fs, io, path::Path};

use cubism_core::Model;

use crate::{
    controller::Controller,
    effect::EyeBlink,
    error::CubismResult,
    json::model::{GroupTarget, Model3},
};

/// A UserModel that represents a functional parsed model3.json.
pub struct UserModel {
    model: Model,
    eye_blink: Option<EyeBlink>,
}

impl UserModel {
    /// Creates a UserModel from a path of a model3.json file
    #[inline]
    pub fn from_model3_json<P: AsRef<Path>>(path: P) -> CubismResult<Self> {
        let path = path.as_ref();
        let model3 = Model3::from_reader(fs::File::open(path)?)?;
        Self::from_model3(path.parent().unwrap(), &model3)
    }

    /// Creates a UserModel from a Model3 and the parent path of the file.
    pub fn from_model3(base: &Path, model3: &Model3) -> CubismResult<Self> {
        if let Some(moc_path) = model3.file_references.moc.as_ref() {
            let model = Model::from_bytes(&fs::read(base.join(moc_path))?)?;
            let eye_blink = Self::try_create_eye_blink(&model, model3);
            Ok(UserModel { model, eye_blink })
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

    /// Run the controllers and apply them to the model.
    pub fn update(&mut self, delta: f32) {
        if let Some(eb) = self.eye_blink.as_mut() {
            eb.update_parameters(&mut self.model, delta);
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
