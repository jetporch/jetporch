TODO items
==========

A list of items for tech-preview

* syntax command -- and everything involved in traversal of everything including roles/handlers
* load and test dynamic inventory scripts
* local command
* ssh command
* work on local connection plugin (run/push/get)
* work on remote connection plugin (run/push/get)
* templating engine
* task keyword logic
* DOCS
* example github


/* MAYBE FOR LATER
use crossbeam_utils::atomic::AtomicCell;
use std::thread;

fn main() {
    let rofl = Some("lol".to_string());
    
    let foo = AtomicCell::new(None);
    foo.store(rofl);
    
    let bar = thread::spawn(move || {
        println!("{:?}", foo.into_inner());
    });
    
    bar.join().unwrap();
}
*/
// default implementation mostly just runs the syntax scan
// FIXME: since these share a lot of output in common, what if we construct this
// to take another class as a parameter and then loop over that vector of embedded handlers?

