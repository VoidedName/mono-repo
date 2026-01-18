use crate::world::World;
use std::collections::HashMap;

pub trait System: 'static {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
    fn run(&mut self, world: &mut World);
}

struct SystemRegistration {
    system: Box<dyn System>,
    enabled: bool,
}

pub struct SystemManager {
    systems: Vec<SystemRegistration>,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn add_system<S: System>(&mut self, system: S) {
        self.systems.push(SystemRegistration {
            system: Box::new(system),
            enabled: true,
        });
    }

    pub fn remove_system_by_type(&mut self, type_id: std::any::TypeId) {
        self.systems.retain(|s| s.system.type_id() != type_id);
    }

    pub fn set_enabled_by_type(&mut self, type_id: std::any::TypeId, enabled: bool) {
        if let Some(sys) = self.systems.iter_mut().find(|s| s.system.type_id() == type_id) {
            sys.enabled = enabled;
        }
    }

    pub fn run(&mut self, world: &mut World) {
        for sys in &mut self.systems {
            if sys.enabled {
                sys.system.run(world);
            }
        }
    }
}
