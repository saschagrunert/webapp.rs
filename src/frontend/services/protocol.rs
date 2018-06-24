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
    #[fail(display = "got error response: {}", description)]
    Response { description: String },
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
        // Clear the data before serialization
        self.data.clear();

        // Serialize and return
        write_message(&mut self.data, &self.builder)?;
        Ok(&self.data)
    }

    /// Read bytes into a reader with owned segments
    fn read(&mut self, data: &mut [u8]) -> Result<Reader<OwnedSegments>, Error> {
        Ok(read_message(&mut data.as_ref(), ReaderOptions::new())?)
    }

    /// Create a new login request from a given username and password
    pub fn write_login_credential_request(&mut self, username: &str, password: &str) -> Result<&[u8], Error> {
        {
            // Set the request parameters
            let mut creds = self
                .builder
                .init_root::<request::Builder>()
                .init_login()
                .init_credentials();
            creds.set_username(username);
            creds.set_password(password);
        }

        self.write()
    }

    /// Create a new login request from a given token
    pub fn write_login_token_request(&mut self, token: &str) -> Result<&[u8], Error> {
        // Set the request parameters
        self.builder
            .init_root::<request::Builder>()
            .init_login()
            .set_token(token);

        self.write()
    }

    /// Create a logout request from a fiven token
    pub fn write_logout_request(&mut self, token: &str) -> Result<&[u8], Error> {
        // Set the request parameters
        self.builder.init_root::<request::Builder>().set_logout(token);

        self.write()
    }

    // Get a login response for given bytes
    pub fn read_login_response(&mut self, data: &mut [u8]) -> Result<String, Error> {
        let reader = self.read(data)?;
        let response = reader.get_root::<response::Reader>()?;
        match response.which()? {
            response::Login(data) => Ok(data?.get_token()?.to_owned()),
            response::Error(data) => Err(ProtocolError::Response {
                description: data?.get_description()?.to_owned(),
            }.into()),
            _ => Err(ProtocolError::Response {
                description: "Retrieved wrong response".to_owned(),
            }.into()),
        }
    }

    // Get a logout response for given bytes
    pub fn read_logout_response(&mut self, data: &mut [u8]) -> Result<(), Error> {
        let reader = self.read(data)?;
        let response = reader.get_root::<response::Reader>()?;
        match response.which()? {
            response::Logout(_) => Ok(()),
            response::Error(data) => Err(ProtocolError::Response {
                description: data?.get_description()?.to_owned(),
            }.into()),
            _ => Err(ProtocolError::Response {
                description: "Retrieved wrong response".to_owned(),
            }.into()),
        }
    }
}
