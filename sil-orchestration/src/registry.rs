//! Registro de componentes do orquestrador

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use sil_core::prelude::*;
use crate::error::{OrchestrationError, OrchestrationResult};

/// ID único de componente
pub type ComponentId = String;

/// Tipo de componente
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentType {
    Sensor,
    Processor,
    Actuator,
    NetworkNode,
    Governor,
    SwarmAgent,
    QuantumState,
    Forkable,
    Entangled,
    Collapsible,
}

/// Wrapper para componentes com tipo erasure
pub struct ComponentWrapper {
    pub id: ComponentId,
    pub name: String,
    pub component_type: ComponentType,
    pub layers: Vec<LayerId>,
    // Box<dyn Any> para armazenar componente concreto
    #[allow(dead_code)]
    inner: Arc<RwLock<Box<dyn std::any::Any + Send + Sync>>>,
}

impl std::fmt::Debug for ComponentWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentWrapper")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("component_type", &self.component_type)
            .field("layers", &self.layers)
            .finish()
    }
}

/// Registro de componentes
#[derive(Debug)]
pub struct ComponentRegistry {
    /// Componentes por ID
    components: HashMap<ComponentId, ComponentWrapper>,
    /// Índice por tipo
    by_type: HashMap<ComponentType, Vec<ComponentId>>,
    /// Índice por camada
    by_layer: HashMap<LayerId, Vec<ComponentId>>,
    /// Contador para IDs únicos
    next_id: usize,
}

impl ComponentRegistry {
    /// Cria novo registro
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            by_type: HashMap::new(),
            by_layer: HashMap::new(),
            next_id: 0,
        }
    }

    /// Gera ID único
    fn generate_id(&mut self, prefix: &str) -> ComponentId {
        let id = format!("{}-{}", prefix, self.next_id);
        self.next_id += 1;
        id
    }

    /// Registra componente genérico
    pub fn register<C>(&mut self, component: C, comp_type: ComponentType) -> OrchestrationResult<ComponentId>
    where
        C: SilComponent + Send + Sync + 'static,
    {
        let name = component.name().to_string();
        let layers: Vec<LayerId> = component.layers().to_vec();

        let id = self.generate_id(&name);

        // Verificar se componente com mesmo nome já existe
        if self.components.values().any(|w| w.name == name) {
            return Err(OrchestrationError::ComponentAlreadyRegistered(name));
        }

        let wrapper = ComponentWrapper {
            id: id.clone(),
            name,
            component_type: comp_type,
            layers: layers.clone(),
            inner: Arc::new(RwLock::new(Box::new(component))),
        };

        // Indexar por tipo
        self.by_type
            .entry(comp_type)
            .or_insert_with(Vec::new)
            .push(id.clone());

        // Indexar por camadas
        for layer in &layers {
            self.by_layer
                .entry(*layer)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        self.components.insert(id.clone(), wrapper);
        Ok(id)
    }

    /// Remove componente por ID
    pub fn unregister(&mut self, id: &ComponentId) -> OrchestrationResult<()> {
        let wrapper = self.components.remove(id)
            .ok_or_else(|| OrchestrationError::ComponentNotFound(id.clone()))?;

        // Remover dos índices
        if let Some(ids) = self.by_type.get_mut(&wrapper.component_type) {
            ids.retain(|i| i != id);
        }

        for layer in &wrapper.layers {
            if let Some(ids) = self.by_layer.get_mut(layer) {
                ids.retain(|i| i != id);
            }
        }

        Ok(())
    }

    /// Busca componente por ID
    pub fn get(&self, id: &ComponentId) -> Option<&ComponentWrapper> {
        self.components.get(id)
    }

    /// Lista IDs de componentes por tipo
    pub fn list_by_type(&self, comp_type: ComponentType) -> Vec<ComponentId> {
        self.by_type
            .get(&comp_type)
            .cloned()
            .unwrap_or_default()
    }

    /// Lista IDs de componentes por camada
    pub fn list_by_layer(&self, layer: LayerId) -> Vec<ComponentId> {
        self.by_layer
            .get(&layer)
            .cloned()
            .unwrap_or_default()
    }

    /// Lista todos os IDs
    pub fn list_all(&self) -> Vec<ComponentId> {
        self.components.keys().cloned().collect()
    }

    /// Conta componentes
    pub fn count(&self) -> usize {
        self.components.len()
    }

    /// Conta componentes por tipo
    pub fn count_by_type(&self, comp_type: ComponentType) -> usize {
        self.by_type
            .get(&comp_type)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Limpa todos os componentes
    pub fn clear(&mut self) {
        self.components.clear();
        self.by_type.clear();
        self.by_layer.clear();
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock component para testes
    #[derive(Debug)]
    struct MockComponent {
        name: String,
        layers: Vec<LayerId>,
    }

    impl SilComponent for MockComponent {
        fn name(&self) -> &str {
            &self.name
        }

        fn layers(&self) -> &[LayerId] {
            &self.layers
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn is_ready(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_registry_new() {
        let registry = ComponentRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_component() {
        let mut registry = ComponentRegistry::new();
        let component = MockComponent {
            name: "test-sensor".into(),
            layers: vec![0],
        };

        let id = registry.register(component, ComponentType::Sensor);
        assert!(id.is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = ComponentRegistry::new();
        let component1 = MockComponent {
            name: "test-sensor".into(),
            layers: vec![0],
        };
        let component2 = MockComponent {
            name: "test-sensor".into(),
            layers: vec![0],
        };

        registry.register(component1, ComponentType::Sensor).unwrap();
        let result = registry.register(component2, ComponentType::Sensor);
        assert!(result.is_err());
    }

    #[test]
    fn test_unregister_component() {
        let mut registry = ComponentRegistry::new();
        let component = MockComponent {
            name: "test-sensor".into(),
            layers: vec![0],
        };

        let id = registry.register(component, ComponentType::Sensor).unwrap();
        assert_eq!(registry.count(), 1);

        registry.unregister(&id).unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_list_by_type() {
        let mut registry = ComponentRegistry::new();

        let sensor1 = MockComponent {
            name: "sensor1".into(),
            layers: vec![0],
        };
        let sensor2 = MockComponent {
            name: "sensor2".into(),
            layers: vec![1],
        };
        let processor = MockComponent {
            name: "processor1".into(),
            layers: vec![5],
        };

        registry.register(sensor1, ComponentType::Sensor).unwrap();
        registry.register(sensor2, ComponentType::Sensor).unwrap();
        registry.register(processor, ComponentType::Processor).unwrap();

        let sensors = registry.list_by_type(ComponentType::Sensor);
        assert_eq!(sensors.len(), 2);

        let processors = registry.list_by_type(ComponentType::Processor);
        assert_eq!(processors.len(), 1);
    }

    #[test]
    fn test_list_by_layer() {
        let mut registry = ComponentRegistry::new();

        let component1 = MockComponent {
            name: "comp1".into(),
            layers: vec![0, 1],
        };
        let component2 = MockComponent {
            name: "comp2".into(),
            layers: vec![0],
        };

        registry.register(component1, ComponentType::Sensor).unwrap();
        registry.register(component2, ComponentType::Sensor).unwrap();

        let layer0 = registry.list_by_layer(0);
        assert_eq!(layer0.len(), 2);

        let layer1 = registry.list_by_layer(1);
        assert_eq!(layer1.len(), 1);
    }

    #[test]
    fn test_clear_registry() {
        let mut registry = ComponentRegistry::new();
        let component = MockComponent {
            name: "test".into(),
            layers: vec![0],
        };

        registry.register(component, ComponentType::Sensor).unwrap();
        assert_eq!(registry.count(), 1);

        registry.clear();
        assert_eq!(registry.count(), 0);
    }
}
