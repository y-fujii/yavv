use std::cell::{Ref, RefCell, RefMut};
use std::sync::{Arc, Weak};
use std::*;

#[derive(Debug)]
struct Relation<T> {
    parent: Weak<T>,
    prev: Weak<T>,
    next: Option<Arc<T>>,
    child: Option<Arc<T>>,
}

#[derive(Debug)]
pub struct Node<T> {
    relation: RefCell<Relation<Node<T>>>,
    content: RefCell<T>,
}

impl<T> Node<T> {
    pub fn new(content: T) -> Arc<Self> {
        Arc::new(Node {
            relation: RefCell::new(Relation {
                parent: Weak::new(),
                prev: Weak::new(),
                next: None,
                child: None,
            }),
            content: RefCell::new(content),
        })
    }

    pub fn orphan(&self) {
        let mut relation = self.relation.borrow_mut();
        let parent = mem::replace(&mut relation.parent, Weak::new());
        let prev_w = mem::replace(&mut relation.prev, Weak::new());
        let prev_s = prev_w.upgrade();
        let next = relation.next.take();
        if let Some(ref next) = next {
            next.relation.borrow_mut().prev = prev_w;
        }
        if let Some(ref prev) = prev_s {
            prev.relation.borrow_mut().next = next;
        } else if let Some(parent) = parent.upgrade() {
            parent.relation.borrow_mut().child = next;
        }
        relation.parent = Weak::new();
    }

    pub fn prepend_child(self: &Arc<Self>, child: Arc<Self>) {
        let mut parent_relation = self.relation.borrow_mut();
        let tmp = parent_relation.child.take();
        if let Some(ref tmp) = tmp {
            tmp.relation.borrow_mut().prev = Arc::downgrade(&child);
        }

        {
            let mut child_relation = child.relation.borrow_mut();
            assert!(child_relation.parent.upgrade().is_none());
            assert!(child_relation.prev.upgrade().is_none());
            assert!(child_relation.next.is_none());
            child_relation.next = tmp;
            child_relation.parent = Arc::downgrade(self);
        }

        parent_relation.child = Some(child);
    }

    pub fn parent(&self) -> Option<Arc<Self>> {
        self.relation.borrow().parent.upgrade()
    }

    pub fn children(&self) -> ChildrenIterator<T> {
        ChildrenIterator {
            next: self.relation.borrow().child.clone(),
        }
    }

    pub fn ancestors(self: &Arc<Self>) -> AncestorsIterator<T> {
        AncestorsIterator {
            next: Some(self.clone()),
        }
    }

    pub fn content(&self) -> Ref<T> {
        self.content.borrow()
    }

    pub fn content_mut(&self) -> RefMut<T> {
        self.content.borrow_mut()
    }
}

#[derive(Debug)]
pub struct ChildrenIterator<T> {
    next: Option<Arc<Node<T>>>,
}

impl<T> iter::Iterator for ChildrenIterator<T> {
    type Item = Arc<Node<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take();
        if let Some(ref next) = next {
            self.next = next.relation.borrow_mut().next.clone();
        }
        next
    }
}

#[derive(Debug)]
pub struct AncestorsIterator<T> {
    next: Option<Arc<Node<T>>>,
}

#[test]
pub fn test() {
    let a = Node::new("A");
    let b = Node::new("B");
    let c = Node::new("C");
    a.prepend_child(b.clone());
    a.prepend_child(c.clone());
    b.orphan();
    c.orphan();
    for n in a.children() {
        dbg!(n.content());
    }
}
