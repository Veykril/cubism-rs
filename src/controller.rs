//! Controller definitions.
use fxhash::FxHashMap;

use std::any::TypeId;

use cubism_core::Model;

mod expression;
pub use self::expression::ExpressionController;
mod eye_blink;
pub use self::eye_blink::EyeBlink;

/// Priorities used by the standard controllers of this crate.
pub mod default_priorities {
    /// The eyeblink controller priority.
    pub const EYE_BLINK: usize = 100;
    /// The eyeblink controller priority.
    pub const EXPRESSION: usize = 200;
}

/// The controller trait. A controller is an object that modifies a models
/// parameter and part values in a defined fashion.
pub trait Controller: 'static {
    /// Run the controller on the passed [`Model`].
    fn update_parameters(&mut self, model: &mut Model, delta: f32);
    /// The execution priority of this controller. The smallest value has the
    /// highest priority.
    fn priority(&self) -> usize;
}

impl dyn Controller {
    unsafe fn downcast_ref_unchecked<C: Controller>(&self) -> &C {
        &*(self as *const Self as *const C)
    }
    unsafe fn downcast_mut_unchecked<C: Controller>(&mut self) -> &mut C {
        &mut *(self as *mut Self as *mut C)
    }
    unsafe fn downcast_unchecked<C: Controller>(self: Box<Self>) -> Box<C> {
        Box::from_raw(Box::into_raw(self) as *mut C)
    }
}

/// A ControllerMap is basically a typemap over [`Controller`]s, it only allows
/// one controller per type to exist and tracks their enabled status.
pub struct ControllerMap {
    map: FxHashMap<TypeId, (Box<dyn Controller>, bool)>,
}

impl ControllerMap {
    /// Creates a new empty controller map.
    pub fn new() -> Self {
        ControllerMap {
            map: FxHashMap::with_hasher(Default::default()),
        }
    }

    /// Registers a new controller, unregistering and returning back the
    /// previous controller of the same type if it exists.
    pub fn register<C: Controller>(&mut self, controller: C) -> Option<Box<C>> {
        let controller: Box<dyn Controller> = Box::new(controller);
        self.map
            .insert(TypeId::of::<C>(), (controller, true))
            .map(|(old, _)| unsafe { old.downcast_unchecked() })
    }

    /// Removes and returns the controller of the type if it exists in the map.
    pub fn remove<C: Controller>(&mut self) -> Option<Box<C>> {
        self.map
            .remove(&TypeId::of::<C>())
            .map(|(old, _)| unsafe { old.downcast_unchecked() })
    }

    /// Returns a reference to the controller of the type if it exists in the
    /// map.
    pub fn get<C: Controller>(&self) -> Option<&C> {
        self.map
            .get(&TypeId::of::<C>())
            .map(|(con, _)| unsafe { (&**con).downcast_ref_unchecked() })
    }

    /// Returns a mutable reference to the controller of the type if it exists
    /// in the map.
    pub fn get_mut<C: Controller>(&mut self) -> Option<&mut C> {
        self.map
            .get_mut(&TypeId::of::<C>())
            .map(|(con, _)| unsafe { (&mut **con).downcast_mut_unchecked() })
    }

    /// Enables or disables the controller of the type. Does nothing if there is
    /// no controller registered under the type.
    pub fn set_enabled<C: Controller>(&mut self, enabled: bool) {
        if let Some((_, en)) = self.map.get_mut(&TypeId::of::<C>()) {
            *en = enabled;
        }
    }

    /// Returns true whether the controller is enabled or not. If it doesn't
    /// exist it returns false.
    pub fn is_enabled<C: Controller>(&self) -> bool {
        self.map
            .get(&TypeId::of::<C>())
            .map(|&(_, enabled)| enabled)
            .unwrap_or(false)
    }

    /// Checks whether the controller type has been registered or not.
    pub fn is_registered<C: Controller>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<C>())
    }

    /// Returns an iterator over the enabled controllers.
    pub fn enabled_controllers<'this>(
        &'this self,
    ) -> impl Iterator<Item = &'this dyn Controller> + 'this {
        self.map
            .values()
            .filter_map(|&(ref con, enabled)| if enabled { Some(&**con) } else { None })
    }

    /// Returns an iterator over the enabled controllers.
    pub fn enabled_controllers_mut<'this>(
        &'this mut self,
    ) -> impl Iterator<Item = &'this mut dyn Controller> + 'this {
        self.map
            .values_mut()
            .filter_map(|&mut (ref mut con, enabled)| if enabled { Some(&mut **con) } else { None })
    }

    /// Returns an iterator over the controllers in this map.
    pub fn controllers<'this>(&'this self) -> impl Iterator<Item = &'this dyn Controller> + 'this {
        self.map.values().map(|(con, _)| &**con)
    }

    /// Returns an iterator over the controllers in this map.
    pub fn controllers_mut<'this>(
        &'this mut self,
    ) -> impl Iterator<Item = &'this mut dyn Controller> + 'this {
        self.map.values_mut().map(|(con, _)| &mut **con)
    }

    /// Calls [`update_parameters`] on every enabled controller in the order of
    /// their priority.
    pub fn update_enabled_controllers(&mut self, model: &mut Model, delta: f32) {
        let mut controllers = self.enabled_controllers_mut().collect::<Vec<_>>();
        controllers.sort_unstable_by_key(|c| c.priority());
        for con in controllers {
            con.update_parameters(model, delta);
        }
    }
}

impl Default for ControllerMap {
    fn default() -> Self {
        Self::new()
    }
}
