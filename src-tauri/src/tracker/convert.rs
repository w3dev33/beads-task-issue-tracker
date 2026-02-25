//! Conversion functions between tracker types (TrackerIssue, TrackerComment, etc.)
//! and frontend-facing types (Issue, Comment, Relation, etc.) in lib.rs.

use super::issues::{CreateIssueParams, TrackerComment, TrackerIssue, TrackerRelation, UpdateIssueParams};
use crate::{
    normalize_issue_status, normalize_issue_type, ChildIssue, Comment, CreatePayload,
    Issue, ParentIssue, Relation, UpdatePayload,
};

/// Convert a TrackerIssue to the frontend-facing Issue struct.
pub fn tracker_issue_to_issue(ti: TrackerIssue) -> Issue {
    let parent = ti.parent.as_ref().map(|parent_id| ParentIssue {
        id: parent_id.clone(),
        title: String::new(),
        status: "open".to_string(),
        priority: "p3".to_string(),
    });

    let blocked_by = if ti.blocked_by.is_empty() {
        None
    } else {
        Some(ti.blocked_by)
    };

    let blocks = if ti.blocks.is_empty() {
        None
    } else {
        Some(ti.blocks)
    };

    let relations = if ti.relations.is_empty() {
        None
    } else {
        Some(
            ti.relations
                .into_iter()
                .map(tracker_relation_to_relation)
                .collect(),
        )
    };

    let comments: Vec<Comment> = ti
        .comments
        .into_iter()
        .map(tracker_comment_to_comment)
        .collect();

    Issue {
        id: ti.id,
        title: ti.title,
        description: ti.body,
        issue_type: normalize_issue_type(&ti.issue_type),
        status: normalize_issue_status(&ti.status),
        priority: ti.priority,
        assignee: ti.assignee,
        labels: ti.labels,
        created_at: ti.created_at,
        updated_at: ti.updated_at,
        closed_at: ti.closed_at,
        comments,
        blocked_by,
        blocks,
        external_ref: ti.external_ref,
        estimate_minutes: ti.estimate_minutes,
        design_notes: ti.design,
        acceptance_criteria: ti.acceptance_criteria,
        working_notes: ti.notes,
        parent,
        children: None, // Populated separately via list_children
        relations,
        metadata: ti.metadata,
        spec_id: ti.spec_id,
        comment_count: Some(ti.comment_count),
        dependency_count: Some(ti.dependency_count),
        dependent_count: Some(ti.dependent_count),
    }
}

/// Convert a TrackerIssue to a ChildIssue (lightweight).
pub fn tracker_issue_to_child(ti: &TrackerIssue) -> ChildIssue {
    ChildIssue {
        id: ti.id.clone(),
        title: ti.title.clone(),
        status: normalize_issue_status(&ti.status),
        priority: ti.priority.clone(),
    }
}

fn tracker_comment_to_comment(tc: TrackerComment) -> Comment {
    Comment {
        id: tc.id,
        author: tc.author,
        content: tc.body,
        created_at: tc.created_at,
    }
}

fn tracker_relation_to_relation(tr: TrackerRelation) -> Relation {
    Relation {
        id: tr.id,
        title: String::new(),
        status: String::new(),
        priority: String::new(),
        relation_type: tr.dep_type,
        direction: tr.direction,
    }
}

/// Convert a CreatePayload (frontend) to CreateIssueParams (tracker engine).
pub fn create_payload_to_params(p: &CreatePayload) -> CreateIssueParams {
    CreateIssueParams {
        title: p.title.clone(),
        body: p.description.clone(),
        issue_type: p.issue_type.clone(),
        status: None,
        priority: p.priority.clone(),
        assignee: p.assignee.clone(),
        author: None,
        labels: p.labels.clone(),
        external_ref: p.external_ref.clone(),
        estimate_minutes: p.estimate_minutes,
        design: p.design_notes.clone(),
        acceptance_criteria: p.acceptance_criteria.clone(),
        notes: p.working_notes.clone(),
        parent: p.parent.clone(),
        metadata: None,
        spec_id: p.spec_id.clone(),
    }
}

/// Convert an UpdatePayload (frontend) to UpdateIssueParams (tracker engine).
///
/// The frontend uses flat `Option<T>` for nullable fields (with empty string
/// meaning "clear"). The tracker engine uses `Option<Option<T>>` where
/// `None` = don't touch, `Some(None)` = clear, `Some(Some(v))` = set.
pub fn update_payload_to_params(u: &UpdatePayload) -> UpdateIssueParams {
    UpdateIssueParams {
        title: u.title.clone(),
        body: u.description.clone(),
        issue_type: u.issue_type.clone(),
        status: u.status.clone(),
        priority: u.priority.clone(),
        assignee: u.assignee.as_ref().map(|a| {
            if a.is_empty() { None } else { Some(a.clone()) }
        }),
        labels: u.labels.clone(),
        external_ref: u.external_ref.as_ref().map(|e| {
            if e.is_empty() { None } else { Some(e.clone()) }
        }),
        estimate_minutes: u.estimate_minutes.map(|e| {
            if e == 0 { None } else { Some(e) }
        }),
        design: u.design_notes.as_ref().map(|d| {
            if d.is_empty() { None } else { Some(d.clone()) }
        }),
        acceptance_criteria: u.acceptance_criteria.as_ref().map(|a| {
            if a.is_empty() { None } else { Some(a.clone()) }
        }),
        notes: u.working_notes.as_ref().map(|n| {
            if n.is_empty() { None } else { Some(n.clone()) }
        }),
        parent: u.parent.as_ref().map(|p| {
            if p.is_empty() { None } else { Some(p.clone()) }
        }),
        metadata: u.metadata.as_ref().map(|m| {
            if m.is_empty() { None } else { Some(m.clone()) }
        }),
        spec_id: u.spec_id.as_ref().map(|s| {
            if s.is_empty() { None } else { Some(s.clone()) }
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tracker_issue() -> TrackerIssue {
        TrackerIssue {
            id: "test-abc".to_string(),
            title: "Test issue".to_string(),
            body: "Description here".to_string(),
            issue_type: "bug".to_string(),
            status: "open".to_string(),
            priority: "p1".to_string(),
            assignee: Some("alice".to_string()),
            author: "bob".to_string(),
            created_at: "2024-01-01 00:00:00".to_string(),
            updated_at: "2024-01-02 00:00:00".to_string(),
            closed_at: None,
            external_ref: None,
            estimate_minutes: Some(60),
            design: Some("Design doc".to_string()),
            acceptance_criteria: Some("It works".to_string()),
            notes: Some("Working notes".to_string()),
            parent: Some("test-parent".to_string()),
            metadata: None,
            spec_id: None,
            labels: vec!["bug".to_string(), "urgent".to_string()],
            comments: vec![TrackerComment {
                id: "c-123".to_string(),
                body: "Hello world".to_string(),
                author: "charlie".to_string(),
                created_at: "2024-01-01 12:00:00".to_string(),
            }],
            blocked_by: vec!["test-dep1".to_string()],
            blocks: vec!["test-dep2".to_string()],
            relations: vec![TrackerRelation {
                id: "test-rel1".to_string(),
                dep_type: "relates-to".to_string(),
                direction: "dependent".to_string(),
            }],
            comment_count: 1,
            dependency_count: 1,
            dependent_count: 1,
        }
    }

    #[test]
    fn test_tracker_issue_to_issue_field_mapping() {
        let ti = make_tracker_issue();
        let issue = tracker_issue_to_issue(ti);

        assert_eq!(issue.id, "test-abc");
        assert_eq!(issue.title, "Test issue");
        assert_eq!(issue.description, "Description here"); // body → description
        assert_eq!(issue.issue_type, "bug");
        assert_eq!(issue.status, "open");
        assert_eq!(issue.priority, "p1");
        assert_eq!(issue.assignee, Some("alice".to_string()));
        assert_eq!(issue.design_notes, Some("Design doc".to_string())); // design → design_notes
        assert_eq!(issue.working_notes, Some("Working notes".to_string())); // notes → working_notes
        assert_eq!(issue.estimate_minutes, Some(60));
    }

    #[test]
    fn test_tracker_issue_to_issue_parent() {
        let ti = make_tracker_issue();
        let issue = tracker_issue_to_issue(ti);

        let parent = issue.parent.unwrap();
        assert_eq!(parent.id, "test-parent");
        assert_eq!(parent.title, ""); // stub
    }

    #[test]
    fn test_tracker_issue_to_issue_comments() {
        let ti = make_tracker_issue();
        let issue = tracker_issue_to_issue(ti);

        assert_eq!(issue.comments.len(), 1);
        assert_eq!(issue.comments[0].content, "Hello world"); // body → content
        assert_eq!(issue.comments[0].author, "charlie");
    }

    #[test]
    fn test_tracker_issue_to_issue_deps() {
        let ti = make_tracker_issue();
        let issue = tracker_issue_to_issue(ti);

        assert_eq!(issue.blocked_by, Some(vec!["test-dep1".to_string()]));
        assert_eq!(issue.blocks, Some(vec!["test-dep2".to_string()]));
    }

    #[test]
    fn test_empty_vecs_become_none() {
        let mut ti = make_tracker_issue();
        ti.blocked_by = vec![];
        ti.blocks = vec![];
        ti.relations = vec![];

        let issue = tracker_issue_to_issue(ti);

        assert!(issue.blocked_by.is_none());
        assert!(issue.blocks.is_none());
        assert!(issue.relations.is_none());
    }

    #[test]
    fn test_tracker_issue_to_issue_relations() {
        let ti = make_tracker_issue();
        let issue = tracker_issue_to_issue(ti);

        let relations = issue.relations.unwrap();
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].id, "test-rel1");
        assert_eq!(relations[0].relation_type, "relates-to");
        assert_eq!(relations[0].direction, "dependent");
    }

    #[test]
    fn test_create_payload_to_params() {
        let payload = CreatePayload {
            title: "New issue".to_string(),
            description: Some("A description".to_string()),
            issue_type: Some("bug".to_string()),
            priority: Some("p0".to_string()),
            assignee: Some("alice".to_string()),
            labels: Some(vec!["bug".to_string()]),
            external_ref: None,
            estimate_minutes: Some(30),
            design_notes: Some("Design".to_string()),
            acceptance_criteria: Some("Works".to_string()),
            working_notes: Some("Notes".to_string()),
            parent: Some("parent-id".to_string()),
            spec_id: None,
            cwd: None,
        };

        let params = create_payload_to_params(&payload);

        assert_eq!(params.title, "New issue");
        assert_eq!(params.body, Some("A description".to_string())); // description → body
        assert_eq!(params.issue_type, Some("bug".to_string()));
        assert_eq!(params.priority, Some("p0".to_string()));
        assert_eq!(params.design, Some("Design".to_string())); // design_notes → design
        assert_eq!(params.notes, Some("Notes".to_string())); // working_notes → notes
    }

    #[test]
    fn test_update_payload_to_params() {
        let payload = UpdatePayload {
            title: Some("Updated".to_string()),
            description: Some("New desc".to_string()),
            issue_type: None,
            status: Some("in_progress".to_string()),
            priority: None,
            assignee: Some("".to_string()), // empty = clear
            labels: None,
            external_ref: None,
            estimate_minutes: None,
            design_notes: Some("Design".to_string()),
            acceptance_criteria: None,
            working_notes: Some("".to_string()), // empty = clear
            parent: None,
            metadata: None,
            spec_id: None,
            cwd: None,
        };

        let params = update_payload_to_params(&payload);

        assert_eq!(params.title, Some("Updated".to_string()));
        assert_eq!(params.body, Some("New desc".to_string()));
        assert!(params.issue_type.is_none()); // not provided
        assert_eq!(params.status, Some("in_progress".to_string()));
        assert_eq!(params.assignee, Some(None)); // empty string → clear
        assert_eq!(params.design, Some(Some("Design".to_string())));
        assert_eq!(params.notes, Some(None)); // empty string → clear
    }

    #[test]
    fn test_tracker_issue_to_child() {
        let ti = make_tracker_issue();
        let child = tracker_issue_to_child(&ti);

        assert_eq!(child.id, "test-abc");
        assert_eq!(child.title, "Test issue");
        assert_eq!(child.status, "open");
        assert_eq!(child.priority, "p1");
    }
}
