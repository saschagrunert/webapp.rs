extern crate webapp;
use webapp::Server;

fn main() {
    if let Err(e) = Server::run() {
        println!("Unable to start server: {}", e);
    }
}
