// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(associated_consts)]
#![feature(conservative_impl_trait)]
#![feature(decl_macro)]
#![allow(warnings)]

mod m {
    fn priv_fn() {}
    enum PrivEnum { Variant }
    pub enum PubEnum { Variant }
    trait PrivTrait { fn method() {} }
    impl PrivTrait for u8 {}
    pub trait PubTrait { fn method() {} }
    impl PubTrait for u8 {}
    struct PrivTupleStruct(u8);
    pub struct PubTupleStruct(u8);
    impl PubTupleStruct { fn method() {} }

    struct Priv;
    pub type Alias = Priv;
    pub struct Pub<T = Alias>(pub T);

    impl Pub<Priv> {
        pub fn static_method() {}
        pub const INHERENT_ASSOC_CONST: u8 = 0;
    }
    impl<T> Pub<T> {
        pub fn static_method_generic_self() {}
        pub const INHERENT_ASSOC_CONST_GENERIC_SELF: u8 = 0;
    }
    impl Pub<u8> {
        fn priv_method(&self) {}
        pub fn method_with_substs<T>(&self) {}
        pub fn method_with_priv_params(&self, _: Priv) {}
    }
    impl TraitWithAssocConst for Priv {}
    impl TraitWithAssocTy for Priv { type AssocTy = u8; }

    pub macro m() {
        priv_fn; //~ ERROR type `fn() {m::priv_fn}` is private
        PrivEnum::Variant; //~ ERROR type `m::PrivEnum` is private
        PubEnum::Variant; // OK
        <u8 as PrivTrait>::method; //~ ERROR type `fn() {<u8 as m::PrivTrait>::method}` is private
        <u8 as PubTrait>::method; // OK
        PrivTupleStruct;
        //~^ ERROR type `fn(u8) -> m::PrivTupleStruct {m::PrivTupleStruct::{{constructor}}}` is priv
        PubTupleStruct;
        //~^ ERROR type `fn(u8) -> m::PubTupleStruct {m::PubTupleStruct::{{constructor}}}` is privat
        Pub(0u8).priv_method();
        //~^ ERROR type `fn(&m::Pub<u8>) {<m::Pub<u8>>::priv_method}` is private
    }

    trait Trait {}
    pub trait TraitWithTyParam<T> {}
    pub trait TraitWithTyParam2<T> { fn pub_method() {} }
    pub trait TraitWithAssocTy { type AssocTy; }
    pub trait TraitWithAssocConst { const TRAIT_ASSOC_CONST: u8 = 0; }
    impl Trait for u8 {}
    impl<T> TraitWithTyParam<T> for u8 {}
    impl TraitWithTyParam2<Priv> for u8 {}
    impl TraitWithAssocTy for u8 { type AssocTy = Priv; }

    pub fn leak_anon1() -> impl Trait + 'static { 0 }
    pub fn leak_anon2() -> impl TraitWithTyParam<Alias> { 0 }
    pub fn leak_anon3() -> impl TraitWithAssocTy<AssocTy = Alias> { 0 }

    pub fn leak_dyn1() -> Box<Trait + 'static> { Box::new(0) }
    pub fn leak_dyn2() -> Box<TraitWithTyParam<Alias>> { Box::new(0) }
    pub fn leak_dyn3() -> Box<TraitWithAssocTy<AssocTy = Alias>> { Box::new(0) }
}

mod adjust {
    // Construct a chain of derefs with a private type in the middle
    use std::ops::Deref;

    pub struct S1;
    struct S2;
    pub type S2Alias = S2;
    pub struct S3;

    impl Deref for S1 {
        type Target = S2Alias;
        fn deref(&self) -> &Self::Target { loop {} }
    }
    impl Deref for S2 {
        type Target = S3;
        fn deref(&self) -> &Self::Target { loop {} }
    }

    impl S3 {
        pub fn method_s3(&self) {}
    }
}

fn main() {
    let _: m::Alias; //~ ERROR type `m::Priv` is private
                     //~^ ERROR type `m::Priv` is private
    let _: <m::Alias as m::TraitWithAssocTy>::AssocTy; //~ ERROR type `m::Priv` is private
    m::Alias {}; //~ ERROR type `m::Priv` is private
    m::Pub { 0: m::Alias {} }; //~ ERROR type `m::Priv` is private
    m::Pub { 0: loop {} }; // OK, `m::Pub` is in value context, so it means Pub<_>, not Pub<Priv>
    m::Pub::static_method; //~ ERROR type `m::Priv` is private
    m::Pub::INHERENT_ASSOC_CONST; //~ ERROR type `m::Priv` is private
    m::Pub(0u8).method_with_substs::<m::Alias>(); //~ ERROR type `m::Priv` is private
    m::Pub(0u8).method_with_priv_params(loop{}); //~ ERROR type `m::Priv` is private
    <m::Alias as m::TraitWithAssocConst>::TRAIT_ASSOC_CONST; //~ ERROR type `m::Priv` is private
    <m::Pub<m::Alias>>::INHERENT_ASSOC_CONST; //~ ERROR type `m::Priv` is private
    <m::Pub<m::Alias>>::INHERENT_ASSOC_CONST_GENERIC_SELF; //~ ERROR type `m::Priv` is private
    <m::Pub<m::Alias>>::static_method_generic_self; //~ ERROR type `m::Priv` is private
    use m::TraitWithTyParam2;
    u8::pub_method; //~ ERROR type `m::Priv` is private

    adjust::S1.method_s3(); //~ ERROR type `adjust::S2` is private

    m::m!();

    m::leak_anon1(); //~ ERROR trait `m::Trait` is private
    m::leak_anon2(); //~ ERROR type `m::Priv` is private
    m::leak_anon3(); //~ ERROR type `m::Priv` is private

    m::leak_dyn1(); //~ ERROR type `m::Trait + 'static` is private
    m::leak_dyn2(); //~ ERROR type `m::Priv` is private
    m::leak_dyn3(); //~ ERROR type `m::Priv` is private

    // Check that messages are not duplicated for various kinds of assignments
    let a = m::Alias {}; //~ ERROR type `m::Priv` is private
    let mut b = a; //~ ERROR type `m::Priv` is private
    b = a; //~ ERROR type `m::Priv` is private
    match a { //~ ERROR type `m::Priv` is private
        _ => {}
    }
}
