//! Test accessibility action routing

use accesskit::{ActionRequest, NodeId};
use iced_runtime::accessibility;

#[test]
fn test_action_request_structure() {
    // Verify that ActionRequest contains the fields we expect
    let request = ActionRequest {
        action: accesskit::Action::Click,
        target: NodeId(123),
        data: None,
    };

    assert_eq!(request.action, accesskit::Action::Click);
    assert_eq!(request.target, NodeId(123));
    assert!(request.data.is_none());
}

#[test]
fn test_accessibility_action_enum() {
    // Verify our accessibility Action enum works
    let action = accessibility::Action::ActionRequested(ActionRequest {
        action: accesskit::Action::Click,
        target: NodeId(456),
        data: None,
    });

    match action {
        accessibility::Action::ActionRequested(req) => {
            assert_eq!(req.target, NodeId(456));
        }
        _ => panic!("Expected ActionRequested variant"),
    }
}
