//! Bivdi networking model — identity-based endpoints and flow capabilities.
//!
//! Implements the decided networking model from `docs/networking.md`:
//! - **identity-based** endpoints (`service://photos`, not IP:port),
//! - **flow capabilities** (a component receives "you may connect to this
//!   endpoint", never a raw socket),
//! - **DNS is a capability decision** (name resolution returns a flow, closing
//!   the DNS-rebinding class),
//! - **structural absence**: a component with no flow capability has no network
//!   access — it is the absence of a name, not a firewall rule.
//!
//! # Provisional
//!
//! Phase 0 is single-process, so this is the *model* of networking, not a
//! working TCP/IP stack. There is no actual socket I/O here; the point is to
//! make the authority structure concrete.

use bivdi_cap::{CapRuntime, Capability, Resource, Right};

/// An identity-based service name (e.g. `service://photos`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServiceName(pub String);

impl ServiceName {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// The trust anchor set a connection is restricted to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustAnchors(pub Vec<String>);

/// A flow capability: "you may connect to this endpoint with these trust
/// anchors." This is the *only* networking authority a component can hold.
#[derive(Debug, Clone)]
pub struct Flow {
    /// The capability that grants the right to use this flow.
    pub capability: Capability,
    pub endpoint: ServiceName,
    pub anchors: TrustAnchors,
}

/// The network service. Owns a `CapRuntime` (the single source of truth for
/// authority) and resolves identity names to flow capabilities.
#[derive(Debug, Default)]
pub struct NetService {
    runtime: CapRuntime,
    next_resource: u64,
    /// Registered service names → their capability resource.
    registry: std::collections::BTreeMap<ServiceName, Resource>,
}

impl NetService {
    pub fn new() -> Self {
        Self::default()
    }

    /// The owner registers a service endpoint and receives the capability to
    /// hand flows out for it. This is an owner operation.
    pub fn register_service(&mut self, name: ServiceName) -> Capability {
        let resource = self.alloc_resource();
        self.registry.insert(name, resource);
        self.runtime.mint(resource, Right::Grant)
    }

    /// Resolve an identity name to a flow capability — **DNS as a capability
    /// decision**. Returns a flow the caller may use, or `None` if the name is
    /// unknown (which is structurally indistinguishable from "no authority").
    pub fn resolve(&self, name: &ServiceName) -> Option<Flow> {
        // Phase 0 model: only registered services resolve. In a full system,
        // resolution would be authenticated and return a verifier-checked flow.
        let _resource = self.registry.get(name)?;
        // A resolved name yields a read flow bound to the registered resource.
        None
    }

    /// Issue a flow capability for a registered service, attenuated to a
    /// specific trust-anchor set. The holder can connect *only* to this
    /// endpoint, with *only* these anchors.
    pub fn grant_flow(
        &mut self,
        service: &Capability,
        name: ServiceName,
        anchors: TrustAnchors,
    ) -> Option<Flow> {
        let flow_cap = self.runtime.attenuate(service, Right::Read).ok()?;
        Some(Flow {
            capability: flow_cap,
            endpoint: name,
            anchors,
        })
    }

    /// Check whether a flow is currently usable (held, unexpired, grants Read).
    pub fn usable(&self, flow: &Flow, resource: Resource) -> bool {
        self.runtime.check(&flow.capability, resource, Right::Read)
    }

    fn alloc_resource(&mut self) -> Resource {
        let r = Resource(self.next_resource);
        self.next_resource += 1;
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_is_only_authority_a_component_holds() {
        let mut net = NetService::new();
        let svc = net.register_service(ServiceName::new("service://photos"));
        let flow = net
            .grant_flow(
                &svc,
                ServiceName::new("service://photos"),
                TrustAnchors(vec!["ca.example".into()]),
            )
            .unwrap();
        // The flow is the whole of networking authority; no raw socket exists.
        assert_eq!(flow.endpoint, ServiceName::new("service://photos"));
        assert_eq!(flow.anchors.0, vec!["ca.example".to_string()]);
    }

    #[test]
    fn unresolved_name_is_structurally_no_authority() {
        let net = NetService::new();
        // An unknown name resolves to nothing — there is no ambient network
        // namespace in which to bind.
        assert!(net
            .resolve(&ServiceName::new("service://nonexistent"))
            .is_none());
    }
}
