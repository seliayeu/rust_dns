pub struct DNSHeader {
    qr: u8,
    opcode: u8,
    aa: u8,
    tc: u8,
    red: u8,
    ra: u8,
    z: u8,
    rCode: u8,
    qdCount: u16,
    anCount: u16,
    nsCount: u16,
    arCount: u16,
}

pub struct DNSQuestion {
    qName: String,
    qType: u16,
    qClass: u16,
}

pub struct DNSResponse {
    name: String,
    rType: u16,
    rClass: u16,
    ttl: u32,
    rdLength: u16,
    rData: String,
}

pub struct DNSPacket {
    header: DNSHeader,
    question: DNSQuestion,
    response: Option<DNSResponse>,
}

pub struct DNSEntry {
    name: String,
    rType: u16,
    rData: String,
}

impl DNSResponse {
    fn new(entry: &DNSEntry) -> DNSResponse {
        DNSResponse {
            name: entry.name.clone(),
            rType: entry.rType,
            rClass: 1,
            ttl: 60,
            rdLength: entry.rData.len() as u16,
            rData: entry.rData.clone(),
        }
    }
}

impl From<Vec<u8>> for DNSPacket {
    fn from(value: Vec<u8>) -> Self {
        todo!()
    }
}

impl From<DNSPacket> for Vec<u8> {
    fn from(value: DNSPacket) -> Self {
        todo!()
    }
}

fn main() {
    println!("Hello, world!");
}

