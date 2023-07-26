use super::*;

const DUMMY_MODULE: &str = "dummy";

#[test]
fn detect_simple() {
    let source = r#"
        from autometrics import autometrics

        @autometrics
        def the_one():
            return 'wake up, Neo'
        "#;

    let import_query = AmImportQuery::try_new().unwrap();
    let import_name = import_query.get_decorator_name(source).unwrap();
    let query = AmQuery::try_new(import_name.as_str()).unwrap();
    let list = query.list_function_names(source, DUMMY_MODULE).unwrap();
    let all_query = AllFunctionsQuery::try_new().unwrap();
    let all_list = all_query.list_function_names(source, DUMMY_MODULE).unwrap();

    let the_one = ExpectedAmLabel {
        module: "dummy".to_string(),
        function: "the_one".to_string(),
    };

    assert_eq!(list.len(), 1);
    assert_eq!(list[0], the_one);
    assert_eq!(all_list, list);
}

#[test]
fn detect_alias() {
    let source = r#"
        from autometrics import autometrics as am

        @am
        def the_one():
            return 'wake up, Neo'
        "#;

    let import_query = AmImportQuery::try_new().unwrap();
    let import_name = import_query.get_decorator_name(source).unwrap();
    let query = AmQuery::try_new(import_name.as_str()).unwrap();
    let list = query.list_function_names(source, DUMMY_MODULE).unwrap();
    let all_query = AllFunctionsQuery::try_new().unwrap();
    let all_list = all_query.list_function_names(source, DUMMY_MODULE).unwrap();

    let the_one = ExpectedAmLabel {
        module: "dummy".to_string(),
        function: "the_one".to_string(),
    };

    assert_eq!(list.len(), 1);
    assert_eq!(list[0], the_one);
    assert_eq!(all_list, list);
}

#[test]
fn detect_nested() {
    let source = r#"
        from autometrics import autometrics

        @autometrics
        def the_one():
            @autometrics
            def the_two():
                return 'wake up, Neo'
            return the_two()
        "#;

    let import_query = AmImportQuery::try_new().unwrap();
    let import_name = import_query.get_decorator_name(source).unwrap();
    let query = AmQuery::try_new(import_name.as_str()).unwrap();
    let list = query.list_function_names(source, DUMMY_MODULE).unwrap();
    let all_query = AllFunctionsQuery::try_new().unwrap();
    let all_list = all_query.list_function_names(source, DUMMY_MODULE).unwrap();

    let the_one = ExpectedAmLabel {
        module: "dummy".to_string(),
        function: "the_one".to_string(),
    };
    let the_two = ExpectedAmLabel {
        module: "dummy".to_string(),
        function: "the_one.<locals>.the_two".to_string(),
    };

    assert_eq!(list.len(), 2);
    assert_eq!(list[0], the_one);
    assert_eq!(list[1], the_two);
    assert_eq!(all_list, list);
}
