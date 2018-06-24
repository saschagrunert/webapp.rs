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
        error  @0 :Error;
        login  @1 :Login;
        logout @2 :Void;
    }

    struct Error {
        description @0 :Text;
    }

    struct Login {
        token @0 :Text;
    }
}
