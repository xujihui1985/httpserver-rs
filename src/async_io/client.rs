use std::{net::TcpStream, sync::{Arc, RwLock, Mutex}, os::fd::AsRawFd, fs::File, io::Read};

use polling::Event;

use crate::{response::Response, router::Router};

use super::{reactor::Reactor, event_handler::EventHandler};

pub struct ClientRequest {
    pub client: Option<TcpStream>,
    pub router: Arc<Router>,
    pub fd: usize,
    pub reactor: Arc<RwLock<Reactor>>,
    pub state: Arc<Mutex<Option<ClientState>>>,
    pub wait_handle: Option<std::thread::JoinHandle<()>>,
}

pub enum ClientState {
    Waiting,
    ReadRequest,
    WaitingReadFile,
    ReadResponseFile(i32, String),
    WriteResponse(i32, Vec<u8>, String),
    WritingResponse,
    Close(TcpStream),
    Closed
}

impl ClientRequest {
    pub fn new(client: TcpStream, router:Arc<Router>, reactor: Arc<RwLock<Reactor>>) -> Self {
        let fd = client.as_raw_fd() as usize;
        Self {
            client: Some(client),
            fd,
            reactor,
            router,
            wait_handle: None,
            state: Arc::new(Mutex::new(None))
        }
    }

    fn set_state(&mut self, st: ClientState) {
        let mut state = self.state.lock().unwrap();
        state.replace(st);
    }

    fn take_state(&mut self) -> Option<ClientState> {
        let mut state = self.state.lock().unwrap();
        state.take()
    }

    fn initialize(&mut self) {
        if let Some(client) = self.client.as_ref() {
            self.reactor.write().unwrap().add(client, Event::readable(self.fd)).unwrap();
            self.set_state(ClientState::Waiting);
        } else {
            self.set_state(ClientState::Closed);
        }
    }

    fn read_request(&mut self) {
        if let Some(client) = self.client.as_ref() {
            let (code, path) = self.router.route_client(client).unwrap();
            self.set_state(ClientState::ReadResponseFile(code, path.to_string()));
            let mut reactor = self.reactor.write().unwrap();
            reactor.schedule(self.fd);
        } else {
            self.set_state(ClientState::Closed);
        }
    }

    fn read_response_file(&mut self, code: i32, path: String){
        let reactor = self.reactor.clone();
        let state = self.state.clone();
        let fd = self.fd;
        let handle = std::thread::spawn(move || {
            let mut file = File::open(&path).unwrap();
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();

            let s = ClientState::WriteResponse(code, buf, path);
            state.lock().unwrap().replace(s);

            reactor.read().unwrap().notify();
            reactor.write().unwrap().schedule(fd);
        });
        self.wait_handle = Some(handle);
        self.set_state(ClientState::WaitingReadFile)
    }

    fn write_response(&mut self, code: i32, _body: Vec<u8>, path: String) {
        if let Some(handle) = self.wait_handle.take() {
            handle.join().unwrap();
        }
        if let Some(client) = self.client.take() {
            self.set_state(ClientState::WritingResponse);
            let state = self.state.clone();
            let reactor = self.reactor.clone();
            let fd = self.fd;
            let handler = std::thread::spawn(move || {
                let mut response = Response::new(client);
                response.sendfile(code, &path).unwrap();

                let client = response.into_inner().unwrap();
                let s = ClientState::Close(client);
                state.lock().unwrap().replace(s);
                reactor.read().unwrap().notify();
                reactor.write().unwrap().schedule(fd);
            });
            self.wait_handle = Some(handler);
        } else {
            self.set_state(ClientState::Closed);
        }
    }

    fn close(&mut self, client: TcpStream) {
        if let Some(handle) = self.wait_handle.take() {
            handle.join().unwrap();
        }
        self.set_state(ClientState::Closed);
        self.reactor.write().unwrap().remove(self.fd,&client).unwrap();
    }
    
}

impl EventHandler for ClientRequest {
    fn id(&self) -> usize {
        self.fd
    }

    fn name(&self) -> String {
        format!("ClientRequest {}", self.fd)
    }

    fn event(&mut self, _event: polling::Event) {
        match self.take_state() {
            Some(ClientState::Waiting) => {
                self.set_state(ClientState::ReadRequest);
                let mut reactor = self.reactor.write().unwrap();
                reactor.schedule(self.fd);
            },
            Some(s) => {
                self.set_state(s);
            }
            None => {}
        }
    }

    fn poll(&mut self) {
        match self.take_state() {
            None => self.initialize(),
            Some(ClientState::ReadRequest) => self.read_request(),
            Some(ClientState::ReadResponseFile(code, path)) => {
                self.read_response_file(code, path);
            }
            Some(ClientState::WriteResponse(code, contents, path)) => {
                self.write_response(code, contents, path);
            }
            Some(ClientState::Close(client)) => self.close(client),
            _ => {}
        }
    }
}