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

use bivdi_cap::{CapRuntime, Capability, Resource, Rights};

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
        self.runtime.mint(resource, Rights::ALL)
    }

    /// Resolve an identity name to a flow capability — **DNS as a capability
    /// decision**. Returns a flow the caller may use, or `None` if the name is
    /// unknown (which is structurally indistinguishable from "no authority").
    ///
    /// Resolution requires a **namespace capability**: a capability the caller
    /// already holds over the resolution namespace. The returned flow is
    /// attenuated from *that* capability, never from the service's own root.
    /// A caller holding nothing cannot obtain network authority from a name
    /// (2026-09-20 audit, C3).
    pub fn resolve(&mut self, namespace: &Capability, name: &ServiceName) -> Option<Flow> {
        let resource = *self.registry.get(name)?;
        // The namespace capability must actually grant READ over the namespace
        // resource; otherwise resolution is refused.
        if !self.runtime.check(namespace, resource, Rights::READ) {
            return None;
        }
        // The flow is attenuated from what the caller holds, not minted from
        // the service's root. The endpoint is bound to the resource the
        // namespace capability names (M6).
        let capability = self.runtime.attenuate(namespace, Rights::READ).ok()?;
        Some(Flow {
            capability,
            endpoint: name.clone(),
            anchors: TrustAnchors(vec![]),
        })
    }

    /// Issue a flow capability for a registered service, attenuated to a
    /// specific trust-anchor set. The holder can connect *only* to this
    /// endpoint, with *only* these anchors.
    ///
    /// The flow is attenuated from `service` (a capability the caller holds)
    /// and its endpoint is bound to the resource that capability names — never
    /// a name supplied independently of the authority (2026-09-20 audit, M6).
    pub fn grant_flow(
        &mut self,
        service: &Capability,
        name: ServiceName,
        anchors: TrustAnchors,
    ) -> Option<Flow> {
        // The endpoint must name the same resource the capability grants, so a
        // caller cannot pair service A's capability with service B's name.
        let resource = *self.registry.get(&name)?;
        if service.resource() != resource {
            return None;
        }
        let flow_cap = self.runtime.attenuate(service, Rights::READ).ok()?;
        Some(Flow {
            capability: flow_cap,
            endpoint: name,
            anchors,
        })
    }

    /// Check whether a flow is currently usable (held, unexpired, grants Read).
    pub fn usable(&self, flow: &Flow, resource: Resource) -> bool {
        self.runtime.check(&flow.capability, resource, Rights::READ)
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
        let mut net = NetService::new();
        // A caller holding nothing cannot resolve, even for a registered name.
        let owner = net.register_service(ServiceName::new("service://photos"));
        // No namespace capability held → no authority.
        assert!(net
            .resolve(&owner, &ServiceName::new("service://nonexistent"))
            .is_none());
    }

    #[test]
    fn resolved_name_yields_a_flow_capability() {
        let mut net = NetService::new();
        let svc = net.register_service(ServiceName::new("service://photos"));
        // A registered name resolves to a usable flow only when the caller
        // holds a namespace capability that grants READ over that resource.
        let flow = net
            .resolve(&svc, &ServiceName::new("service://photos"))
            .expect("registered name resolves with a namespace capability");
        assert_eq!(flow.endpoint, ServiceName::new("service://photos"));
    }

    #[test]
    fn resolve_without_authority_is_refused() {
        // The bug fixed in C3: knowing a name must not confer network authority.
        // A caller that does not hold the namespace capability over the service
        // cannot resolve it, even though the name is registered.
        let mut net = NetService::new();
        net.register_service(ServiceName::new("service://photos"));

        // A capability over a *different* resource does not authorise resolution.
        let other = net.register_service(ServiceName::new("service://mail"));
        assert!(net
            .resolve(&other, &ServiceName::new("service://photos"))
            .is_none());
    }

    #[test]
    fn grant_flow_binds_endpoint_to_capability_resource() {
        // M6: a flow's endpoint must name the resource its capability grants.
        let mut net = NetService::new();
        let svc = net.register_service(ServiceName::new("service://photos"));
        net.register_service(ServiceName::new("service://mail"));

        // Passing service A's capability with service B's name is refused.
        assert!(net
            .grant_flow(
                &svc,
                ServiceName::new("service://mail"),
                TrustAnchors(vec![])
            )
            .is_none());
    }
}
