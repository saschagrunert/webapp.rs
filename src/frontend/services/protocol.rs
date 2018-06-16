//! A protocol handling service to make (de)serialization comfortable

use capnp::{
    message::{Builder, HeapAllocator, Reader, ReaderOptions},
    serialize::OwnedSegments,
    serialize_packed::{read_message, write_message},
};
use failure::Error;
use protocol_capnp::{request, response};

pub struct ProtocolService {
    builder: Builder<HeapAllocator>,
    data: Vec<u8>,
}

#[derive(Debug, Fail)]
pub enum ProtocolError {
    #[fail(display = "invalid message type, need: {}", needed_type)]
    InvalidMessageType { needed_type: String },
}

impl ProtocolService {
    /// Create a new protocol service instance
    pub fn new() -> Self {
        Self {
            builder: Builder::new_default(),
            data: Vec::new(),
        }
    }

    /// Write the data and return to the caller
    fn write(&mut self) -> Result<&[u8], Error> {
        // Serialize and return
        write_message(&mut self.data, &self.builder)?;
        Ok(&self.data)
    }

    /// Read bytes into a reader with owned segments
    fn read(&mut self, data: &mut [u8]) -> Result<Reader<OwnedSegments>, Error> {
        Ok(read_message(&mut data.as_ref(), ReaderOptions::new())?)
    }

    /// Create a new login request from a given username and password
    pub fn write_login_request(&mut self, username: &str, password: &str) -> Result<&[u8], Error> {
        {
            // Set the request parameters
            let request = self.builder.init_root::<request::Builder>();
            let mut login = request.init_login();
            login.set_username(username);;
            login.set_password(password);;
        }

        self.write()
    }

    // Get a login response for given bytes
    pub fn read_login_response(&mut self, data: &mut [u8]) -> Result<bool, Error> {
        let reader = self.read(data)?;
        let response = reader.get_root::<response::Reader>()?;
        match response.which()? {
            response::Login(data) => Ok(data?.get_success()),
            _ => Err(Error::from(ProtocolError::InvalidMessageType {
                needed_type: "login reseponse".to_owned(),
            })),
        }
    }
}
