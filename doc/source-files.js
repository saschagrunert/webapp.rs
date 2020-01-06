var N = null;var sourcesIndex = {};
sourcesIndex["app"] = {"name":"","files":["main.rs"]};
sourcesIndex["backend"] = {"name":"","files":["main.rs"]};
sourcesIndex["webapp"] = {"name":"","dirs":[{"name":"protocol","files":["mod.rs","model.rs","request.rs","response.rs"]}],"files":["config.rs","lib.rs","schema.rs"]};
sourcesIndex["webapp_backend"] = {"name":"","dirs":[{"name":"http","files":["login_credentials.rs","login_session.rs","logout.rs","mod.rs"]},{"name":"server","files":["mod.rs"]},{"name":"token","files":["mod.rs"]}],"files":["database.rs","lib.rs"]};
sourcesIndex["webapp_frontend"] = {"name":"","dirs":[{"name":"component","files":["content.rs","login.rs","mod.rs","root.rs"]},{"name":"service","files":["cookie.rs","log.rs","mod.rs","session_timer.rs","uikit.rs"]}],"files":["api.rs","lib.rs","route.rs","string.rs"]};
createSourceSidebar();
