use std::net;
use std::process;
use std::str::FromStr;

mod cli;


fn main() {

    let matches = cli::cli();
    let verbose = matches.get_one::<bool>("verbose").unwrap();

    let domain_name = matches.get_one::<String>("domain name").unwrap();
    let data = dnsr::compose(45, &domain_name);
    if *verbose {
        dnsr::print_header(&data);
        dnsr::print_flags(&data);
        dnsr::print(&data);
    }

    let udp_result = net::UdpSocket::bind("0.0.0.0:0");
    let udp = match udp_result {
        Ok(udp) => udp,
        Err(err) => {
            eprintln!("Cannot bind to UDP socket {}", "0.0.0.0");
            eprintln!("Error: {err}");
            process::exit(1);
        },
    };

    let addr_str = matches.get_one::<String>("server").unwrap();
    let addr = net::Ipv4Addr::from_str(addr_str).unwrap_or_else(|err| {
            eprintln!("Invalid DNS server IP address");
            eprintln!("Error: {err}");
            process::exit(1);
    });

    let port: u16 = 53;
    let socket = net::SocketAddrV4::new(addr, port);
    if let Err(err) = udp.connect(socket) {
        eprintln!("Cannot connect to UDP socket {}", socket.to_string());
        eprintln!("Error: {err}");
        process::exit(1);
    }

    let sent_size = match udp.send(&data)
    {
        Ok(size) => {
            if *verbose {
                println!("Sent {} bytes", size);
            }
            size
        },
        Err(err) => {
            eprintln!("Couldn't send data to DNS server");
            eprintln!("Error: {err}");
            process::exit(1);
        },
    };

    let mut buf = [0; 256];
    match udp.recv(&mut buf) {
        Ok(size) => {
            println!("Received {} bytes", size);
            if *verbose {
                dnsr::print_header(&buf[..size]);
                dnsr::print_flags(&buf[..size]);
                dnsr::print(&buf[..size]);
            }
        },
        Err(err) => {
            eprintln!("Couldn't send data to DNS server");
            eprintln!("Error: {err}");
            process::exit(1);
        },
    };

    let rcode = dnsr::get_rcode(&buf);
    if rcode == dnsr::ResponseCode::Ok {
        let ips = dnsr::get_ips(&buf, sent_size);
        println!("{:?}", ips);
    } else {
        eprintln!("Received error code");
        eprintln!("RCODE: {:?}", rcode);
    }

}
