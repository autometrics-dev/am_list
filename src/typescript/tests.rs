use std::path::PathBuf;

use super::{
    imports::{CanonicalSource, Identifier},
    queries::ImportsMapQuery,
    *,
};

#[test]
fn detect_simple() {
    let source = r#"
import express from "express";
import { autometrics } from "@autometrics/autometrics";

const app = express();
const port = 8080;

function resolveAfterHalfSecond(): Promise<string> {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve("Function resolved");
    }, 500);
  });
}

const asyncCallMetricized = autometrics(async function asyncCall() {
  console.log("Calling async function");
  return await resolveAfterHalfSecond();
});
        "#;

    let list = AmQuery::try_new()
        .unwrap()
        .list_function_names("", source, None)
        .unwrap();
    let all = AllFunctionsQuery::try_new()
        .unwrap()
        .list_function_names("", source)
        .unwrap();
    let resolve_after_half = ExpectedAmLabel {
        module: String::new(),
        function: "resolveAfterHalfSecond".to_string(),
    };
    let async_call = ExpectedAmLabel {
        module: String::new(),
        function: "asyncCall".to_string(),
    };

    assert_eq!(
        list.len(),
        1,
        "list should have 1 item, got this instead: {list:?}"
    );
    assert_eq!(list[0], async_call);

    assert_eq!(
        all.len(),
        2,
        "list of all functions should have 2 items, got this instead: {all:?}"
    );
    assert!(
        all.contains(&async_call),
        "List of all functions should contain {async_call:?}; complete list is {all:?}"
    );
    assert!(
        all.contains(&resolve_after_half),
        "List of all functions should contain {resolve_after_half:?}; complete list is {all:?}"
    );
}

#[test]
fn detect_inner_route() {
    let source = r#"
import express from "express";
import { autometrics } from "@autometrics/autometrics";

const app = express();

app.get("/", rootRoute);
app.get("/bad", autometrics(badRoute));
app.get("/async", autometrics(asyncRoute));
        "#;

    let list = AmQuery::try_new()
        .unwrap()
        .list_function_names("", source, None)
        .unwrap();
    let all = AllFunctionsQuery::try_new()
        .unwrap()
        .list_function_names("", source)
        .unwrap();
    // Should `list` track the origin of badRoute and asyncRoute??
    let bad_route = ExpectedAmLabel {
        module: String::new(),
        function: "badRoute".to_string(),
    };
    let async_route = ExpectedAmLabel {
        module: String::new(),
        function: "asyncRoute".to_string(),
    };

    assert_eq!(
        list.len(),
        2,
        "list should have 2 items, got this instead: {list:?}"
    );
    // In this example, as no function is _defined_ in the source code, we actually have
    // an empty list for "all functions"
    assert_eq!(
        all.len(),
        0,
        "list of all functions should have 2 items, got this instead: {all:?}"
    );

    assert!(
        list.contains(&bad_route),
        "The list should contain {bad_route:?}; complete list is {list:?}"
    );
    assert!(
        list.contains(&async_route),
        "The list should contain {async_route:?}; complete list is {list:?}"
    );
}

#[test]
fn detect_class() {
    let source = r#"
import express from "express";

@Autometrics
class Foo {
    x: number
    constructor(x = 0) {
        this.x = x;
    }
    method_b(): string {
        return "you win";
    }
}

class NotGood {
    x: string
    constructor(x = "got you") {
        this.x = x;
    }
    gotgot(): string {
        return "!";
    }
}
        "#;

    let list = AmQuery::try_new()
        .unwrap()
        .list_function_names("", source, None)
        .unwrap();
    let all = AllFunctionsQuery::try_new()
        .unwrap()
        .list_function_names("", source)
        .unwrap();
    let foo_constructor = ExpectedAmLabel {
        module: String::new(),
        function: "Foo.constructor".to_string(),
    };
    let method_b = ExpectedAmLabel {
        module: String::new(),
        function: "Foo.method_b".to_string(),
    };
    let not_good_constructor = ExpectedAmLabel {
        module: String::new(),
        function: "NotGood.constructor".to_string(),
    };
    let gotgot_method = ExpectedAmLabel {
        module: String::new(),
        function: "NotGood.gotgot".to_string(),
    };

    assert_eq!(
        list.len(),
        2,
        "list should have 2 items, got this instead: {list:?}"
    );
    assert_eq!(
        all.len(),
        4,
        "list of all functions should have 4 items, got this instead: {all:?}"
    );

    assert!(
        list.contains(&foo_constructor),
        "The list should contain {foo_constructor:?}; complete list is {list:?}"
    );
    assert!(
        list.contains(&method_b),
        "The list should contain {method_b:?}; complete list is {list:?}"
    );

    assert!(
        all.contains(&foo_constructor),
        "The list of all functions should contain {foo_constructor:?}; complete list is {all:?}"
    );
    assert!(
        all.contains(&method_b),
        "The list of all functions should contain {method_b:?}; complete list is {all:?}"
    );
    assert!(
        all.contains(&not_good_constructor),
        "The list of all functions should contain {not_good_constructor:?}; complete list is {all:?}"
    );
    assert!(
        all.contains(&gotgot_method),
        "The list of all functions should contain {gotgot_method:?}; complete list is {all:?}"
    );
}

#[test]
fn compute_import_map() {
    let source = r#"
import { exec } from 'child_process'
import { anyRoute as myRoute } from './handlers'
import * as other from '../other'
import { autometrics } from '@autometrics/autometrics'

const instrumentedExec = autometrics(exec);
const instrumentedRoute = autometrics(myRoute);
const instrumentedOther = autometrics(other.stuff);
        "#;

    let imports_query = ImportsMapQuery::try_new().expect("can build the imports map query");
    let imports_map = imports_query
        .list_imports(Some(&PathBuf::try_from("src/").unwrap()), source)
        .expect("can build the imports map from a query");

    let other_import = CanonicalSource::from("sibling://other");
    let exec_import = (
        Identifier::from("exec"),
        CanonicalSource::from("ext://child_process"),
    );
    let route_import = (
        Identifier::from("anyRoute"),
        CanonicalSource::from("src/handlers"),
    );
    let autometrics_import = (
        Identifier::from("autometrics"),
        CanonicalSource::from("ext://@autometrics/autometrics"),
    );

    assert_eq!(
        imports_map
            .find_namespace(&Identifier::from("other"))
            .unwrap(),
        other_import
    );
    assert_eq!(
        imports_map
            .find_identifier(&Identifier::from("exec"))
            .unwrap(),
        exec_import
    );
    assert_eq!(
        imports_map
            .find_identifier(&Identifier::from("myRoute"))
            .unwrap(),
        route_import
    );
    assert_eq!(
        imports_map
            .find_identifier(&Identifier::from("autometrics"))
            .unwrap(),
        autometrics_import
    );
}

#[test]
fn detect_imported_functions() {
    let source = r#"
import { exec } from 'child_process'
import { anyRoute as myRoute } from './handlers'
import * as other from '../other'
import { autometrics } from '@autometrics/autometrics'

const instrumentedExec = autometrics(exec);
const instrumentedRoute = autometrics(myRoute);
const instrumentedOther = autometrics(other.stuff);
        "#;

    let list = AmQuery::try_new()
        .unwrap()
        .list_function_names("", source, Some(&PathBuf::from("src/")))
        .unwrap();
    let all = AllFunctionsQuery::try_new()
        .unwrap()
        .list_function_names("", source)
        .unwrap();

    let exec = ExpectedAmLabel {
        module: "ext://child_process".to_string(),
        function: "exec".to_string(),
    };
    let any_route = ExpectedAmLabel {
        module: "src/handlers".to_string(),
        function: "anyRoute".to_string(),
    };
    let stuff = ExpectedAmLabel {
        module: "sibling://other".to_string(),
        function: "stuff".to_string(),
    };

    assert_eq!(
        list.len(),
        3,
        "list should have 3 items, got this instead: {list:?}"
    );
    assert!(
        list.contains(&exec),
        "List of instrumented functions should contain {exec:?}. Complete list is {list:?}"
    );
    assert!(
        list.contains(&any_route),
        "List of instrumented functions should contain {any_route:?}. Complete list is {list:?}"
    );
    assert!(
        list.contains(&stuff),
        "List of instrumented functions should contain {stuff:?}. Complete list is {list:?}"
    );

    assert!(
        all.is_empty(),
        "the complete list of functions should be empty, nothing is defined in this file. Got this instead: {all:?}"
    );
}

#[test]
fn detect_two_args_wrapper() {
    let source = r#"
  import { autometrics } from "autometrics";

  const getWow = autometrics(
    {
      functionName: "getThatWow",
      moduleName: "MODULE",
    },
    async () => {
      const res = await fetch(
        "https://owen-wilson-wow-api.onrender.com/wows/random"
      );
      return await res.json();
    }
  );
        "#;

    let list = AmQuery::try_new()
        .unwrap()
        .list_function_names("", source, None)
        .unwrap();
    let all = AllFunctionsQuery::try_new()
        .unwrap()
        .list_function_names("", source)
        .unwrap();
    let get_wow = ExpectedAmLabel {
        module: "MODULE".to_string(),
        function: "getThatWow".to_string(),
    };

    assert_eq!(
        list.len(),
        1,
        "list should have 1 item, got this instead: {list:?}"
    );
    assert_eq!(list[0], get_wow);

    assert_eq!(
        all.len(),
        0,
        "list of all functions should have 0 items, got this instead: {all:?}"
    );
}
