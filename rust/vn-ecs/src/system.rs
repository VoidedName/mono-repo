use std::collections::HashMap;
use crate::world::World;

pub trait System: 'static {
    fn name(&self) -> String {
        format!(
            "{:?}::{}",
            std::any::TypeId::of::<Self>(),
            std::any::type_name::<Self>()
        )
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

    pub fn remove_system_by_name(&mut self, name: &str) {
        self.systems.retain(|s| s.system.name() != name);
    }

    pub fn set_enabled_by_name(&mut self, name: &str, enabled: bool) {
        if let Some(sys) = self.systems.iter_mut().find(|s| s.system.name() == name) {
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
