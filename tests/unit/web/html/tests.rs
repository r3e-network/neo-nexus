//! Unit coverage for the HTML helpers every page depends on. The escaping and
//! the query round trip are the two places where operator data reaches a
//! browser, so both are pinned here rather than only through HTTP tests.

use crate::web::html;

#[test]
fn escape_neutralises_every_markup_character() {
    // `=` is left alone on purpose: the escaper only covers the five
    // characters that can open or terminate markup.
    assert_eq!(
        html::escape(r#"<img src=x onerror="alert('hi')">&"#),
        "&lt;img src=x onerror=&quot;alert(&#39;hi&#39;)&quot;&gt;&amp;"
    );
}

#[test]
fn escape_leaves_ordinary_operator_text_intact() {
    assert_eq!(
        html::escape("seed-1 running on 10.0.0.5:10332"),
        "seed-1 running on 10.0.0.5:10332"
    );
}

#[test]
fn status_badge_labels_itself_for_the_polling_script() {
    let running = html::status_badge("Running");
    assert!(running.contains(r#"class="badge running""#), "{running}");
    assert!(
        running.contains(r#"data-node-status="Running""#),
        "{running}"
    );
    assert!(html::status_badge("Deploying").contains(r#"class="badge stopped""#));
}

#[test]
fn encoding_keeps_unreserved_and_escapes_the_rest() {
    assert_eq!(html::urlencoding_lite("node-1_2.0~a"), "node-1_2.0~a");
    assert_eq!(html::urlencoding_lite("a b"), "a%20b");
    assert_eq!(html::urlencoding_lite("a/b"), "a%2Fb");
}

/// The redirect carries a flash message through `urlencoding_lite` and the
/// next page reads it back with `query_value`. A partial decoder showed the
/// escape sequences themselves to the operator, which is exactly the text they
/// needed to read — a blocked launch or a Windows log path.
#[test]
fn flash_messages_survive_the_encode_decode_round_trip() {
    let messages = [
        "web-suite-node was not running",
        "readiness blocked — missing binary",
        r#"web-suite-node launched with PID 4210; log D:\NeoNexus\logs\web-suite-node.log"#,
        "failed: config path, then wallet (locked)",
        "узел остановлен · 节点已停止",
    ];
    for message in messages {
        let encoded = html::flash_query(message);
        let decoded = html::query_value(encoded.strip_prefix('?'), "flash");
        assert_eq!(
            decoded.as_deref(),
            Some(message),
            "round trip lost {message:?}"
        );
    }
}

#[test]
fn query_value_picks_the_requested_key() {
    let query = "flash=node%20stopped&tab=health";
    assert_eq!(
        html::query_value(Some(query), "flash").as_deref(),
        Some("node stopped")
    );
    assert_eq!(
        html::query_value(Some(query), "tab").as_deref(),
        Some("health")
    );
    assert_eq!(html::query_value(Some(query), "missing"), None);
    assert_eq!(html::query_value(None, "flash"), None);
}

/// A truncated or non-hex escape is operator-typed noise, not a crash: the
/// decoder must hand back the bytes it could not interpret.
#[test]
fn query_value_tolerates_malformed_escapes() {
    assert_eq!(
        html::query_value(Some("flash=tail%"), "flash").as_deref(),
        Some("tail%")
    );
    assert_eq!(
        html::query_value(Some("flash=zz%2"), "flash").as_deref(),
        Some("zz%2")
    );
    assert_eq!(
        html::query_value(Some("flash=a%zzb"), "flash").as_deref(),
        Some("a%zzb")
    );
    // A lone '%' before a multi-byte character must not index mid-char.
    assert_eq!(
        html::query_value(Some("flash=%中文"), "flash").as_deref(),
        Some("%中文")
    );
}

#[test]
fn layout_has_accessible_desktop_and_native_mobile_navigation() {
    let page = html::layout("Fleet", "home", "", "<h1>Fleet</h1>");
    assert!(page.contains(r##"class="skip-link" href="#main-content""##));
    assert!(page.contains(r#"<nav class="sidebar-nav" aria-label="Primary navigation">"#));
    assert!(page.contains(r#"<header class="mobile-nav">"#));
    assert!(page.contains(r#"<details>"#));
    assert!(page.contains(r#"aria-label="Mobile navigation""#));
    assert!(page.contains(r#"<main class="content" id="main-content" tabindex="-1">"#));
    assert!(page.contains(r#"href="/metrics""#));
    assert_eq!(page.matches(r#"action="/logout""#).count(), 2);
}

#[test]
fn tables_have_header_groups_scoped_columns_and_a_keyboard_scroll_region() {
    let markup = html::table(
        &["Name", "Status"],
        &[html::row(&[html::cell("seed-1"), html::cell("Running")])],
    );
    assert!(markup.contains(r#"role="region""#));
    assert!(markup.contains(r#"tabindex="0""#));
    assert!(markup.contains("<thead><tr>"));
    assert!(markup.contains(r#"<th scope="col">Name</th>"#));
    assert!(markup.contains("</thead><tbody><tr>"));
}

#[test]
fn fields_accept_unique_ids_and_expose_validation_state() {
    let first = html::TextField {
        id: Some("first-label"),
        label: "Label",
        name: "label",
        value: "",
        error: Some("required"),
        ..Default::default()
    }
    .render();
    let second = html::TextField {
        id: Some("second-label"),
        label: "Label",
        name: "label",
        value: "",
        ..Default::default()
    }
    .render();
    assert!(first.contains(r#"id="first-label""#));
    assert!(first.contains(r#"aria-invalid="true""#));
    assert!(first.contains(r#"aria-describedby="first-label-error""#));
    assert!(second.contains(r#"id="second-label""#));
}

#[test]
fn typed_filters_render_human_labels_and_native_controls() {
    let markup = html::typed_filter_form(
        "/health",
        &[],
        &[
            html::FilterControl::Search {
                label: "Search",
                name: "q",
                value: "seed",
                placeholder: "Node or process",
            },
            html::FilterControl::Checkbox {
                label: "High CPU",
                name: "high_cpu",
                checked: true,
            },
        ],
    );
    assert!(markup.contains(r#"type="search" name="q""#));
    assert!(markup.contains(">Search</span>"));
    assert!(markup.contains(r#"type="checkbox" name="high_cpu" value="1" checked"#));
    assert!(!markup.contains(">q</span>"));
}

#[test]
fn progressive_script_submits_the_named_intent_without_reloading_jobs() {
    let script = crate::web::assets::SCRIPT;
    assert!(script.contains("form.requestSubmit(submitter)"));
    assert!(script.contains("flag.name = intent"));
    assert!(!script.contains("window.location.reload"));
}
