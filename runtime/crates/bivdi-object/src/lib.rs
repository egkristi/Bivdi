//! Bivdi object store — Phase 0.
//!
//! Implements the decided primitives from `docs/object-store-format.md`:
//! - **Blob**: an immutable byte sequence named by its content hash.
//! - **Cell**: a mutable single-slot holder, updated only by compare-and-swap.
//! - **Catalog**: a persistent map from names to blobs/cells/sub-catalogs,
//!   reached only by handle (no root, no `/`, no upward walk).
//!
//! # Provisional
//!
//! - The content hash is BLAKE3-256. It is the leading *proposal*, not a
//!   decided item.
//! - The on-disk encoding (JSON, one file) is a *provisional* Phase 0 format,
//!   not the decided on-disk layout — see `docs/object-store-format.md`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// A content hash (BLAKE3-256, 32 bytes).
pub type Hash = [u8; 32];

/// Hash some bytes with BLAKE3-256 (provisional algorithm).
pub fn blake3_hash(bytes: &[u8]) -> Hash {
    *blake3::hash(bytes).as_bytes()
}

/// An immutable, content-addressed blob.
#[derive(Debug, Clone)]
pub struct Blob {
    hash: Hash,
    data: Vec<u8>,
}

impl Blob {
    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// A cell: the *only* mutation primitive — a single-slot holder updated by CAS.
#[derive(Debug, Clone)]
pub struct Cell {
    value: Option<Hash>,
}

impl Cell {
    fn new() -> Self {
        Self { value: None }
    }

    pub fn read(&self) -> Option<Hash> {
        self.value
    }

    /// Compare-and-swap. Returns `Err(actual)` if `expected` does not match the
    /// current value (the actual value is returned so the caller can retry).
    pub fn cas(&mut self, expected: Option<Hash>, new: Option<Hash>) -> Result<(), Option<Hash>> {
        if self.value == expected {
            self.value = new;
            Ok(())
        } else {
            Err(self.value)
        }
    }
}

/// A handle to a catalog (or object) within the store. In Phase 0 this is an
/// opaque id. It is *not* a path and cannot be walked upward from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Handle(pub(crate) u64);

/// A node in the object graph.
#[derive(Debug, Clone)]
enum Node {
    Blob(Arc<Blob>),
    Cell(Arc<Mutex<Cell>>),
    Catalog(Arc<Mutex<CatalogInner>>),
}

#[derive(Debug, Default)]
struct CatalogInner {
    entries: BTreeMap<String, Handle>,
}

/// The object store. All access is through handles; there is no root catalog.
#[derive(Debug, Default)]
pub struct Store {
    nodes: Mutex<BTreeMap<Handle, Node>>,
    next_id: Mutex<u64>,
}

impl Store {
    pub fn new() -> Self {
        Self::default()
    }

    fn alloc(&self, node: Node) -> Handle {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;
        let h = Handle(id);
        self.nodes.lock().unwrap().insert(h, node);
        h
    }

    /// Store a blob and return its handle. Content addressing means the blob is
    /// *named by* its hash — identical bytes hash identically. Phase 0 does not
    /// yet deduplicate storage: each `put_blob` allocates a fresh handle even
    /// for identical bytes. (Block-level deduplication is a future concern.)
    pub fn put_blob(&self, data: impl Into<Vec<u8>>) -> Handle {
        let data = data.into();
        let hash = blake3_hash(&data);
        let blob = Arc::new(Blob { hash, data });
        self.alloc(Node::Blob(blob))
    }

    /// Create an empty cell and return its handle.
    pub fn new_cell(&self) -> Handle {
        self.alloc(Node::Cell(Arc::new(Mutex::new(Cell::new()))))
    }

    /// Create an empty catalog and return its handle.
    pub fn new_catalog(&self) -> Handle {
        self.alloc(Node::Catalog(Arc::new(Mutex::new(CatalogInner::default()))))
    }

    fn with_node<R>(&self, h: Handle, f: impl FnOnce(&Node) -> R) -> Option<R> {
        let nodes = self.nodes.lock().unwrap();
        nodes.get(&h).map(f)
    }

    /// Read a blob's bytes by handle.
    pub fn get_blob(&self, h: Handle) -> Option<Vec<u8>> {
        self.with_node(h, |n| match n {
            Node::Blob(b) => Some(b.data.clone()),
            _ => None,
        })?
    }

    /// Read a cell's value.
    pub fn cell_read(&self, h: Handle) -> Option<Option<Hash>> {
        self.with_node(h, |n| match n {
            Node::Cell(c) => Some(c.lock().unwrap().read()),
            _ => None,
        })?
    }

    /// Compare-and-swap a cell.
    pub fn cell_cas(
        &self,
        h: Handle,
        expected: Option<Hash>,
        new: Option<Hash>,
    ) -> Result<(), CasError> {
        let node = self.nodes.lock().unwrap().get(&h).cloned();
        match node {
            Some(Node::Cell(c)) => c
                .lock()
                .unwrap()
                .cas(expected, new)
                .map_err(|actual| CasError { actual }),
            _ => Err(CasError { actual: None }),
        }
    }

    /// Look up a name in a catalog, returning the bound handle (if any).
    pub fn catalog_get(&self, catalog: Handle, name: &str) -> Option<Handle> {
        self.with_node(catalog, |n| match n {
            Node::Catalog(c) => c.lock().unwrap().entries.get(name).copied(),
            _ => None,
        })?
    }

    /// Bind (or rebind) a name to a handle inside a catalog.
    pub fn catalog_put(
        &self,
        catalog: Handle,
        name: &str,
        target: Handle,
    ) -> Result<(), ObjectError> {
        let node = self.nodes.lock().unwrap().get(&catalog).cloned();
        match node {
            Some(Node::Catalog(c)) => {
                c.lock().unwrap().entries.insert(name.to_string(), target);
                Ok(())
            }
            _ => Err(ObjectError::NotACatalog),
        }
    }

    /// Serialize the store to a JSON string (provisional Phase 0 format).
    ///
    /// # Provisional
    ///
    /// This JSON layout is a *provisional* on-disk encoding for Phase 0 — it is
    /// NOT the decided on-disk format (which is open; see
    /// `docs/object-store-format.md`). It exists so the runtime is a stateful,
    /// running implementation across restarts.
    pub fn save(&self) -> String {
        let nodes = self.nodes.lock().unwrap();
        let mut snap = Snapshot {
            next_id: *self.next_id.lock().unwrap(),
            blobs: BTreeMap::new(),
            cells: BTreeMap::new(),
            catalogs: BTreeMap::new(),
        };
        for (handle, node) in nodes.iter() {
            match node {
                Node::Blob(b) => {
                    snap.blobs.insert(handle.0, b.data.clone());
                }
                Node::Cell(c) => {
                    snap.cells.insert(handle.0, c.lock().unwrap().read());
                }
                Node::Catalog(c) => {
                    snap.catalogs
                        .insert(handle.0, c.lock().unwrap().entries.clone());
                }
            }
        }
        serde_json::to_string(&snap).expect("store snapshot serializes")
    }

    /// Load a store from the JSON produced by [`Store::save`].
    pub fn load(json: &str) -> Result<Self, ObjectError> {
        let snap: Snapshot =
            serde_json::from_str(json).map_err(|_| ObjectError::CorruptSnapshot)?;
        let mut nodes = BTreeMap::new();
        for (id, data) in snap.blobs {
            let hash = blake3_hash(&data);
            nodes.insert(Handle(id), Node::Blob(Arc::new(Blob { hash, data })));
        }
        for (id, value) in snap.cells {
            nodes.insert(Handle(id), Node::Cell(Arc::new(Mutex::new(Cell { value }))));
        }
        for (id, entries) in snap.catalogs {
            nodes.insert(
                Handle(id),
                Node::Catalog(Arc::new(Mutex::new(CatalogInner { entries }))),
            );
        }
        Ok(Self {
            nodes: Mutex::new(nodes),
            next_id: Mutex::new(snap.next_id),
        })
    }
}

/// The serializable snapshot of the store (provisional Phase 0 format).
#[derive(Debug, Serialize, Deserialize)]
struct Snapshot {
    next_id: u64,
    blobs: BTreeMap<u64, Vec<u8>>,
    cells: BTreeMap<u64, Option<Hash>>,
    catalogs: BTreeMap<u64, BTreeMap<String, Handle>>,
}

/// Object-store errors.
#[derive(Debug)]
pub enum ObjectError {
    NotACatalog,
    CorruptSnapshot,
}

/// A failed CAS: the actual value observed.
#[derive(Debug)]
pub struct CasError {
    pub actual: Option<Hash>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_addressing_is_deterministic() {
        let s = Store::new();
        let a = s.put_blob(b"hello".to_vec());
        let b = s.put_blob(b"hello".to_vec());
        assert_eq!(s.get_blob(a).unwrap(), b"hello");
        assert_eq!(s.get_blob(a).unwrap(), s.get_blob(b).unwrap());
        // Same content => same hash.
        let ha = match s.nodes.lock().unwrap().get(&a).unwrap() {
            Node::Blob(b) => b.hash(),
            _ => panic!(),
        };
        let hb = match s.nodes.lock().unwrap().get(&b).unwrap() {
            Node::Blob(b) => b.hash(),
            _ => panic!(),
        };
        assert_eq!(ha, hb);
    }

    #[test]
    fn cell_cas_retries() {
        let s = Store::new();
        let cell = s.new_cell();
        assert_eq!(s.cell_read(cell).unwrap(), None);
        // First CAS from None -> H1 succeeds.
        let h1 = blake3_hash(b"v1");
        s.cell_cas(cell, None, Some(h1)).unwrap();
        // Stale CAS with expected None fails and reports actual.
        let err = s.cell_cas(cell, None, Some(h1)).unwrap_err();
        assert_eq!(err.actual, Some(h1));
        // Correct CAS succeeds.
        let h2 = blake3_hash(b"v2");
        s.cell_cas(cell, Some(h1), Some(h2)).unwrap();
        assert_eq!(s.cell_read(cell).unwrap(), Some(h2));
    }

    #[test]
    fn catalog_is_reached_by_handle_not_root() {
        let s = Store::new();
        let cat = s.new_catalog();
        let blob = s.put_blob(b"data");
        s.catalog_put(cat, "doc", blob).unwrap();
        assert_eq!(s.catalog_get(cat, "doc"), Some(blob));
        // A different catalog is a disjoint universe.
        let other = s.new_catalog();
        assert_eq!(s.catalog_get(other, "doc"), None);
    }

    #[test]
    fn wrong_node_kind_errors_cleanly() {
        let s = Store::new();
        let blob = s.put_blob(b"x");
        assert!(s.catalog_get(blob, "nope").is_none());
        assert!(matches!(
            s.catalog_put(blob, "nope", blob),
            Err(ObjectError::NotACatalog)
        ));
        assert!(s.cell_read(blob).is_none());
    }

    #[test]
    fn save_and_load_roundtrip_preserves_state() {
        let s = Store::new();
        let blob = s.put_blob(b"durable data".to_vec());
        let cell = s.new_cell();
        let h = blake3_hash(b"v1");
        s.cell_cas(cell, None, Some(h)).unwrap();
        let cat = s.new_catalog();
        s.catalog_put(cat, "blob", blob).unwrap();
        s.catalog_put(cat, "cell", cell).unwrap();

        let json = s.save();
        let loaded = Store::load(&json).unwrap();

        assert_eq!(loaded.get_blob(blob).unwrap(), b"durable data");
        assert_eq!(loaded.cell_read(cell).unwrap(), Some(h));
        assert_eq!(loaded.catalog_get(cat, "blob"), Some(blob));
        assert_eq!(loaded.catalog_get(cat, "cell"), Some(cell));
    }

    #[test]
    fn load_rejects_corrupt_snapshot() {
        assert!(matches!(
            Store::load("not json"),
            Err(ObjectError::CorruptSnapshot)
        ));
    }
}
