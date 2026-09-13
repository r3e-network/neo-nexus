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
            var isDot = element.classList.contains("status-dot");
            var statusClass = node.status.toLowerCase();
            if (isDot) {
              element.className = "status-dot " + statusClass;
            } else {
              element.textContent = node.status;
              element.className = "badge " + statusClass;
            }
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

  // Node binary inference and installed runtime quick-picker
  function inferClientFromPath(path) {
    if (!path) return null;
    var trimmed = path.trim();
    if (!trimmed) return null;
    var normalized = trimmed.replace(/\\/g, "/");
    var filename = normalized.split("/").pop().toLowerCase();
    var stem = filename.replace(/\.(exe|dll)$/, "");
    if (stem.indexOf("neox-geth") !== -1 || (stem.indexOf("geth") !== -1 && stem.indexOf("reth") === -1)) {
      return { type: "neox-geth", label: "Neo X (neox-geth)" };
    }
    if (stem.indexOf("neox-rs") !== -1 || stem.indexOf("reth") !== -1) {
      return { type: "neox-rs", label: "Neo X (neox-rs)" };
    }
    if (stem.indexOf("neo-node") !== -1 || stem === "neo-rs") {
      return { type: "neo-rs", label: "Neo N3 (neo-rs)" };
    }
    if (stem.indexOf("neo-go") !== -1) {
      return { type: "neo-go", label: "Neo N3 (neo-go)" };
    }
    if (stem.indexOf("neo-cli") !== -1) {
      return { type: "neo-cli", label: "Neo N3 (neo-cli)" };
    }
    return null;
  }

  var binaryInput = document.getElementById("node_binary_path");
  var clientSelect = document.querySelector('select[name="node_type"]');
  var badge = document.getElementById("binary-inference-badge");

  if (binaryInput && clientSelect) {
    binaryInput.addEventListener("input", function () {
      var detected = inferClientFromPath(binaryInput.value);
      if (detected) {
        if (clientSelect.value !== detected.type) {
          clientSelect.value = detected.type;
        }
        if (badge) {
          badge.textContent = "✨ Auto-detected client: " + detected.label;
          badge.style.display = "inline-flex";
        }
      } else {
        if (badge) badge.style.display = "none";
      }
    });

    document.addEventListener("click", function (event) {
      var chip = event.target.closest(".runtime-chip");
      if (!chip) return;
      event.preventDefault();
      var binary = chip.getAttribute("data-binary");
      var version = chip.getAttribute("data-version");
      var client = chip.getAttribute("data-client");
      if (binary) binaryInput.value = binary;
      var versionInput = document.querySelector('input[name="runtime_version"]');
      if (versionInput && version) versionInput.value = version;
      if (clientSelect && client && clientSelect.value !== client) {
        clientSelect.value = client;
      }
      document.querySelectorAll(".runtime-chip").forEach(function (c) {
        c.classList.remove("runtime-chip-active");
      });
      chip.classList.add("runtime-chip-active");
      if (badge) {
        badge.textContent = "✅ Selected installed runtime: " + (client || "node") + (version ? " v" + version : "");
        badge.style.display = "inline-flex";
      }
    });
  }

  // Role presets and conditional RPC service controls
  window.toggleNodeRpcService = function (enabled) {
    var configPanel = document.getElementById("rpc-config-panel");
    var disabledNotice = document.getElementById("rpc-disabled-notice");
    var statusTag = document.getElementById("rpc-status-tag");

    if (configPanel) configPanel.style.display = enabled ? "block" : "none";
    if (disabledNotice) disabledNotice.style.display = enabled ? "none" : "flex";

    if (statusTag) {
      statusTag.innerHTML = enabled
        ? '<span class="badge running">RPC Enabled</span>'
        : '<span class="badge stopped">RPC Disabled (P2P Only)</span>';
    }

    // Toggle RpcServer plugin checkbox if present
    var rpcServerCheckbox = document.querySelector('.plugin-checkbox[data-plugin="RpcServer"]');
    if (rpcServerCheckbox) {
      if (!enabled) {
        rpcServerCheckbox.checked = false;
        rpcServerCheckbox.disabled = true;
        rpcServerCheckbox.title = "RPC Server plugin requires JSON-RPC service to be enabled.";
      } else {
        rpcServerCheckbox.disabled = false;
        rpcServerCheckbox.checked = true;
        rpcServerCheckbox.title = "";
      }
    }
  };

  window.selectNodeRolePreset = function (button) {
    if (!button) return;
    var role = button.getAttribute("data-role");
    var rpcFlag = button.getAttribute("data-rpc") === "1";
    var pluginsStr = button.getAttribute("data-plugins") || "";
    var storage = button.getAttribute("data-storage");

    // Update active highlight
    document.querySelectorAll(".role-preset-card").forEach(function (btn) {
      btn.classList.remove("active");
    });
    button.classList.add("active");

    // Update hidden role input
    var roleInput = document.getElementById("f-role");
    if (roleInput) roleInput.value = role;

    // Update RPC checkbox and panel
    var rpcCheckbox = document.getElementById("f-enable_rpc");
    if (rpcCheckbox) {
      rpcCheckbox.checked = rpcFlag;
      window.toggleNodeRpcService(rpcFlag);
    }

    // Update recommended storage engine if selectable
    var storageSelect = document.querySelector('select[name="storage_engine"]');
    if (storageSelect && storage) {
      for (var i = 0; i < storageSelect.options.length; i++) {
        if (storageSelect.options[i].value.toLowerCase() === storage.toLowerCase()) {
          storageSelect.selectedIndex = i;
          break;
        }
      }
    }

    // Update plugin checkboxes
    var targetPlugins = pluginsStr ? pluginsStr.split(",") : [];
    document.querySelectorAll(".plugin-checkbox").forEach(function (cb) {
      var pluginId = cb.getAttribute("data-plugin");
      cb.checked = targetPlugins.indexOf(pluginId) !== -1;
    });
  };

  window.toggleSelectAllNodes = function (master) {
    document.querySelectorAll('input[name="node_ids"]').forEach(function (cb) {
      cb.checked = master.checked;
    });
  };

  // Progressive AWS tabs switching
  document.addEventListener("click", function (event) {
    var tabBtn = event.target && event.target.closest ? event.target.closest("[data-tab-target]") : null;
    if (!tabBtn) return;
    var targetId = tabBtn.getAttribute("data-tab-target");
    var container = tabBtn.closest(".aws-tab-container");
    if (!container) return;
    container.querySelectorAll("[data-tab-target]").forEach(function (btn) {
      var isMatch = btn === tabBtn;
      btn.classList.toggle("active", isMatch);
      btn.setAttribute("aria-selected", isMatch ? "true" : "false");
    });
    container.querySelectorAll("[data-tab-panel]").forEach(function (panel) {
      var isMatch = panel.getAttribute("data-tab-panel") === targetId;
      panel.classList.toggle("active", isMatch);
      panel.style.display = isMatch ? "block" : "none";
    });
  });

  // CSP-safe bulk toggle all nodes
  document.addEventListener("click", function (event) {
    var btn = event.target && event.target.closest ? event.target.closest('[data-action="toggle-all-nodes"]') : null;
    if (!btn) return;
    var cbs = document.querySelectorAll('input[name="node_ids"]');
    var allChecked = Array.from(cbs).every(function (c) { return c.checked; });
    cbs.forEach(function (c) { c.checked = !allChecked; });
  });

  // Global search shortcut (press / when not inside an input)
  document.addEventListener("keydown", function (event) {
    if (event.key === "/" && document.activeElement && !document.activeElement.matches("input, textarea, select")) {
      var searchInput = document.getElementById("global-resource-search") || document.querySelector('input[name="q"]');
      if (searchInput) {
        event.preventDefault();
        searchInput.focus();
        searchInput.select();
      }
    }
  });

  // Progressive clipboard copy for elements with data-copy-target attribute
  document.addEventListener("click", function (event) {
    var btn = event.target && event.target.closest ? event.target.closest("[data-copy-target]") : null;
    if (!btn) return;
    var targetId = btn.getAttribute("data-copy-target");
    var target = document.getElementById(targetId);
    if (!target) return;
    var text = target.value || target.textContent || "";
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(function () {
        var original = btn.textContent;
        btn.textContent = "✓ Copied!";
        setTimeout(function () {
          btn.textContent = original;
        }, 1500);
      });
    }
  });

  // Progressive confirmation prompt for forms with data-confirm attribute (CSP compliant)
  document.addEventListener("submit", function (event) {
    var form = event.target;
    var msg = form && form.getAttribute ? form.getAttribute("data-confirm") : null;
    if (msg && !window.confirm(msg)) {
      event.preventDefault();
    }
  });

  // Progressive AWS EC2 Master-Detail Row Selection
  document.addEventListener("click", function (event) {
    var row = event.target && event.target.closest ? event.target.closest("tbody tr[data-node-id]") : null;
    if (!row) return;
    if (event.target.closest("input, button, a, form")) return;
    document.querySelectorAll("tbody tr[data-node-id]").forEach(function (r) {
      r.classList.remove("aws-row-selected");
    });
    row.classList.add("aws-row-selected");
    var drawer = document.getElementById("ec2-instance-drawer");
    if (!drawer) return;
    var name = row.getAttribute("data-node-name");
    var id = row.getAttribute("data-node-id");
    var type = row.getAttribute("data-node-type");
    var net = row.getAttribute("data-node-net");
    var ver = row.getAttribute("data-node-ver");
    var rpc = row.getAttribute("data-node-rpc-port");
    var p2p = row.getAttribute("data-node-p2p-port");
    var role = row.getAttribute("data-node-role");
    var signer = row.getAttribute("data-node-signer");
    var status = row.getAttribute("data-node-status");
    if (name) {
      var nameEl = drawer.querySelector("[data-drawer-name]");
      if (nameEl) nameEl.textContent = name;
    }
    if (id) {
      var idEl = drawer.querySelector("[data-drawer-id]");
      if (idEl) idEl.textContent = "(i-" + id + ")";
      var studioLink = drawer.querySelector("[data-drawer-studio-link]");
      if (studioLink) studioLink.href = "/nodes/" + encodeURIComponent(id);
      var logLink = drawer.querySelector("[data-drawer-log-link]");
      if (logLink) logLink.href = "/logs?node=" + encodeURIComponent(id);
    }
    if (type) {
      var typeEl = drawer.querySelector("[data-drawer-type]");
      if (typeEl) typeEl.textContent = "t3." + type;
    }
    if (ver) {
      var verEl = drawer.querySelector("[data-drawer-ami]");
      if (verEl) verEl.textContent = ver;
    }
    if (net) {
      var netEl = drawer.querySelector("[data-drawer-net]");
      if (netEl) netEl.textContent = net;
    }
    if (rpc) {
      var rpcEl = drawer.querySelector("[data-drawer-rpc]");
      if (rpcEl) rpcEl.textContent = ":" + rpc;
    }
    if (p2p) {
      var p2pEl = drawer.querySelector("[data-drawer-p2p]");
      if (p2pEl) p2pEl.textContent = ":" + p2p;
    }
    if (signer) {
      var signerEl = drawer.querySelector("[data-drawer-signer]");
      if (signerEl) signerEl.textContent = signer;
    }
    if (role) {
      var roleEl = drawer.querySelector("[data-drawer-role]");
      if (roleEl) roleEl.textContent = role;
    }
    if (status) {
      var statusEl = drawer.querySelector("[data-drawer-status]");
      if (statusEl) {
        var cls = status.toLowerCase() === "running" ? "running" : (status.toLowerCase() === "stopped" ? "stopped" : "error");
        statusEl.className = "badge " + cls;
        statusEl.textContent = status;
      }
    }
  });

})();
"#;
