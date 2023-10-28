use std::cmp::Ordering;

use rustc_hash::FxHashMap;

use crate::expr::Var;
use crate::index_table::{Id, IndexTable};
use crate::{InternalNode, Node, NodeKind};

#[derive(Default)]
pub struct Pool {
    pub nodes: IndexTable<InternalNode>,
    pub not_cache: FxHashMap<Id<InternalNode>, Node>,
    pub and_cache: FxHashMap<(Id<InternalNode>, Id<InternalNode>), Node>,
    pub or_cache: FxHashMap<(Id<InternalNode>, Id<InternalNode>), Node>,
    pub exists_cache: FxHashMap<(Id<Var>, Id<InternalNode>), Node>,
}

impl Pool {
    pub fn new_node(&mut self, label: Id<Var>, lo: Node, hi: Node) -> Node {
        if lo == hi {
            return lo;
        }

        self.nodes
            .get_or_insert(InternalNode { label, lo, hi })
            .into()
    }

    fn sort(
        &self,
        x: Id<InternalNode>,
        y: Id<InternalNode>,
    ) -> (Id<InternalNode>, Id<InternalNode>) {
        match Ord::cmp(&self.nodes[x].label, &self.nodes[y].label) {
            Ordering::Equal if x > y => (y, x),
            Ordering::Greater => (y, x),
            _ => (x, y),
        }
    }

    pub fn variable(&mut self, var: Id<Var>) -> Node {
        let res = self.nodes.get_or_insert(InternalNode {
            label: var,
            lo: Node::TRUE,
            hi: Node::FALSE,
        });
        res.into()
    }

    pub fn not(&mut self, x: Node) -> Node {
        let ix = match x.kind() {
            NodeKind::True => return Node::FALSE,
            NodeKind::False => return Node::TRUE,
            NodeKind::Internal(ix) => ix,
        };

        if let Some(&cached) = self.not_cache.get(&ix) {
            return cached.into();
        }

        let nx = self.nodes[ix];
        let lo = self.not(nx.lo);
        let hi = self.not(nx.hi);

        let res = self.new_node(nx.label, lo, hi);
        self.not_cache.insert(ix, res);
        res
    }

    pub fn and(&mut self, x: Node, y: Node) -> Node {
        let (ix, iy) = match (x.kind(), y.kind()) {
            (NodeKind::False, _) | (_, NodeKind::False) => return Node::FALSE,
            (NodeKind::True, k) | (k, NodeKind::True) => return k.into(),
            (NodeKind::Internal(iy), NodeKind::Internal(ix)) if ix == iy => return ix.into(),
            (NodeKind::Internal(ix), NodeKind::Internal(iy)) => self.sort(ix, iy),
        };

        if let Some(&cached) = self.and_cache.get(&(ix, iy)) {
            return cached.into();
        }

        let (nx, ny) = (self.nodes[ix], self.nodes[iy]);

        let (lo, hi) = match nx.label == ny.label {
            true => (self.and(nx.lo, ny.lo), self.and(nx.hi, ny.hi)),
            false => (self.and(nx.lo, iy.into()), self.and(nx.hi, iy.into())),
        };
        let res = self.new_node(nx.label, lo, hi);
        self.and_cache.insert((ix, iy), res);
        res
    }

    pub fn or(&mut self, x: Node, y: Node) -> Node {
        let (ix, iy) = match (x.kind(), y.kind()) {
            (NodeKind::True, _) | (_, NodeKind::True) => return Node::TRUE,
            (NodeKind::False, k) | (k, NodeKind::False) => return k.into(),
            (NodeKind::Internal(iy), NodeKind::Internal(ix)) if ix == iy => return ix.into(),
            (NodeKind::Internal(ix), NodeKind::Internal(iy)) => self.sort(ix, iy),
        };

        if let Some(&cached) = self.or_cache.get(&(ix, iy)) {
            return cached.into();
        }

        let (nx, ny) = (self.nodes[ix], self.nodes[iy]);

        let (lo, hi) = match nx.label == ny.label {
            true => (self.or(nx.lo, ny.lo), self.or(nx.hi, ny.hi)),
            false => (self.or(nx.lo, iy.into()), self.or(nx.hi, iy.into())),
        };
        let res = self.new_node(nx.label, lo, hi);
        self.or_cache.insert((ix, iy), res);
        res
    }

    pub fn exists(&mut self, label: Id<Var>, x: Node) -> Node {
        let ix = match x.kind() {
            NodeKind::True => return Node::TRUE,
            NodeKind::False => return Node::FALSE,
            NodeKind::Internal(ix) => ix,
        };

        if let Some(&cached) = self.exists_cache.get(&(label, ix)) {
            return cached.into();
        }

        let nx = self.nodes[ix];

        if nx.label > label {
            return ix.into();
        }
        if nx.label == label {
            return self.or(nx.lo, nx.hi);
        }

        let lo = self.exists(label, nx.lo);
        let hi = self.exists(label, nx.hi);
        let res = self.new_node(nx.label, lo, hi);
        self.exists_cache.insert((label, ix), res);
        res
    }

    pub fn exists_many(&mut self, mut labels: Vec<Id<Var>>, x: Node) -> Node {
        labels.sort_unstable();
        self.exists_many_sorted(labels, x)
    }

    pub fn exists_many_sorted(
        &mut self,
        labels: impl IntoIterator<Item = Id<Var>>,
        x: Node,
    ) -> Node {
        labels.into_iter().fold(x, |x, label| self.exists(label, x))
    }
}
