use std::fmt::Debug;

use crate::expr::Var;
use crate::index_table::Id;

/// Optimized representation for [`NodeKind`]:
/// - [`NodeKind::True`] is represented as usize::MAX;
/// - [`NodeKind::False`] is represented as `usize::MAX - 1`;
/// - [`NodeKind::Internal`] is represented as any other value (in practice can't be bigger than isize::MAX).
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Node(usize);

impl Node {
    pub const TRUE: Node = Node(usize::MAX);
    pub const FALSE: Node = Node(usize::MAX - 1);

    pub const fn kind(&self) -> NodeKind {
        match *self {
            Self::TRUE => NodeKind::True,
            Self::FALSE => NodeKind::False,
            Node(index) => NodeKind::Internal(Id::new(index)),
        }
    }
}

impl From<NodeKind> for Node {
    fn from(kind: NodeKind) -> Self {
        match kind {
            NodeKind::True => Self::TRUE,
            NodeKind::False => Self::FALSE,
            NodeKind::Internal(internal) => internal.into(),
        }
    }
}

impl From<Id<InternalNode>> for Node {
    fn from(internal: Id<InternalNode>) -> Self {
        assert!(
            internal.get() < isize::MAX as usize,
            "Can't have ids bigger than isize::MAX"
        );
        Self(internal.get())
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            NodeKind::True => f.write_str("Node(true)"),
            NodeKind::False => f.write_str("Node(false)"),
            NodeKind::Internal(id) => f.debug_tuple("Node").field(&id).finish(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    True,
    False,
    Internal(Id<InternalNode>),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct InternalNode {
    pub label: Id<Var>,
    pub lo: Node,
    pub hi: Node,
}
