@0x998efb67a0d7453f;

struct Request {
    union {
        login  @0 :Login;   # The login credentials or a token
        logout @1 :Text;    # The session token
    }

    struct Login {
        union {
            credentials @0 :Credentials;
            token       @1 :Text;
        }

        struct Credentials {
            username @0 :Text;
            password @1 :Text;
        }
    }
}

struct Response {
    union {
        login  @0 :Login;
        logout @1 :Logout;
    }

    struct Login {
        union {
            token @0 :Text;
            error @1 :Text;
        }
    }

    struct Logout {
        union {
            success @0 :Void;
            error   @1 :Text;
        }
    }
}
