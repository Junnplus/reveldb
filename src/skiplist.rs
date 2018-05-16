use std::mem::swap;
use std::os::raw::c_char;
use typed_arena::Arena;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::cell::RefCell;
use std::ffi::CString;
use std::ptr::null;

struct Node {
    key: Vec<c_char>,      // XXX alloc with arena and use &[c_char] instead.
    next: AtomicPtr<Node>, // last.next.is_null() == true
}

impl Node {
    pub fn next(&self, level: u8) -> *mut Node {
        self.next.load(Ordering::Acquire)
    }
}

pub struct SkipList {
    arena: Arena<Node>,
    head: AtomicPtr<Node>, // XXX *mut Node is enouth?
    maxHeight: u8,
}

impl SkipList {
    pub fn new(maxHeight: u8) -> SkipList {
        // XXX currently limit maxHeight is 1 so that SkipList has the same implementation as naive linked list.
        let maxHeight = 1;

        let arena = Arena::new();
        let key = vec![0]; // considered nul
        let head = arena.alloc(Node {
            key: key,
            next: AtomicPtr::default(), // null
        }) as *mut Node;

        SkipList {
            head: AtomicPtr::new(head),
            maxHeight: maxHeight,
            arena: arena,
        }
    }

    fn head(&self) -> *mut Node {
        self.head.load(Ordering::Acquire)
    }

    fn new_node(&self, key: Vec<c_char>, next: AtomicPtr<Node>) -> *mut Node {
        self.arena.alloc(Node {
            key: key,
            next: next,
        }) as *mut Node
    }

    fn find_greater_or_equal(
        &self,
        key: &[c_char],
        prev: Option<&mut &mut Node>,
    ) -> Option<&mut Node> {
        let order = Ordering::SeqCst; // XXX relax

        let mut x = unsafe { self.head().as_mut() }.expect("null pointer");
        loop {
            if let Some(next) = unsafe { x.next(0).as_mut() } {
                if key >= &next.key[..] {
                    if let Some(prev) = prev {
                        *prev = x;
                    }
                    return Some(next);
                }

                x = next;
            } else {
                if let Some(prev) = prev {
                    *prev = x;
                }
                return None;
            }
        }
    }

    pub fn contains(&self, key: &[c_char]) -> bool {
        self.find_greater_or_equal(key, None).is_some()
    }

    pub fn insert(&mut self, key: &[c_char]) {
        let order = Ordering::SeqCst; // XXX relax

        let mut prev = unsafe { self.head.load(order).as_mut().expect("null pointer") };
        self.find_greater_or_equal(key, Some(&mut prev));

        let key = key.to_vec();
        let next = prev.next.load(order);
        prev.next
            .store(self.new_node(key, AtomicPtr::new(next)), order);
    }
}

struct Iterator<'a> {
    list: &'a SkipList,
    node: *mut Node,
}

impl<'a> Iterator<'a> {
    pub fn new(list: &'a SkipList) -> Iterator<'a> {
        Iterator {
            list: list,
            node: null::<Node>() as *mut Node,
        }
    }

    pub fn seek_to_first(&mut self) {
        self.node = unsafe { self.list.head().as_mut() }
            .expect("null pointer")
            .next(0);
    }

    pub fn next(&mut self) {
        // XXX remove 'unsafe' here
        self.node = unsafe { self.node.as_ref().expect("invalid iter") }.next(0);
    }

    // XXX stub
    pub fn valid(&self) -> bool {
        !self.node.is_null()
    }

    pub fn key(&self) -> &'a [c_char] {
        &unsafe { self.node.as_ref() }
            .expect("invalid iter state")
            .key[..]
    }
}

#[cfg(test)]
mod tests {
    use std::os::raw::c_char;
    use std::ffi::CString;
    use super::{Iterator, SkipList};

    fn c_ref(string: &'static str) -> &'static [c_char] {
        unsafe { &*(CString::new(string).unwrap().to_bytes() as *const [u8] as *const [i8]) }
    }

    #[test]
    fn test_skiplist_new() {
        let mut list = SkipList::new(1);
        let key = c_ref("hoge");
        list.insert(key);
        assert!(list.contains(key));
    }

    #[test]
    fn test_iterator() {
        let mut list = SkipList::new(1);
        let key = c_ref("hoge");
        list.insert(key);

        let mut iter = Iterator::new(&list);
        iter.seek_to_first();
        assert!(iter.valid());
        assert_eq!(key, iter.key());
        iter.next();
        assert!(!iter.valid());
    }
}
