@0x998efb67a0d7453f;

struct Request {
    union {
        login       @0 :Login;
        placeholder @1 :Void;
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
        error @0 :Error;
        login @1 :Login;
    }

    struct Error {
        description @0 :Text;
    }

    struct Login {
        token @0 :Text;
    }
}
