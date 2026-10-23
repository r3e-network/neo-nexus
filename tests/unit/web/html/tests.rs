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
    assert_eq!(page.matches(r#"action="/logout""#).count(), 3);
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

/// A failure and a success used to render through the same banner with the
/// same colour and the same `role="status"`, so the only way to tell them
/// apart was to read the sentence. The severity now travels with the message.
#[test]
fn a_failed_control_is_not_rendered_like_a_successful_one() {
    let failure = html::flash_banner(&html::Flash::danger("failed: port 26032 is in use"));
    let success = html::flash_banner(&html::Flash::ok("node launched with PID 4210"));

    assert!(failure.contains(r#"class="flash danger""#), "{failure}");
    assert!(success.contains(r#"class="flash""#), "{success}");
    assert!(!success.contains("danger"), "{success}");

    // A failure interrupts; a success queues politely behind whatever the
    // screen reader is still reading.
    assert!(failure.contains(r#"role="alert""#), "{failure}");
    assert!(failure.contains(r#"aria-live="assertive""#), "{failure}");
    assert!(success.contains(r#"role="status""#), "{success}");
    assert!(success.contains(r#"aria-live="polite""#), "{success}");
}

/// The severity has to survive the redirect, not just the render: the page that
/// reads the query back is a different request from the one that wrote it.
#[test]
fn flash_severity_survives_the_redirect_round_trip() {
    let danger = html::Flash::danger("failed: config path is not writable");
    let target = danger.redirect("/nodes");
    assert!(target.starts_with("/nodes?flash="), "{target}");
    assert!(target.contains("flash_kind=danger"), "{target}");

    let query = target
        .split_once('?')
        .expect("the redirect carries a query")
        .1;
    let recovered = html::flash(Some(query));
    assert_eq!(recovered.message, "failed: config path is not writable");
    assert_eq!(recovered.kind, html::FlashKind::Danger);

    // A success must not pick up the failure marker by accident.
    let ok = html::Flash::ok("node launched with PID 4210");
    let target = ok.redirect("/nodes");
    assert!(!target.contains("flash_kind"), "{target}");
    let query = target
        .split_once('?')
        .expect("the redirect carries a query")
        .1;
    assert_eq!(html::flash(Some(query)).kind, html::FlashKind::Ok);
}

/// A path that already carries a query keeps it; the banner joins with `&`.
#[test]
fn flash_redirect_keeps_the_query_a_path_already_carries() {
    let flash = html::Flash::ok("done");
    assert_eq!(
        flash.redirect("/plugins?node=seed-1"),
        "/plugins?node=seed-1&flash=done"
    );
    assert_eq!(flash.redirect("/plugins"), "/plugins?flash=done");
}

/// The severity has to be a decision the *caller* makes, not one the renderer
/// guesses from the wording of the message.
#[test]
fn a_plain_string_is_a_successful_outcome() {
    let flash: html::Flash = "node launched".into();
    assert_eq!(flash.kind, html::FlashKind::Ok);
    assert_eq!(flash.message, "node launched");
    assert!(!flash.is_empty());
    assert!(html::Flash::default().is_empty());
}

/// The content security policy pins `script-src` to a hash of the bundled
/// script and allows no `'unsafe-inline'`, so an inline `onclick` is never
/// executed by the browser.
///
/// The rule that keeps the console clear of them lives in the CI contract
/// suite, where it reads the syntax tree. This test only pins that the
/// delegated listeners the console relies on are present in the script — a
/// page that marks an element with `data-*` and a script that never listens
/// for it is the same dead control by another route.
#[test]
fn the_delegated_listeners_the_pages_mark_elements_with_are_present() {
    let script = crate::web::assets::SCRIPT;
    assert!(
        script.contains("[data-rpc-toggle]"),
        "the RPC service toggle is marked with data-rpc-toggle but nothing listens for it"
    );
    assert!(
        script.contains("[data-role-preset]"),
        "the role preset cards are marked with data-role-preset but nothing listens for them"
    );
    assert!(
        script.contains("[data-confirm]") || script.contains("data-confirm"),
        "the confirmation prompt is missing from the bundled script"
    );
    // The delegated handlers replace the inline ones; none may come back.
    assert!(
        !script.contains("window.toggleNodeRpcService"),
        "the RPC toggle is still exposed on window for an inline handler to call"
    );
    assert!(
        !script.contains("window.selectNodeRolePreset"),
        "the role preset handler is still exposed on window for an inline handler to call"
    );
}

#[test]
fn a_text_field_carries_the_constraints_the_domain_enforces() {
    let markup = crate::web::html::TextField {
        label: "RPC port",
        name: "rpc_port",
        value: "10332",
        required: true,
        min: Some("1"),
        max: Some("65535"),
        step: Some("1"),
        input_type: crate::web::html::TextInputType::Number,
        ..Default::default()
    }
    .render();
    assert!(markup.contains(r#"type="number""#), "{markup}");
    assert!(markup.contains(r#" min="1""#), "{markup}");
    assert!(markup.contains(r#" max="65535""#), "{markup}");
    assert!(markup.contains(r#" step="1""#), "{markup}");
    assert!(markup.contains(" required"), "{markup}");
}

#[test]
fn a_text_field_without_constraints_stays_a_plain_text_input() {
    let markup = crate::web::html::TextField {
        label: "Name",
        name: "name",
        value: "seed-1",
        ..Default::default()
    }
    .render();
    assert!(markup.contains(r#"type="text""#), "{markup}");
    assert!(!markup.contains(" required"), "{markup}");
    assert!(!markup.contains(" min="), "{markup}");
    assert!(!markup.contains(" pattern="), "{markup}");
}

#[test]
fn a_text_field_pattern_is_escaped_like_every_other_attribute() {
    let markup = crate::web::html::TextField {
        label: "Expected SHA-256",
        name: "expected_sha256",
        value: "",
        pattern: Some("[0-9a-fA-F]{64}"),
        maxlength: Some("64"),
        ..Default::default()
    }
    .render();
    assert!(markup.contains(r#"pattern="[0-9a-fA-F]{64}""#), "{markup}");
    assert!(markup.contains(r#"maxlength="64""#), "{markup}");
}

#[test]
fn a_choice_field_can_be_required() {
    let options = vec!["a".to_string(), "b".to_string()];
    let markup = crate::web::html::ChoiceField {
        label: "Client",
        name: "node_type",
        options: &options,
        selected: "a",
        required: true,
        ..Default::default()
    }
    .render();
    assert!(markup.contains(" required"), "{markup}");
}

#[test]
fn an_empty_state_uses_the_heading_level_it_is_given() {
    let heading_two = crate::web::html::empty_state("Nothing here", "Not yet.", "");
    assert!(
        heading_two.contains("<h2>Nothing here</h2>"),
        "{heading_two}"
    );
    let heading_three = crate::web::html::empty_state_at(3, "Nothing here", "Not yet.", "");
    assert!(
        heading_three.contains("<h3>Nothing here</h3>"),
        "{heading_three}"
    );
    // A heading that jumps up a level is not one a screen reader can follow.
    assert!(!heading_three.contains("<h2>"), "{heading_three}");
}

#[test]
fn an_empty_state_escapes_the_title_and_body_but_not_the_actions() {
    let markup = crate::web::html::empty_state(
        "<script>alert(1)</script>",
        "Quotes \" and & ampersands",
        r#"<a class="btn" href="/nodes">Go</a>"#,
    );
    // The title and body are operator-facing text, so they are escaped:
    // the markup never reaches the page as markup.
    assert!(!markup.contains("<script>alert(1)</script>"), "{markup}");
    assert!(
        markup.contains("&lt;script&gt;alert(1)&lt;/script&gt;"),
        "{markup}"
    );
    assert!(markup.contains("&quot; and &amp; ampersands"), "{markup}");
    // The action is markup the page built itself.
    assert!(
        markup.contains(r#"<a class="btn" href="/nodes">Go</a>"#),
        "{markup}"
    );
}

#[test]
fn the_pending_marker_is_hidden_until_the_form_is_submitted() {
    let marker = crate::web::html::pending_marker("Collecting…");
    assert!(marker.contains(r#"data-pending"#), "{marker}");
    assert!(marker.contains(r#"role="status""#), "{marker}");
    // The stylesheet keeps it out of the way until the script reveals it.
    let styles = crate::web::assets::CSS;
    assert!(
        styles.contains(".pending { display: none;"),
        "the pending marker must start hidden"
    );
    assert!(
        styles.contains(r#"form[data-pending-state="running"] .pending"#),
        "nothing reveals the pending marker"
    );
}

#[test]
fn the_bundled_script_reveals_the_pending_marker_without_blocking_the_submit() {
    let script = crate::web::assets::SCRIPT;
    assert!(
        script.contains("[data-pending]"),
        "the pending marker is rendered but nothing looks for it"
    );
    assert!(
        script.contains(r#"data-pending-state"#),
        "the script never marks the form as running"
    );
    // The listener must not call `preventDefault`: a form that takes a while
    // still has to submit when the operator clicks it.
    //
    // The script has more than one submit listener, so the pending one is
    // found by what it looks for rather than by being the first.
    let listener = script
        .split("addEventListener(\"submit\"")
        .find(|section| section.contains("[data-pending]"))
        .expect("a submit listener that looks for the pending marker");
    let handler = &listener[..listener.len().min(600)];
    assert!(
        !handler.contains("preventDefault"),
        "the pending listener blocks the submission: {handler}"
    );
}
