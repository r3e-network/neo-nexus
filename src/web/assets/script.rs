//! Progressive enhancement for the server-rendered workbench.

pub const SCRIPT: &str = r#"
(function () {
  document.addEventListener("change", function (event) {
    var el = event.target;
    if (el.matches && el.matches("select[data-autosubmit]")) {
      var form = el.form;
      var intent = el.getAttribute("data-autosubmit");
      var submitter = form && form.elements.namedItem(intent);
      if (!form || !submitter || submitter.type !== "submit") return;
      if (form.requestSubmit) {
        form.requestSubmit(submitter);
      } else {
        // `form.submit()` does not include a submit button's name/value. Carry
        // the intent explicitly on older engines before using that fallback.
        var flag = document.createElement("input");
        flag.type = "hidden";
        flag.name = intent;
        flag.value = submitter.value || "1";
        form.appendChild(flag);
        form.submit();
      }
    }
  });

  function refreshFleet() {
    fetch("/api/fleet").then(function (response) {
      if (response.status === 401) {
        window.location.href = "/login";
        return null;
      }
      return response.ok ? response.json() : null;
    }).then(function (data) {
      if (!data) return;
      data.nodes.forEach(function (node) {
        var selector = '[data-node-id="' + node.id + '"]';
        document.querySelectorAll(selector + " [data-node-status]")
          .forEach(function (element) {
            element.textContent = node.status;
            element.className = "badge " + node.status.toLowerCase();
          });
        document.querySelectorAll(selector + " [data-node-rpc]")
          .forEach(function (element) {
            element.textContent = node.rpc_health;
          });
      });
    }).catch(function () {
      // The next interval is the retry. Keep the last trustworthy SSR value.
    });
  }

  if (document.querySelector("[data-node-id]")) {
    window.setInterval(refreshFleet, 5000);
  }

})();
"#;
