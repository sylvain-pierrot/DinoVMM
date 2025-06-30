extern crate kvm_ioctls;
extern crate kvm_bindings;

use kvm_ioctls::Kvm;


fn main() {
    println!("Hello, world!");

    let kvm = Kvm::new().unwrap();

    println!("{}", kvm.get_api_version());
    println!("{:?}", kvm);
}
