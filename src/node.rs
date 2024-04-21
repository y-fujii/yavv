use std::cell::{Ref, RefCell, RefMut};
use std::sync::{Arc, Weak};
use std::*;

struct Relation<T> {
    parent: Weak<T>,
    children: Vec<Arc<T>>,
}

pub struct Node<T> {
    relation: RefCell<Relation<Node<T>>>,
    content: RefCell<T>,
}

impl<T> Node<T> {
    pub fn new(content: T) -> Arc<Self> {
        Arc::new(Node {
            relation: RefCell::new(Relation {
                parent: Weak::new(),
                children: Vec::new(),
            }),
            content: RefCell::new(content),
        })
    }

    pub fn append_child(self: &Arc<Self>, child: Arc<Self>) {
        assert!(child.parent().is_none());
        child.relation.borrow_mut().parent = Arc::downgrade(self);
        self.relation.borrow_mut().children.push(child);
    }

    pub fn remove_child(&self, child: &Self) -> Arc<Self> {
        assert!(ptr::eq(Arc::as_ptr(&child.parent().unwrap()), self));
        child.relation.borrow_mut().parent = Weak::new();
        let children = &mut self.relation.borrow_mut().children;
        children.swap_remove(children.iter().position(|n| ptr::eq(Arc::as_ptr(n), child)).unwrap())
    }

    pub fn parent(&self) -> Option<Arc<Self>> {
        self.relation.borrow().parent.upgrade()
    }

    pub fn children(&self) -> Ref<[Arc<Node<T>>]> {
        Ref::map(self.relation.borrow(), |r| &r.children[..])
    }

    pub fn content(&self) -> Ref<T> {
        self.content.borrow()
    }

    pub fn content_mut(&self) -> RefMut<T> {
        self.content.borrow_mut()
    }
}

pub fn test() {
    let a = Node::new("A");
    let b = Node::new("B");
    let mut content = a.content_mut();
    a.append_child(b.clone());
    *content = "C";
    for n in &*a.children() {
        dbg!(&n.content);
    }
    a.remove_child(&b);
}
