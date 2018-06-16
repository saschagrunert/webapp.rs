@0x998efb67a0d7453f;

struct Request {
    struct Login {
        username @0 :Text;
        password @1 :Text;
    }

    union {
        login @0 :Login;
        logout @1 :Void;
    }
}

struct Response {
    struct Login {
        success @0 :Bool;
    }

    union {
        login @0 :Login;
        logout @1 :Text;
    }
}
