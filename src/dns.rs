#[derive(Debug)]
pub struct Message {
    /// The header section of the message.
    header: MessageHeader,
    /// A flag indicating whether domain name compression is used in the message.
    compress: bool,
    /// Questions are queries the client has for the server.
    question: Vec<Question>,
    /// Answers are responses from the server to the client's questions.
    answer: Vec<ResourceRecord>,
    /// Authority records are the servers that are authoritative for the domain in the question section.
    authority: Vec<ResourceRecord>,
    /// Additional records contain extra information that may be helpful in processing the response.
    extra: Vec<ResourceRecord>,
}

#[derive(Debug)]
struct MessageHeader {
    /// Assigned by the program that generates any kind of query.
    /// This identifier is copied into the response.
    id: u16,
    /// Specifies whether this message is a query (0) or a response (1).
    qr: u8,
    /// Specifies the kind of query in this message. 0 represents a standard query (QUERY).
    opcode: u8,
    /// Authoritative Answer - set in responses to indicate that the responding server is an authority for the domain.
    aa: u8,
    /// Truncation - indicates that this message was truncated.
    tc: u8,
    /// Recursion Desired - directs the server to pursue the query recursively.
    rd: u8,
    /// Recursion Available - set or cleared in a response to indicate recursive query support.
    ra: u8,
    /// Reserved for future use. Must be zero in all queries and responses.
    z: u8,
    /// Response code - set as part of responses and indicates success or failure of the query.
    rcode: RCode,
    /// The number of entries in the question section.
    qdcount: u16,
    /// The number of resource records in the answer section.
    ancount: u16,
    /// The number of name server resource records in the authority records section.
    nscount: u16,
    /// The number of resource records in the additional records section.
    arcount: u16,
}

#[derive(Debug, Clone)]
struct Question {
    /// The domain name that is the subject of the query.
    qname: Vec<String>,
    /// Specifies the type of the query.
    qtype: u16,
    /// Specifies the class of the query.
    qclass: u16,
}

#[derive(Debug)]
pub struct ResourceRecord {
    name: Vec<String>,
    rtype: RecordType,
    rclass: u16,
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>,
}

#[derive(Debug)]
pub enum RecordType {
    A,     // = 1, RFC 1035
    AAAA,  // = 28, RFC 3596
    CNAME, // = 5, RFC 1035
    MX,    // = 15, RFC 1035
    NS,    // = 2, RFC 1035
    PTR,   // = 12, RFC 1035
    SOA,   // = 6, RFC 1035
    SRV,   // = 33, RFC 2782
    TXT,   // = 16, RFC 1035
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum RCode {
    /// DNS Query completed successfully
    NOERROR = 0,
    /// DNS Query Format Error
    FORMERR = 1,
    /// Server failed to complete the DNS request
    SERVFAIL = 2,
     /// Domain name does not exist
    NXDOMAIN = 3,
    /// Function not implemented
    NOTIMP = 4,
    /// The server refused to answer for the query
    REFUSED = 5,
    /// Name that should not exist, does exist
    YXDOMAIN = 6,
    /// RRset that should not exist, does exist
    XRRSET = 7,
    /// Server not authoritative for the zone
    NOTAUTH = 9,
    /// Name not in zone
    NOTZONE = 10,
}

impl Message {

    /// Creates a new DNS message with the header fields set based on the request.
    ///
    /// # Arguments
    /// 
    /// * `request` - The request message to base the new message on.
    /// 
    /// # Returns
    /// 
    /// A new DNS message with the header fields set based on the request.
    pub fn new(request: &Message) -> Self {
        Message {
            header: MessageHeader {
                id: request.header.id,
                qr: 1,
                opcode: request.header.opcode,
                aa: 1, // review this later
                tc: 0, // review this later
                rd: request.header.rd,
                ra: 0, // review this later
                z: 0,
                rcode: RCode::NOERROR,
                qdcount: request.header.qdcount,
                ancount: 1, // review this later
                nscount: 0, // review this later
                arcount: 0, // review this later
            },
            compress: false,
            question: request.question.clone(),
            answer: Vec::new(),
            authority: Vec::new(),
            extra: Vec::new(),
        }
    }


    /// Creates a new DNS "not found" (NXDOMAIN) response based on the request.
    ///
    /// # Arguments
    /// 
    /// * `request` - The request message to which this response corresponds.
    /// 
    /// # Returns
    /// 
    /// A new DNS message configured as a "not found" response.
    pub fn new_not_found_response(request: &Message) -> Self {
        Message {
            header: MessageHeader {
                id: request.header.id,
                qr: 1, // This is a response
                opcode: request.header.opcode,
                aa: 1, // Assuming authoritative answer; adjust as necessary
                tc: 0, // Message not truncated
                rd: request.header.rd, // Copy recursion desired flag from request
                ra: 1, // Assuming recursion is available; adjust as necessary
                z: 0, // Reserved bits must be 0
                rcode: RCode::NXDOMAIN, // NXDOMAIN
                qdcount: request.header.qdcount, // Echo back the question count
                ancount: 0, // No answers
                nscount: 0, // No authority records in this example; adjust if you include them
                arcount: 0, // No additional records
            },
            compress: false,
            question: request.question.clone(), // Echo back the question section
            answer: Vec::new(), // No answer section for NXDOMAIN
            authority: Vec::new(), // Optionally, include SOA record in authority section
            extra: Vec::new(),
        }
    }

    /// Serializes a DNS message to a byte vector.
    ///
    /// # Arguments
    ///
    /// * `self` - The DNS message to be serialized.
    ///
    /// # Returns
    ///
    /// A byte vector containing the serialized DNS message.
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.header.serialize());
        for question in &self.question {
            bytes.extend_from_slice(&question.serialize());
        }
        for answer in &self.answer {
            bytes.extend_from_slice(&answer.serialize());
        }
        for authority in &self.authority {
            bytes.extend_from_slice(&authority.serialize());
        }
        for extra in &self.extra {
            bytes.extend_from_slice(&extra.serialize());
        }
        bytes
    }

    /// Deserializes a DNS message from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the DNS message.
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized `Message` if successful, or an `std::io::Error` if deserialization fails.
    pub fn deserialize(data: &[u8]) -> Result<Self, std::io::Error> {
        let mut offset = 0;

        let header = MessageHeader::deserialize(&data[offset..])?;
        offset += 12; // Header is 12 bytes

        let mut question = Vec::new();
        for _ in 0..header.qdcount {
            let (q, new_offset) = Question::deserialize(data, offset)?;
            offset = new_offset;
            question.push(q);
        }

        Ok(Message {
            header,
            compress: false,
            question,
            answer: Vec::new(),
            authority: Vec::new(),
            extra: Vec::new(),
        })
    }

    /// Parses a QNAME from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the QNAME.
    ///
    /// * `start_offset` - The offset in the byte slice where the QNAME starts.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple with the QNAME as a vector of strings and the new offset in the byte slice if successful, or an `std::io::Error` if parsing fails.
    fn parse_qname(
        data: &[u8],
        start_offset: usize,
    ) -> Result<(Vec<String>, usize), std::io::Error> {
        let mut offset = start_offset;
        let mut labels = Vec::new();
        let mut length = data[offset] as usize;

        while length > 0 {
            offset += 1; // Move past the length byte
            let label = std::str::from_utf8(&data[offset..offset + length]).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid QNAME")
            })?;
            labels.push(label.to_string());

            offset += length; // Move past the current label
            length = data[offset] as usize; // Length of the next label
        }

        Ok((labels, offset + 1)) // Return the labels and the new offset (after the null byte)
    }

    /// Gets the number of question in the message.
    ///
    /// # Returns
    ///
    /// The number of question in the message.
    pub fn question_count(&self) -> usize {
        self.question.len()
    }

    /// Converts the qname of a specified question to a dot-separated string.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the question for which the domain name is to be returned.
    ///
    /// # Returns
    ///
    /// An `Option<String>` containing the dot-separated domain name if the index is valid, or `None` if the index is out of bounds.
    pub fn qname_to_string(&self) -> String {
        self.question
            .get(0)
            .map(|q| q.qname.join("."))
            .unwrap_or_else(|| "default_value".to_string())
    }
}

impl MessageHeader {
    /// Serializes a message header to a byte vector.
    ///
    /// # Arguments
    ///
    /// * `self` - The message header to be serialized.
    ///
    /// # Returns
    ///
    /// A byte vector containing the serialized message header.
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id.to_be_bytes());
        let mut flag_bytes = [0u8; 2];
        flag_bytes[0] =
            (self.qr << 7) | (self.opcode << 3) | (self.aa << 2) | (self.tc << 1) | self.rd;
        flag_bytes[1] = (self.ra << 7) | (self.z << 4) | self.rcode.to_u8();
        bytes.extend_from_slice(&flag_bytes);
        bytes.extend_from_slice(&self.qdcount.to_be_bytes());
        bytes.extend_from_slice(&self.ancount.to_be_bytes());
        bytes.extend_from_slice(&self.nscount.to_be_bytes());
        bytes.extend_from_slice(&self.arcount.to_be_bytes());
        bytes
    }

    /// Deserializes a message header from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the message header.
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized `MessageHeader` if successful, or an `std::io::Error` if deserialization fails.
    fn deserialize(data: &[u8]) -> Result<Self, std::io::Error> {
        if data.len() < 12 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Message header is too short",
            ));
        }

        let rcode_value = data[3] & 0x0F; // Extract the lower 4 bits for RCODE
        let rcode = RCode::from_u8(rcode_value).ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid RCODE value",
        ))?;

        Ok(MessageHeader {
            id: u16::from_be_bytes([data[0], data[1]]),
            qr: (data[2] >> 7) & 0x1,
            opcode: (data[2] >> 3) & 0xF,
            aa: (data[2] >> 2) & 0x1,
            tc: (data[2] >> 1) & 0x1,
            rd: data[2] & 0x1,
            ra: (data[3] >> 7) & 0x1,
            z: (data[3] >> 4) & 0x7,
            rcode,
            qdcount: u16::from_be_bytes([data[4], data[5]]),
            ancount: u16::from_be_bytes([data[6], data[7]]),
            nscount: u16::from_be_bytes([data[8], data[9]]),
            arcount: u16::from_be_bytes([data[10], data[11]]),
        })
    }
}

impl Question {
    /// Serializes a question to a byte vector.
    ///
    /// # Arguments
    ///
    /// * `self` - The question to be serialized.
    ///
    /// # Returns
    ///
    /// A byte vector containing the serialized question.
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for label in &self.qname {
            bytes.push(label.len() as u8); // Length octet
            bytes.extend_from_slice(label.as_bytes()); // Label octets
        }
        bytes.push(0); // Null byte to end QNAME
        bytes.extend_from_slice(&self.qtype.to_be_bytes());
        bytes.extend_from_slice(&self.qclass.to_be_bytes());
        bytes
    }

    /// Deserializes a question from a byte slice.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the question.
    ///
    /// * `start_offset` - The offset in the byte slice where the question starts.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple with the question and the new offset in the byte slice if successful, or an `std::io::Error` if deserialization fails.
    pub fn deserialize(data: &[u8], start_offset: usize) -> Result<(Self, usize), std::io::Error> {
        let mut offset = start_offset;
        let (qname, new_offset) = Message::parse_qname(data, offset)?;
        offset = new_offset; // Update offset to position after QNAME

        if offset + 4 > data.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Unexpected end of data",
            ));
        }

        let qtype = u16::from_be_bytes([data[offset], data[offset + 1]]);
        let qclass = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
        offset += 4; // Move past qtype and qclass

        Ok((
            Question {
                qname,
                qtype,
                qclass,
            },
            offset,
        ))
    }
}

impl ResourceRecord {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Todo
        return bytes;
    }
}

impl RCode {
    /// Converts an `RCode` to its corresponding `u8` value.
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Attempts to convert a `u8` to its corresponding `RCode` variant.
    /// Returns `None` if the value does not correspond to a valid `RCode`.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::NOERROR),
            1 => Some(Self::FORMERR),
            2 => Some(Self::SERVFAIL),
            3 => Some(Self::NXDOMAIN),
            4 => Some(Self::NOTIMP),
            5 => Some(Self::REFUSED),
            6 => Some(Self::YXDOMAIN),
            7 => Some(Self::XRRSET),
            9 => Some(Self::NOTAUTH),
            10 => Some(Self::NOTZONE),
            _ => None,
        }
    }
}