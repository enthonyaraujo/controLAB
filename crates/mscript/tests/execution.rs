use mscript::{
    CancellationToken, DiagnosticCategory, ExecutionEventKind as Event, ExecutionOptions,
    ExecutionStatus, Session, Value,
};
use std::time::Duration;

#[test]
fn prefixes_and_workspace_survive_later_semantic_errors_in_order() {
    let mut session = Session::new();
    let mut events = Vec::new();
    let options = ExecutionOptions {
        id: 42,
        ..Default::default()
    };
    let outcome =
        session.execute_with_events("a=1;disp(a);plot([0 1],[1 2]);missing", &options, |e| {
            events.push(e.clone());
            Ok(())
        });
    assert_eq!(outcome.status, ExecutionStatus::Failed);
    assert_eq!(
        outcome.diagnostic.unwrap().category,
        DiagnosticCategory::Semantic
    );
    assert_eq!(outcome.execution.outputs.len(), 1);
    assert_eq!(outcome.execution.plots.len(), 1);
    assert!(session.get("a").is_some());
    assert!(matches!(events[0].kind, Event::Started));
    assert!(matches!(events[1].kind, Event::WorkspaceChanged { .. }));
    assert!(matches!(events[2].kind, Event::Output(_)));
    assert!(matches!(events[3].kind, Event::Plot(_)));
    assert!(matches!(events[4].kind, Event::Error(_)));
    assert!(matches!(
        events[5].kind,
        Event::Finished {
            status: ExecutionStatus::Failed,
            ..
        }
    ));
    for (sequence, event) in events.iter().enumerate() {
        assert_eq!(event.id, 42);
        assert_eq!(event.sequence, sequence as u64);
    }
}
#[test]
fn syntax_errors_are_atomic_and_have_their_own_category() {
    let mut s = Session::new();
    let out = s.execute_with_events("a=1; b=[", &Default::default(), |_| Ok(()));
    assert_eq!(out.diagnostic.unwrap().category, DiagnosticCategory::Syntax);
    assert!(s.workspace().is_empty());
    assert!(s.execute("a=1").is_ok());
}
#[test]
fn work_and_allocation_limits_are_global_and_do_not_poison_next_runs() {
    let mut s = Session::new();
    s.execute("A=ones(4,4);").unwrap();
    let baseline = s.get("A").cloned();
    let mut opts = ExecutionOptions::default();
    opts.resources.work_units = 200;
    let out = s.execute_with_events("B=A*A;C=B*B;D=C*C;", &opts, |_| Ok(()));
    assert_eq!(out.diagnostic.unwrap().identifier, "mscript:WorkLimit");
    assert!(s.get("B").is_some());
    assert!(s.get("D").is_none());
    assert_eq!(s.get("A"), baseline.as_ref());
    opts.resources = Default::default();
    opts.resources.allocation_bytes = 1000;
    let out = s.execute_with_events("X=zeros(100,100);", &opts, |_| Ok(()));
    assert_eq!(
        out.diagnostic.unwrap().identifier,
        "mscript:AllocationLimit"
    );
    assert!(s.get("X").is_none());
    assert!(s.execute("X=2;").is_ok());
}
#[test]
fn cancellation_preserves_prefix_and_does_not_reach_later_requests() {
    let mut s = Session::new();
    let opts = ExecutionOptions::default();
    let token = opts.cancellation.clone();
    let out = s.execute_with_events("a=1;disp(a);b=ones(200,200);", &opts, |e| {
        if matches!(e.kind, Event::Output(_)) {
            token.cancel();
        }
        Ok(())
    });
    assert_eq!(out.status, ExecutionStatus::Cancelled);
    assert_eq!(out.execution.outputs.len(), 1);
    assert!(s.get("a").is_some());
    assert!(s.get("b").is_none());
    assert!(s.execute("b=3;").is_ok());
    let cancelled = ExecutionOptions {
        cancellation: {
            let t = CancellationToken::default();
            t.cancel();
            t
        },
        ..Default::default()
    };
    let out = s.clear_with_events(&cancelled, |_| Ok(()));
    assert_eq!(out.status, ExecutionStatus::Cancelled);
    assert!(s.get("b").is_some());
}
#[test]
fn workspace_limits_validate_all_output_bindings_before_commit() {
    let mut s = Session::new();
    s.execute("a=9;").unwrap();
    let opts = ExecutionOptions {
        max_variables: 2,
        ..Default::default()
    };
    let out = s.execute_with_events("[a,b,c]=size(ones(2,3));", &opts, |_| Ok(()));
    assert_eq!(out.diagnostic.unwrap().category, DiagnosticCategory::Limit);
    let Some(Value::Numeric(a)) = s.get("a") else {
        panic!()
    };
    assert_eq!(a.data(), &[9.]);
    assert!(s.get("b").is_none());
    let opts = ExecutionOptions {
        max_workspace_bytes: 64,
        ..Default::default()
    };
    let out = s.execute_with_events("a=ones(10,10);", &opts, |_| Ok(()));
    assert!(out.diagnostic.is_some());
    assert!(s.execute("clear;B=1;").is_ok());
}
#[test]
fn deadline_event_budget_and_delivery_failures_are_explicit() {
    let mut s = Session::new();
    let mut opts = ExecutionOptions::default();
    opts.resources.wall_time = Some(Duration::ZERO);
    let out = s.execute_with_events("a=1;", &opts, |_| Ok(()));
    assert_eq!(out.diagnostic.unwrap().identifier, "mscript:Deadline");
    assert!(s.get("a").is_none());
    let opts = ExecutionOptions {
        max_events: 1,
        ..Default::default()
    };
    let out = s.execute_with_events("a=1;b=2;", &opts, |_| Ok(()));
    assert_eq!(out.diagnostic.unwrap().category, DiagnosticCategory::Limit);
    let out = s.execute_with_events("c=1;", &Default::default(), |_| {
        Err(mscript::Diagnostic::transport("consumer disconnected"))
    });
    assert_eq!(
        out.diagnostic.unwrap().category,
        DiagnosticCategory::Transport
    );
    assert!(s.get("c").is_none());
}
