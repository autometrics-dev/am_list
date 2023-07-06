use super::*;

#[test]
fn detect_simple() {
    let source = r#"
        package lambda

        //autometrics:inst
        func the_one() {
        	return nil
        }
        "#;

    let query = AmQuery::try_new().unwrap();
    let list = query.list_function_names(source).unwrap();
    let all_query = AllFunctionsQuery::try_new().unwrap();
    let all_list = all_query.list_function_names(source).unwrap();

    let the_one = ExpectedAmLabel {
        module: "lambda".to_string(),
        function: "the_one".to_string(),
    };

    assert_eq!(list.len(), 1);
    assert_eq!(list[0], the_one);
    assert_eq!(all_list, list);
}

#[test]
fn detect_legacy() {
    let source = r#"
        package beta

        func not_the_one() {
        }

        //autometrics:doc
        func sandwiched_function() {
        	return nil
        }

        func not_that_one_either() {
        }
        "#;

    let query = AmQuery::try_new().unwrap();
    let list = query.list_function_names(source).unwrap();
    let all_query = AllFunctionsQuery::try_new().unwrap();
    let all_list = all_query.list_function_names(source).unwrap();

    let sandwiched = ExpectedAmLabel {
        module: "beta".to_string(),
        function: "sandwiched_function".to_string(),
    };
    let not_the_one = ExpectedAmLabel {
        module: "beta".to_string(),
        function: "not_the_one".to_string(),
    };
    let not_that_one = ExpectedAmLabel {
        module: "beta".to_string(),
        function: "not_that_one_either".to_string(),
    };

    assert_eq!(list.len(), 1);
    assert_eq!(list[0], sandwiched);

    assert_eq!(
        all_list.len(),
        3,
        "complete functions list should have 3 items, got {} instead: {all_list:?}",
        all_list.len()
    );
    assert!(all_list.contains(&sandwiched));
    assert!(all_list.contains(&not_the_one));
    assert!(all_list.contains(&not_that_one));
}
