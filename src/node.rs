use std::ptr::NonNull;

pub struct Node {
    pub name: String,
    first_child: Option<Box<Node>>,
    next_sibling: Option<Box<Node>>,

    parent: Option<NonNull<Node>>,

    attachment: Option<Box<dyn Attachment>>,
}

impl Node {
    pub fn new(name: &str, attachment: Option<Box<dyn Attachment>>) -> Self {
        Self {
            name: name.to_string(),
            first_child: None,
            next_sibling: None,
            parent: None,
            attachment,
        }
    }

    pub fn add_child(&mut self, mut child: Node) {
        child.parent = Some(unsafe { NonNull::new_unchecked(self as *mut _) });

        if let Some(first_child) = &mut self.first_child {
            let mut current_sibling = first_child;

            while let Some(ref mut child) = current_sibling.next_sibling {
                current_sibling = child;
            }

            current_sibling.next_sibling = Some(Box::new(child));
        } else {
            self.first_child = Some(Box::new(child));
        }
    }

    pub fn iter(&self) -> NodeIterator {
        NodeIterator::new(self)
    }
}

pub trait Attachment {
    fn tick(&mut self);
    fn draw(&self);

    fn on_entered_tree(&self);
    fn on_left_tree(&self);
}

pub struct NodeIterator<'a, 'b> {
    root: &'a Node,
    current: Option<&'b Node>,
}

impl<'a: 'b, 'b> NodeIterator<'a, 'b> {
    pub fn new(root: &'a Node) -> Self {
        Self {
            root,
            current: Some(root),
        }
    }
}

impl<'a, 'b> Iterator for NodeIterator<'a, 'b> {
    type Item = &'b Node;

    fn next(&mut self) -> Option<Self::Item> {
        let out = self.current;

        if let Some(current) = &self.current {
            if let Some(first_child) = &current.first_child {
                self.current = Some(first_child);
            } else if let Some(next_sibling) = &current.next_sibling {
                self.current = Some(next_sibling);
            } else if current.parent.is_some() {
                // Search up until a parent has a next sibling.
                let mut current_parent = current.parent;

                while let Some(parent) = current_parent {
                    if std::ptr::eq(parent.as_ptr(), self.root) {
                        self.current = None;
                    }

                    if let Some(next_sibling) = unsafe { &parent.as_ref().next_sibling } {
                        self.current = Some(next_sibling);
                    }
                    
                    current_parent = unsafe { parent.as_ref().parent };
                }
            } else {
                self.current = None;
            }
        }

        out
    }
}

