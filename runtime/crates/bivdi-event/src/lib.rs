//! Bivdi event bus — first-class typed events.
//!
//! Implements the decided "events over polling" invariant from `docs/events.md`:
//! changes produce structured, first-class events consumed by a single fabric.
//! Events are *typed records*, not text to parse, and a shared **correlation
//! identity** traces a single action across components.
//!
//! # Provisional
//!
//! Phase 0 is single-process, so this is an in-memory bus. The durable,
//! network-distributed fabric belongs to Bivdi Core.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::sync::Mutex;

/// A correlation id: traces one action across components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CorrelationId(pub u64);

/// The kinds of first-class events (representative, not exhaustive).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Event {
    ObjectCreated { correlation: u64, object: String },
    ObjectChanged { correlation: u64, object: String },
    ObjectDeleted { correlation: u64, object: String },
    WorkloadStarted { correlation: u64, workload: String },
    WorkloadStopped { correlation: u64, workload: String },
    WorkloadFailed { correlation: u64, workload: String },
    CapabilityGranted { correlation: u64, cap: u64 },
    CapabilityUsed { correlation: u64, cap: u64 },
    CapabilityRevoked { correlation: u64, cap: u64 },
    DeviceAttached { correlation: u64, device: String },
    DeviceRemoved { correlation: u64, device: String },
    PolicyChanged { correlation: u64 },
    GenerationActivated { correlation: u64, generation: u64 },
    RollbackStarted { correlation: u64, to: u64 },
}

impl Event {
    /// The correlation id carried by every event.
    pub fn correlation(&self) -> u64 {
        match self {
            Event::ObjectCreated { correlation, .. }
            | Event::ObjectChanged { correlation, .. }
            | Event::ObjectDeleted { correlation, .. }
            | Event::WorkloadStarted { correlation, .. }
            | Event::WorkloadStopped { correlation, .. }
            | Event::WorkloadFailed { correlation, .. }
            | Event::CapabilityGranted { correlation, .. }
            | Event::CapabilityUsed { correlation, .. }
            | Event::CapabilityRevoked { correlation, .. }
            | Event::DeviceAttached { correlation, .. }
            | Event::DeviceRemoved { correlation, .. }
            | Event::PolicyChanged { correlation }
            | Event::GenerationActivated { correlation, .. }
            | Event::RollbackStarted { correlation, .. } => *correlation,
        }
    }
}

/// A subscription filter over event kinds. Empty means "match all".
#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub kinds: Vec<String>,
}

impl Filter {
    pub fn all() -> Self {
        Self::default()
    }

    pub fn kind(kind: &str) -> Self {
        Self {
            kinds: vec![kind.to_string()],
        }
    }

    fn matches(&self, event: &Event) -> bool {
        if self.kinds.is_empty() {
            return true;
        }
        let kind = event_kind(event);
        self.kinds.iter().any(|k| k == kind)
    }
}

/// The event bus: publish typed events, subscribe with a filter, and read
/// history. A single fabric for applications, automation, and observability.
#[derive(Debug, Default)]
pub struct EventBus {
    inner: Mutex<BusInner>,
}

#[derive(Debug, Default)]
struct BusInner {
    history: VecDeque<Event>,
    subscribers: BTreeMap<u64, (Filter, VecDeque<Event>)>,
    next_subscriber: u64,
    next_correlation: u64,
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// A fresh correlation id for a new action.
    pub fn new_correlation(&self) -> CorrelationId {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_correlation;
        inner.next_correlation += 1;
        CorrelationId(id)
    }

    /// Publish an event to the fabric and all matching subscribers.
    pub fn publish(&self, event: Event) {
        let mut inner = self.inner.lock().unwrap();
        for (_, (filter, queue)) in inner.subscribers.iter_mut() {
            if filter.matches(&event) {
                queue.push_back(event.clone());
            }
        }
        inner.history.push_back(event);
    }

    /// Subscribe with a filter; returns a subscription id to drain with.
    pub fn subscribe(&self, filter: Filter) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_subscriber;
        inner.next_subscriber += 1;
        inner.subscribers.insert(id, (filter, VecDeque::new()));
        id
    }

    /// Drain (consume) the queued events for a subscriber, oldest first.
    pub fn drain(&self, subscriber: u64) -> Vec<Event> {
        let mut inner = self.inner.lock().unwrap();
        match inner.subscribers.get_mut(&subscriber) {
            Some((_, queue)) => queue.drain(..).collect(),
            None => Vec::new(),
        }
    }

    /// All events ever published (append-only history).
    pub fn history(&self) -> Vec<Event> {
        self.inner.lock().unwrap().history.iter().cloned().collect()
    }

    /// Count of events matching a filter, from history.
    pub fn count(&self, filter: &Filter) -> usize {
        self.inner
            .lock()
            .unwrap()
            .history
            .iter()
            .filter(|e| filter.matches(e))
            .count()
    }
}

/// The serialized kind tag of an event, for filters.
fn event_kind(event: &Event) -> &'static str {
    match event {
        Event::ObjectCreated { .. } => "object_created",
        Event::ObjectChanged { .. } => "object_changed",
        Event::ObjectDeleted { .. } => "object_deleted",
        Event::WorkloadStarted { .. } => "workload_started",
        Event::WorkloadStopped { .. } => "workload_stopped",
        Event::WorkloadFailed { .. } => "workload_failed",
        Event::CapabilityGranted { .. } => "capability_granted",
        Event::CapabilityUsed { .. } => "capability_used",
        Event::CapabilityRevoked { .. } => "capability_revoked",
        Event::DeviceAttached { .. } => "device_attached",
        Event::DeviceRemoved { .. } => "device_removed",
        Event::PolicyChanged { .. } => "policy_changed",
        Event::GenerationActivated { .. } => "generation_activated",
        Event::RollbackStarted { .. } => "rollback_started",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_and_subscribe_matches_filter() {
        let bus = EventBus::new();
        let sub = bus.subscribe(Filter::kind("object_created"));
        bus.publish(Event::ObjectCreated {
            correlation: 1,
            object: "photo-1".into(),
        });
        bus.publish(Event::ObjectDeleted {
            correlation: 2,
            object: "photo-1".into(),
        });
        let drained = bus.drain(sub);
        assert_eq!(drained.len(), 1);
        assert!(matches!(drained[0], Event::ObjectCreated { .. }));
    }

    #[test]
    fn empty_filter_matches_all() {
        let bus = EventBus::new();
        let sub = bus.subscribe(Filter::all());
        bus.publish(Event::ObjectCreated {
            correlation: 1,
            object: "a".into(),
        });
        bus.publish(Event::PolicyChanged { correlation: 2 });
        assert_eq!(bus.drain(sub).len(), 2);
    }

    #[test]
    fn history_is_append_only_and_queryable() {
        let bus = EventBus::new();
        bus.publish(Event::GenerationActivated {
            correlation: 1,
            generation: 7,
        });
        bus.publish(Event::GenerationActivated {
            correlation: 2,
            generation: 8,
        });
        assert_eq!(bus.history().len(), 2);
        assert_eq!(bus.count(&Filter::kind("generation_activated")), 2);
    }

    #[test]
    fn correlation_ids_are_unique() {
        let bus = EventBus::new();
        let a = bus.new_correlation();
        let b = bus.new_correlation();
        assert_ne!(a, b);
    }

    #[test]
    fn events_roundtrip_through_json() {
        let e = Event::WorkloadStarted {
            correlation: 5,
            workload: "web".into(),
        };
        let s = serde_json::to_string(&e).unwrap();
        let back: Event = serde_json::from_str(&s).unwrap();
        assert_eq!(e, back);
    }
}
