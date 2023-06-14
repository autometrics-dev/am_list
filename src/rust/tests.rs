use super::*;
use pretty_assertions::assert_eq;

#[test]
fn detect_single() {
    let source = r#"
        #[autometrics]
        fn main() {}
        "#;

    let list = list_function_names(String::new(), source).unwrap();

    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0],
        ExpectedAmLabel {
            module: String::new(),
            function: "main".into()
        }
    );
}

#[test]
fn detect_impl_block() {
    let source = r#"
        #[autometrics]
        struct Foo{};

        impl Foo {
            fn method_a() {}
        }
        "#;

    let list = list_function_names(String::new(), source).unwrap();

    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0],
        ExpectedAmLabel {
            module: "Foo".into(),
            function: "method_a".into()
        }
    );
}

#[test]
fn detect_struct_annotation() {
    let source = r#"
        #[autometrics]
        struct Foo{};
        "#;

    let list = list_struct_names(source).unwrap();

    assert_eq!(list.len(), 1);
    assert_eq!(list[0], "Foo");
}

#[test]
fn detect_trait_impl_block() {
    let source = r#"
        #[autometrics]
        struct Foo{};

        impl A for Foo {
            fn m_a() {}
        }
        "#;

    let list = list_function_names(String::new(), source).unwrap();

    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0],
        ExpectedAmLabel {
            module: "Foo".into(),
            function: "m_a".into()
        }
    );
}

#[test]
fn dodge_wrong_impl_block() {
    let source = r#"
        #[autometrics]
        struct Foo{};

        struct Bar{};

        impl Bar {
            fn method_one() {}
        }
        impl Foo {
            fn method_two() {}
        }
        impl Bar {
            fn method_three() {}
        }
        impl Foo {
            fn method_four() {}
        }
        "#;

    let list = list_function_names(String::new(), source).unwrap();

    assert_eq!(list.len(), 2);
    let method_two = ExpectedAmLabel {
        module: "Foo".into(),
        function: "method_two".into(),
    };
    assert!(
        list.contains(&method_two),
        "Expecting the list to contain {method_two:?}\nComplete list is {list:?}"
    );
    let method_four = ExpectedAmLabel {
        module: "Foo".into(),
        function: "method_four".into(),
    };
    assert!(
        list.contains(&method_four),
        "Expecting the list to contain {method_four:?}\nComplete list is {list:?}"
    );
}

#[test]
fn detect_inner_module() {
    let source = r#"
        mod inner{
            #[autometrics]
            fn inner_function() {}
        }

        mod well{
            mod nested {
                mod stuff {
                    #[autometrics]
                    fn hidden_function() {}
                }
           }
        }
        "#;

    let list = list_function_names(String::new(), source).unwrap();
    assert_eq!(
        list.len(),
        2,
        "Expected to find 2 items, instead the list is {list:?}"
    );
    let inner_fn = ExpectedAmLabel {
        module: "inner".into(),
        function: "inner_function".into(),
    };
    assert!(
        list.contains(&inner_fn),
        "Expecting the detected functions to contain {inner_fn:?}\nComplete list is {list:?}"
    );
    let nested_fn = ExpectedAmLabel {
        module: "well::nested::stuff".into(),
        function: "hidden_function".into(),
    };
    assert!(
        list.contains(&nested_fn),
        "Expecting the detected functions to contain {nested_fn:?}\nComplete list is {list:?}"
    );
}
