use core::fmt;
use std::{fs, io, path::Path};

use cubism_core::Model;

use crate::{error::CubismResult, json::model::Model3};

pub struct UserModel {
    model: Model,
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
            Ok(UserModel { model })
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "no moc file has been specified").into())
        }
    }

    pub fn model(&self) -> &Model {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut Model {
        &mut self.model
    }
}

impl fmt::Debug for UserModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserModel").finish()
    }
}
