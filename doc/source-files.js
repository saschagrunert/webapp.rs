var N = null;var sourcesIndex = {};
sourcesIndex["app"] = {"name":"","dirs":[],"files":["main.rs"]};
sourcesIndex["backend"] = {"name":"","dirs":[],"files":["main.rs"]};
sourcesIndex["webapp"] = {"name":"","dirs":[{"name":"protocol","dirs":[],"files":["mod.rs","model.rs","request.rs","response.rs"]}],"files":["config.rs","lib.rs","schema.rs"]};
sourcesIndex["webapp_backend"] = {"name":"","dirs":[{"name":"http","dirs":[],"files":["login_credentials.rs","login_session.rs","logout.rs","mod.rs"]},{"name":"server","dirs":[],"files":["mod.rs"]},{"name":"token","dirs":[],"files":["mod.rs"]}],"files":["cbor.rs","database.rs","lib.rs"]};
sourcesIndex["webapp_frontend"] = {"name":"","dirs":[{"name":"component","dirs":[],"files":["content.rs","login.rs","mod.rs","root.rs"]},{"name":"service","dirs":[],"files":["cookie.rs","log.rs","mod.rs","session_timer.rs","uikit.rs"]}],"files":["api.rs","lib.rs","route.rs","string.rs"]};
createSourceSidebar();
