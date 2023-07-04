use super::*;
use pretty_assertions::assert_eq;

#[test]
fn detect_single() {
    let source = r#"
        #[autometrics]
        fn main() {}
        "#;

    let list = AmQuery::try_new()
        .unwrap()
        .list_function_names(String::new(), source)
        .unwrap();

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
        struct Foo{};

        #[autometrics]
        impl Foo {
            fn method_a() {}
        }
        "#;

    let list = AmQuery::try_new()
        .unwrap()
        .list_function_names(String::new(), source)
        .unwrap();

    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0],
        ExpectedAmLabel {
            module: String::new(),
            function: "Foo::method_a".into()
        }
    );
}

#[test]
fn detect_trait_impl_block() {
    let source = r#"
        struct Foo{};

        #[autometrics]
        impl A for Foo {
            fn m_a() {}
        }
        "#;

    let list = AmQuery::try_new()
        .unwrap()
        .list_function_names(String::new(), source)
        .unwrap();

    assert_eq!(list.len(), 1);
    assert_eq!(
        list[0],
        ExpectedAmLabel {
            module: String::new(),
            function: "Foo::m_a".into()
        }
    );
}

#[test]
fn dodge_wrong_impl_block() {
    let source = r#"
        struct Foo{};

        struct Bar{};

        impl Bar {
            fn method_one() {}
        }
        #[autometrics]
        impl Foo {
            fn method_two() {}
        }
        impl Bar {
            fn method_three() {}
        }
        #[autometrics]
        impl Foo {
            fn method_four() {}
        }
        "#;

    let list = AmQuery::try_new()
        .unwrap()
        .list_function_names(String::new(), source)
        .unwrap();
    let all = AllFunctionsQuery::try_new()
        .unwrap()
        .list_function_names(String::new(), source)
        .unwrap();

    let method_one = ExpectedAmLabel {
        module: String::new(),
        function: "Bar::method_one".into(),
    };
    let method_two = ExpectedAmLabel {
        module: String::new(),
        function: "Foo::method_two".into(),
    };
    let method_three = ExpectedAmLabel {
        module: String::new(),
        function: "Bar::method_three".into(),
    };
    let method_four = ExpectedAmLabel {
        module: String::new(),
        function: "Foo::method_four".into(),
    };

    assert_eq!(list.len(), 2);
    assert!(
        list.contains(&method_two),
        "Expecting the list to contain {method_two:?}\nComplete list is {list:?}"
    );
    assert!(
        list.contains(&method_four),
        "Expecting the list to contain {method_four:?}\nComplete list is {list:?}"
    );

    assert_eq!(all.len(), 4, "Complete list is {all:?}");
    assert!(
        all.contains(&method_one),
        "Expecting the list to contain {method_one:?}\nComplete list is {all:?}"
    );
    assert!(
        all.contains(&method_two),
        "Expecting the list to contain {method_two:?}\nComplete list is {all:?}"
    );
    assert!(
        all.contains(&method_three),
        "Expecting the list to contain {method_three:?}\nComplete list is {all:?}"
    );
    assert!(
        all.contains(&method_four),
        "Expecting the list to contain {method_four:?}\nComplete list is {all:?}"
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

    let list = AmQuery::try_new()
        .unwrap()
        .list_function_names(String::new(), source)
        .unwrap();
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

#[test]
fn detect_partially_annotated_impl_block() {
    let source = r#"
        struct Foo{};

        impl A for Foo {
            fn nothing_to_see_here() {}

            #[autometrics]
            fn m_a() {}
        }
        "#;

    let list = AmQuery::try_new()
        .unwrap()
        .list_function_names(String::new(), source)
        .unwrap();
    let all = AllFunctionsQuery::try_new()
        .unwrap()
        .list_function_names(String::new(), source)
        .unwrap();

    let m_a = ExpectedAmLabel {
        module: String::new(),
        function: "Foo::m_a".into(),
    };

    let dummy = ExpectedAmLabel {
        module: String::new(),
        function: "Foo::nothing_to_see_here".into(),
    };

    assert_eq!(list.len(), 1, "Complete list is {list:?}");
    assert!(
        list.contains(&m_a),
        "Expecting the list to contain {m_a:?}\nComplete list is {list:?}"
    );

    assert_eq!(all.len(), 2);
    assert!(
        all.contains(&m_a),
        "Expecting the list to contain {m_a:?}\nComplete list is {all:?}"
    );
    assert!(
        all.contains(&dummy),
        "Expecting the list to contain {dummy:?}\nComplete list is {all:?}"
    );
}
