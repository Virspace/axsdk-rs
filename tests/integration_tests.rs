extern crate raxsdk;

use raxsdk::foundation::*;

#[test]
fn kba() {
    //assert_eq!(raxsdk::foo::add(2, 2), 4);
    assert_eq!(1024, axtypes::kb(1));
}