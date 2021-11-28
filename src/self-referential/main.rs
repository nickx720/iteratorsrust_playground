/// https://dev.to/arunanshub/self-referential-structs-in-rust-33cm
use std::{cell::RefCell, collections::BTreeSet, marker::PhantomPinned, pin::Pin, rc::Rc};

type RcCell<T> = Rc<RefCell<T>>;

#[derive(Debug)]
struct Holder {
    set_of_me: BTreeSet<*mut Me>,
}
impl Holder {
    fn new() -> RcCell<Self> {
        Rc::new(RefCell::new(Self {
            set_of_me: Default::default(),
        }))
    }

    fn mutate_value_of_me(&self, val: i32) {
        self.set_of_me.iter().for_each(|a| {
            let a = unsafe { Pin::new_unchecked(&mut **a) };
            a.mutate_me(val);
        })
    }
}
#[derive(Debug)]
struct Me {
    name: String,
    mutate_by_holder: i32,
    my_holder: RcCell<Holder>,
    _pinned: PhantomPinned,
}

impl Me {
    pub fn new(my_holder: RcCell<Holder>, name: impl Into<String>) -> Pin<Box<Self>> {
        let mut this = Box::pin(Self {
            name: name.into(),
            mutate_by_holder: 0,
            my_holder,
            _pinned: PhantomPinned,
        });
        let this_ptr: *mut _ = unsafe { this.as_mut().get_unchecked_mut() };
        this.my_holder.borrow_mut().set_of_me.insert(this_ptr);
        this
    }

    fn mutate_me(self: Pin<&mut Self>, val: i32) {
        let this = unsafe { self.get_unchecked_mut() };
        this.mutate_by_holder += val;
    }
}

impl Drop for Me {
    fn drop(&mut self) {
        println!("Dropping {:#?}", self);
        let this = &(self as *mut _);
        self.my_holder.borrow_mut().set_of_me.remove(this);
    }
}

fn make_ref_of_holder(holder: RcCell<Holder>) {
    let holder = Rc::clone(&holder);
    println!("Making a ref of {:?}", holder);
    println!("No. of refs = {}", Rc::strong_count(&holder));
}

#[derive(Debug)]
struct Test {
    value: String,
    pointer_to_value: *const String,
}

impl Test {
    fn new(txt: &str) -> Pin<Box<Self>> {
        let mut this = Box::pin(Test {
            value: String::from(txt),
            pointer_to_value: std::ptr::null(),
        });
        this.as_mut().pointer_to_value = &this.value;
        this
    }

    // Whenever using Pin -> self value changes to Pin<T>
    fn get_value(self: Pin<&Self>) -> &str {
        &self.get_ref().value
    }

    // Whenever using Pin -> self value changes to Pin<T>
    fn get_pointer_to_value(self: Pin<&Self>) -> &String {
        unsafe { &*(self.pointer_to_value) }
    }
}

#[derive(Debug)]
struct Test_P {
    value: String,
    pointer_to_value: *const String,
    _pinned: PhantomPinned,
}

impl Test_P {
    fn new(txt: &str) -> Pin<Box<Self>> {
        let mut this = Box::pin(Test_P {
            value: String::from(txt),
            // In `C`, you'd call this initalize to null
            pointer_to_value: std::ptr::null(),
            _pinned: PhantomPinned,
        });
        unsafe {
            this.as_mut().get_unchecked_mut().pointer_to_value = &this.value;
        }
        this
    }
}

fn main() {
    let mut test1 = Test::new("test1");
    let mut test2 = Test::new("test2");

    println!(
        "test1: value: {}, ptr to value: {}",
        test1.as_ref().get_value(),
        test1.as_ref().get_pointer_to_value()
    );
    println!(
        "test2: value: {}, ptr to value: {}",
        test2.as_ref().get_value(),
        test2.as_ref().get_pointer_to_value()
    );
    println!("Swapping..");
    std::mem::swap(&mut test1, &mut test2);
    println!(
        "test1: value: {}, ptr to value: {}",
        test1.as_ref().get_value(),
        test1.as_ref().get_pointer_to_value()
    );
    println!(
        "test2: value: {}, ptr to value: {}",
        test2.as_ref().get_value(),
        test2.as_ref().get_pointer_to_value()
    );
    println!("---------------------");
    let holder = Holder::new();
    // explicit Rc cloning
    let a = Me::new(Rc::clone(&holder), "a");
    let b = Me::new(Rc::clone(&holder), "b");

    holder.borrow_mut().mutate_value_of_me(455);
    make_ref_of_holder(Rc::clone(&holder))
}
