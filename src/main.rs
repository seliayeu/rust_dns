use tokio::{net::UdpSocket, fs};
use std::io;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;

pub struct DNSHeader {
    id: u16,
    qr: u8,
    opcode: u8,
    aa: u8,
    tc: u8,
    rd: u8,
    ra: u8,
    z: u8,
    r_code: u8,
    qd_count: u16,
    an_count: u16,
    ns_count: u16,
    ar_count: u16,
}

pub struct DNSQuestion {
    q_name: String,
    q_type: u16,
    q_class: u16,
}

pub struct DNSResponse {
    name: String,
    r_type: u16,
    r_class: u16,
    ttl: u32,
    rd_length: u16,
    r_data: String,
}

pub struct DNSPacket {
    header: DNSHeader,
    question: DNSQuestion,
    response: Option<DNSResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DNSEntry {
    name: String,
    #[serde(rename = "type")]
    r_type: String,
    #[serde(rename = "data")]
    r_data: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DNSEntryList {
    records: Vec<DNSEntry>,
}



impl DNSResponse {
    fn new(entry: &DNSEntry) -> DNSResponse {
        DNSResponse {
            name: entry.name.clone(),
            r_type: get_r_type_id(entry.r_type.clone().as_str()),
            r_class: 1,
            ttl: 60,
            rd_length: entry.r_data.len() as u16,
            r_data: entry.r_data.clone(),
        }
    }
}

impl DNSPacket {
    fn to_packet(self, req_buffer: [u8; 64], len: usize ) -> [u8; 64] {
        let mut req_buffer = req_buffer;
        let response = self.response.unwrap();
        req_buffer[len] = (response.r_type >> 8) as u8;
        req_buffer[len + 1] = response.r_type as u8;
        req_buffer[len + 2] = (response.r_class >> 8) as u8;
        req_buffer[len + 3] = response.r_class as u8;
        req_buffer[len + 4] = (response.ttl >> 24) as u8;
        req_buffer[len + 5] = (response.ttl >> 16) as u8;
        req_buffer[len + 6] = (response.ttl >> 8) as u8;
        req_buffer[len + 7] = response.ttl as u8;
        req_buffer[len + 8] = (response.rd_length >> 8) as u8;
        req_buffer[len + 9] = response.rd_length as u8;

        req_buffer
    }
}

pub fn get_r_type_id(r_type: &str) -> u16 {
    match r_type {
        "A" => 1,
        "AAAA" => 28,
        "TXT" => 16,
        "RP" => 17,
        _ => 0,
    }
}

pub fn parse_q_name(buffer: &[u8; 52]) -> String {
    let mut buffer_ind = 0;
    let mut section_size = 0;
    let mut vec = Vec::new();

    while buffer[buffer_ind] != 0 {
        section_size = buffer[buffer_ind] as usize;
        buffer_ind += 1;

        while buffer_ind < buffer_ind + section_size {
            vec.push(buffer[buffer_ind] as char);
            buffer_ind += 1;
        }
        vec.push('.');
    }

    vec.pop();

    String::from_iter(vec)
}

impl From<[u8; 64]> for DNSPacket {
    fn from(value: [u8; 64]) -> Self {
        let header = DNSHeader {
            id: (value[0] as u16) << 8 | value[1] as u16,
            qr: value[2] & 0b1000_0000,
            opcode: value[2] & 0b0111_1000,
            aa: value[2] & 0b0000_0100,
            tc: value[2] & 0b0000_0010,
            rd: value[2] & 0b0000_0001,
            ra: value[3] & 0b1000_0000,
            z: value[3] & 0b0111_1000,
            r_code: value[3] & 0b0000_1111,
            qd_count: (value[4] as u16) << 8 | value[5] as u16,
            an_count: (value[6] as u16) << 8 | value[7] as u16,
            ns_count: (value[8] as u16) << 8 | value[9] as u16,
            ar_count: (value[10] as u16) << 8 | value[11] as u16, 
        };

        let q_name = parse_q_name(&value[12..].try_into().expect("invalid"));

        let question = DNSQuestion {
            q_name: q_name.clone(),
            q_type: (value[12 + q_name.len() + 2] as u16) << 8 | value[12 + q_name.len() + 3] as u16,
            q_class: (value[12 + q_name.len() + 4] as u16) << 8 | value[12 + q_name.len() + 5] as u16,
        };

        DNSPacket { header: header, question: question, response: None }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let records_file = &args[3];
    println!("{:?}", records_file);
    let socket = UdpSocket::bind("127.0.0.1:53").await?;
    let mut buf = [0; 64];
    
    let entry_list: DNSEntryList = serde_json::from_str(&fs::read_to_string(records_file).await?)?;

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        println!("{:?} bytes received from {:?}", len, addr);

        if buf[2] & 0b1111_0000 == 0b0000_0000 {
            let mut packet = DNSPacket::from(buf.clone());
            let entry = entry_list
                .records
                .iter()
                .find(|&x| x.name == packet.question.q_name)
                .unwrap();
            let response = DNSResponse::new(&entry);
            packet.response = Some(response);
            let response_buf: [u8; 64] = packet.to_packet(buf, len);
            socket.send_to(&response_buf, addr).await?;
        }
    }

    Ok(())
}

