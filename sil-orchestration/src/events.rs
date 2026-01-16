//! Sistema de eventos do orquestrador

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sil_core::prelude::*;
use crate::error::OrchestrationResult;

/// Handler de eventos (callback)
pub type EventHandler = Arc<dyn Fn(&SilEvent) + Send + Sync>;

/// Filtro de eventos
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventFilter {
    /// Todos os eventos
    All,
    /// Eventos de uma camada específica
    Layer(LayerId),
    /// Eventos de um range de camadas
    LayerRange(LayerId, LayerId),
    /// Eventos de mudança de estado
    StateChange,
    /// Eventos de threshold
    Threshold,
    /// Eventos de erro
    Error,
    /// Eventos de um componente específico
    Source(String),
    /// Eventos de medição de energia
    Energy,
    /// Alertas de energia (threshold/power)
    EnergyAlert,
}

impl EventFilter {
    /// Verifica se um evento passa pelo filtro
    pub fn matches(&self, event: &SilEvent) -> bool {
        match (self, event) {
            (EventFilter::All, _) => true,
            (EventFilter::Layer(layer), SilEvent::StateChange { layer: event_layer, .. })
                => layer == event_layer,
            (EventFilter::Layer(layer), SilEvent::Threshold { layer: event_layer, .. })
                => layer == event_layer,
            (EventFilter::LayerRange(start, end), SilEvent::StateChange { layer, .. })
                => layer >= start && layer <= end,
            (EventFilter::LayerRange(start, end), SilEvent::Threshold { layer, .. })
                => layer >= start && layer <= end,
            (EventFilter::StateChange, SilEvent::StateChange { .. }) => true,
            (EventFilter::Threshold, SilEvent::Threshold { .. }) => true,
            (EventFilter::Error, SilEvent::Error { .. }) => true,
            (EventFilter::Source(src), SilEvent::Error { component, .. })
                => component == src,
            (EventFilter::Source(src), SilEvent::EnergyMeasurement { source, .. })
                => source == src,
            (EventFilter::Source(src), SilEvent::PowerAlert { source, .. })
                => source == src,
            // Filtros de energia
            (EventFilter::Energy, SilEvent::EnergyMeasurement { .. }) => true,
            (EventFilter::EnergyAlert, SilEvent::EnergyThreshold { .. }) => true,
            (EventFilter::EnergyAlert, SilEvent::PowerAlert { .. }) => true,
            _ => false,
        }
    }
}

/// Bus de eventos
#[derive(Clone)]
pub struct EventBus {
    /// Handlers registrados por filtro
    handlers: Arc<Mutex<HashMap<EventFilter, Vec<EventHandler>>>>,
    /// Histórico de eventos (opcional, limitado)
    history: Arc<Mutex<Vec<SilEvent>>>,
    /// Tamanho máximo do histórico
    max_history: usize,
}

impl EventBus {
    /// Cria novo bus de eventos
    pub fn new() -> Self {
        Self::with_history(100)
    }

    /// Cria com tamanho de histórico customizado
    pub fn with_history(max_history: usize) -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(Mutex::new(Vec::new())),
            max_history,
        }
    }

    /// Registra handler para um filtro
    pub fn subscribe<F>(&self, filter: EventFilter, handler: F) -> OrchestrationResult<()>
    where
        F: Fn(&SilEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock()?;
        handlers
            .entry(filter)
            .or_insert_with(Vec::new)
            .push(Arc::new(handler));
        Ok(())
    }

    /// Remove todos os handlers de um filtro
    pub fn unsubscribe(&self, filter: &EventFilter) -> OrchestrationResult<()> {
        let mut handlers = self.handlers.lock()?;
        handlers.remove(filter);
        Ok(())
    }

    /// Emite um evento
    pub fn emit(&self, event: SilEvent) -> OrchestrationResult<()> {
        // Adiciona ao histórico
        {
            let mut history = self.history.lock()?;
            history.push(event.clone());

            // Limita tamanho do histórico
            if history.len() > self.max_history {
                history.remove(0);
            }
        }

        // Dispara handlers
        let handlers = self.handlers.lock()?;
        for (filter, handler_list) in handlers.iter() {
            if filter.matches(&event) {
                for handler in handler_list {
                    handler(&event);
                }
            }
        }

        Ok(())
    }

    /// Retorna histórico de eventos
    pub fn history(&self) -> OrchestrationResult<Vec<SilEvent>> {
        let history = self.history.lock()?;
        Ok(history.clone())
    }

    /// Limpa histórico
    pub fn clear_history(&self) -> OrchestrationResult<()> {
        let mut history = self.history.lock()?;
        history.clear();
        Ok(())
    }

    /// Conta handlers registrados
    pub fn handler_count(&self) -> OrchestrationResult<usize> {
        let handlers = self.handlers.lock()?;
        Ok(handlers.values().map(|v| v.len()).sum())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("max_history", &self.max_history)
            .field("history_len", &self.history.lock().map(|h| h.len()).unwrap_or(0))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_bus_new() {
        let bus = EventBus::new();
        assert_eq!(bus.handler_count().unwrap(), 0);
    }

    #[test]
    fn test_subscribe_handler() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        bus.subscribe(EventFilter::All, move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }).unwrap();

        assert_eq!(bus.handler_count().unwrap(), 1);
    }

    #[test]
    fn test_emit_event() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        bus.subscribe(EventFilter::All, move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }).unwrap();

        let event = SilEvent::StateChange {
            layer: 0,
            old: ByteSil::NULL,
            new: ByteSil::ONE,
            timestamp: 0,
        };

        bus.emit(event).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_filter_layer() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        bus.subscribe(EventFilter::Layer(0), move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }).unwrap();

        // Evento na camada 0 - deve disparar
        let event1 = SilEvent::StateChange {
            layer: 0,
            old: ByteSil::NULL,
            new: ByteSil::ONE,
            timestamp: 0,
        };
        bus.emit(event1).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // Evento na camada 1 - não deve disparar
        let event2 = SilEvent::StateChange {
            layer: 1,
            old: ByteSil::NULL,
            new: ByteSil::ONE,
            timestamp: 0,
        };
        bus.emit(event2).unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_filter_layer_range() {
        let filter = EventFilter::LayerRange(0, 4);

        let event0 = SilEvent::StateChange {
            layer: 0,
            old: ByteSil::NULL,
            new: ByteSil::ONE,
            timestamp: 0,
        };
        assert!(filter.matches(&event0));

        let event4 = SilEvent::StateChange {
            layer: 4,
            old: ByteSil::NULL,
            new: ByteSil::ONE,
            timestamp: 0,
        };
        assert!(filter.matches(&event4));

        let event5 = SilEvent::StateChange {
            layer: 5,
            old: ByteSil::NULL,
            new: ByteSil::ONE,
            timestamp: 0,
        };
        assert!(!filter.matches(&event5));
    }

    #[test]
    fn test_filter_state_change() {
        let filter = EventFilter::StateChange;

        let event = SilEvent::StateChange {
            layer: 0,
            old: ByteSil::NULL,
            new: ByteSil::ONE,
            timestamp: 0,
        };
        assert!(filter.matches(&event));

        let error_event = SilEvent::Error {
            component: "test".into(),
            message: "error".into(),
            recoverable: false,
        };
        assert!(!filter.matches(&error_event));
    }

    #[test]
    fn test_history() {
        let bus = EventBus::with_history(3);

        let event1 = SilEvent::StateChange {
            layer: 0,
            old: ByteSil::NULL,
            new: ByteSil::ONE,
            timestamp: 0,
        };
        let event2 = SilEvent::StateChange {
            layer: 1,
            old: ByteSil::NULL,
            new: ByteSil::ONE,
            timestamp: 0,
        };

        bus.emit(event1).unwrap();
        bus.emit(event2).unwrap();

        let history = bus.history().unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_history_limit() {
        let bus = EventBus::with_history(2);

        for i in 0..5 {
            let event = SilEvent::StateChange {
                layer: i,
                old: ByteSil::NULL,
                new: ByteSil::ONE,
                timestamp: 0,
            };
            bus.emit(event).unwrap();
        }

        let history = bus.history().unwrap();
        assert_eq!(history.len(), 2); // Apenas os 2 últimos
    }

    #[test]
    fn test_clear_history() {
        let bus = EventBus::new();

        let event = SilEvent::StateChange {
            layer: 0,
            old: ByteSil::NULL,
            new: ByteSil::ONE,
            timestamp: 0,
        };
        bus.emit(event).unwrap();

        assert_eq!(bus.history().unwrap().len(), 1);

        bus.clear_history().unwrap();
        assert_eq!(bus.history().unwrap().len(), 0);
    }

    #[test]
    fn test_unsubscribe() {
        let bus = EventBus::new();

        bus.subscribe(EventFilter::All, |_| {}).unwrap();
        assert_eq!(bus.handler_count().unwrap(), 1);

        bus.unsubscribe(&EventFilter::All).unwrap();
        assert_eq!(bus.handler_count().unwrap(), 0);
    }
}
