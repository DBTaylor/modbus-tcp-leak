use warp::{self, path, Filter, get2};
use tokio_modbus::prelude::*;
use tokio_core::reactor::Core;
use futures::{future, Future};
use tokio_service::Service;
use std::thread;


struct MbServer;

impl Service for MbServer {
    type Request = Request;
    type Response = Response;
    type Error = std::io::Error;
    type Future = future::FutureResult<Self::Response, Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            Request::ReadInputRegisters(_addr, cnt) => {
                let mut registers = vec![0; cnt as usize];
                registers[2] = 0x77;
                let rsp = Response::ReadInputRegisters(registers);
                future::ok(rsp)
            }
            _ => unimplemented!(),
        }
    }
}


fn main() {
    let socket_addr = "127.0.0.1:502".parse().unwrap();
    //modbus server
    thread::spawn(move || {
        tcp::Server::new(socket_addr).serve(|| Ok(MbServer));
    });

    let mut core = Core::new().unwrap();
    let remote = core.remote();

    //get requests at this endpoint leak the opened TCP connections
    let read_input_registers = get2()
        .and(path!("rir" / u16 / u16)
        .and(path::end())
        .and_then(move |start, count|{
            let (p, c) = futures::sync::oneshot::channel::<String>(); //c eventually resolves to the result from the spawned closure
            remote.spawn(move |handle|
                client::tcp::connect(handle, socket_addr)
                .and_then(move |ctx| ctx.read_input_registers(start, count)/*.and_then(move |res| ctx.disconnect().then(|_| Ok(res)))*/) //commented out disconnect call causes panic
                .then(|res|{
                    p.send(match res{
                        Ok(data) => format!("{:?}", data),
                        Err(err) =>  format!("{:?}", err)
                    }).unwrap_or(());
                    Ok(())
                })
            );
            c.map_err(|err| warp::reject::custom(err))
        }));

    core.run(
        warp::serve(read_input_registers).bind(([127,0,0,1], 3030))
    ).unwrap();
}
