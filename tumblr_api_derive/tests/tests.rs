
use tumblr_api_derive::Builder;

#[test]
fn foo() {
    #[derive(Builder)]
    #[builder()]
    #[allow(unused)]
    struct Foo {
        #[builder(ctor())]
        a: u32,
    }
}
