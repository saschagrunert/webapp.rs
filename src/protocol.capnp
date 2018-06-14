@0x998efb67a0d7453f;

struct Login {
    struct Request {
        username @0 :Text;
        password @1 :Text;
    }

    struct Response {
        success @0 :Bool;
    }
}
