(function() {var implementors = {};
implementors["webapp"] = [{"text":"impl Send for Config","synthetic":true,"types":[]},{"text":"impl Send for ServerConfig","synthetic":true,"types":[]},{"text":"impl Send for LogConfig","synthetic":true,"types":[]},{"text":"impl Send for PostgresConfig","synthetic":true,"types":[]},{"text":"impl Send for Session","synthetic":true,"types":[]},{"text":"impl Send for LoginCredentials","synthetic":true,"types":[]},{"text":"impl Send for LoginSession","synthetic":true,"types":[]},{"text":"impl Send for Logout","synthetic":true,"types":[]},{"text":"impl Send for Login","synthetic":true,"types":[]},{"text":"impl Send for Logout","synthetic":true,"types":[]},{"text":"impl Send for table","synthetic":true,"types":[]},{"text":"impl Send for star","synthetic":true,"types":[]},{"text":"impl Send for token","synthetic":true,"types":[]}];
implementors["webapp_backend"] = [{"text":"impl !Send for Server","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()