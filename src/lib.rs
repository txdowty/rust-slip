
struct SlipDatagrams {
    max_datagram_size: usize,
    datagrams: Vec<Vec<u8>>,
}

// since this is a library, we may not use all functions right away
#[allow(dead_code)]
impl SlipDatagrams {
    const END: u8 = 0xC0;
    const ESC: u8 = 0xDB;
    const ESC_END: u8 = 0xDC;
    const ESC_ESC: u8 = 0xDD;
    const MAX_DATAGRAM_SIZE: usize = 1066;

    fn new() -> Self {
        SlipDatagrams {
            datagrams: Vec::new(),
            max_datagram_size: SlipDatagrams::MAX_DATAGRAM_SIZE,
        }
    }

    pub fn datagram_count(&self) -> usize {
        self.datagrams.len()
    }

    pub fn get_datagram(&self, index: usize) -> Option<&Vec<u8>> {
        self.datagrams.get(index)
    }

    pub fn set_max_datagram_size(&mut self, size: usize) -> Result<(), String> {
        if size < 2 {
            return Err("max_datagram_size must be at least 2".to_string());
        }
        self.max_datagram_size = size;
        Ok(())
    }

    pub fn get_data_vector(&self) -> Vec<u8> {
        let mut stream: Vec<u8> = Vec::new();
        for datagram in &self.datagrams {
            stream.extend(datagram);
        }
        stream
    }

    pub fn serialize(&mut self, data: &[u8]) -> Result<(), String> {
        // datagrams are built here, then pushed to self.datagrams when either the
        // input data runs out, or the datagram hits max size - 2
        let mut datagram: Vec<u8> = Vec::new();

        for byte in data {
            match *byte {
                SlipDatagrams::END => {
                    datagram.push(SlipDatagrams::ESC);
                    datagram.push(SlipDatagrams::ESC_END);
                }
                SlipDatagrams::ESC => {
                    datagram.push(SlipDatagrams::ESC);
                    datagram.push(SlipDatagrams::ESC_ESC);
                }
                _ => {
                    datagram.push(*byte);
                }
            }
            if datagram.len() >= SlipDatagrams::MAX_DATAGRAM_SIZE - 2 {
                datagram.push(SlipDatagrams::END);
                self.datagrams.push(datagram.clone());
                datagram.clear();
            }
        }
        datagram.push(SlipDatagrams::END);
        self.datagrams.push(datagram.clone());
        Ok(())
    }

    pub fn deserialize(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        let mut output: Vec<u8> = Vec::new();
        let mut escape_seen: bool = false;
        let mut last_datagram_ended: bool = false;
        for byte in data {
            match *byte {
                SlipDatagrams::ESC => {
                    if escape_seen {
                        return Err("Invalid SLIP sequence".to_string());
                    }
                    escape_seen = true;
                    last_datagram_ended = false;
                }
                SlipDatagrams::ESC_END => {
                    if escape_seen {
                        output.push(SlipDatagrams::END);
                        escape_seen = false;
                        last_datagram_ended = false;
                    }
                }
                SlipDatagrams::ESC_ESC => {
                    if escape_seen {
                        output.push(SlipDatagrams::ESC);
                        escape_seen = false;
                        last_datagram_ended = false;
                    }
                }
                SlipDatagrams::END => {
                    self.datagrams.push(output.clone());
                    output.clear();
                    escape_seen = false;
                    last_datagram_ended = true;
                }
                _ => {
                    output.push(*byte);
                }
            }
        }
        if last_datagram_ended == false {
            return Err("Invalid SLIP sequence".to_string());
        }
        Ok(self.get_data_vector())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_test() {
        // let raw_data: &[u8] = b"\xAA\xAA\xAA\xC0";
        let raw_data: &[u8] = &[0xAA, 0xAA, 0xAA, SlipDatagrams::END];

        let mut slip_datagrams: SlipDatagrams = SlipDatagrams::new();
        slip_datagrams.set_max_datagram_size(10).unwrap();

        assert_eq!(slip_datagrams.serialize(raw_data), Ok(()));

        assert_eq!(slip_datagrams.datagram_count(), 1);
        assert_eq!(
            slip_datagrams.get_datagram(0).unwrap(),
            &[
                0xAA,
                0xAA,
                0xAA,
                SlipDatagrams::ESC,
                SlipDatagrams::ESC_END,
                SlipDatagrams::END
            ]
        );
    }

    #[test]
    fn deserialize_test() {
        let slip_data: &[u8] = &[
            0xAA,
            0xAA,
            0xAA,
            SlipDatagrams::ESC,
            SlipDatagrams::ESC_END,
            SlipDatagrams::END,
        ];

        let mut slip_datagrams: SlipDatagrams = SlipDatagrams::new();
        let raw_data: Vec<u8> = slip_datagrams.deserialize(slip_data).unwrap();
        assert_eq!(slip_datagrams.datagram_count(), 1);
        assert_eq!(raw_data, &[0xAA, 0xAA, 0xAA, 0xC0]);
    }

    #[test]
    fn deserialize_test1() {
        let slip_data: &[u8] = &[
            0xAA,
            0xAA,
            0xAA,
            0xC0,
            SlipDatagrams::ESC,
            SlipDatagrams::ESC_END,
            SlipDatagrams::END,
        ];

        let mut slip_datagrams: SlipDatagrams = SlipDatagrams::new();
        let raw_data: Vec<u8> = slip_datagrams.deserialize(slip_data).unwrap();
        assert_eq!(slip_datagrams.datagram_count(), 2);
        assert_eq!(raw_data, &[0xAA, 0xAA, 0xAA, 0xC0]);
    }

    #[test]
    fn deserialize_test2() {
        // two consecutive ESC bytes is an error
        let slip_data: &[u8] = &[
            0x55,
            0x55,
            SlipDatagrams::ESC,
            SlipDatagrams::ESC,
            0x55];

        let mut slip_datagrams: SlipDatagrams = SlipDatagrams::new();
        let raw_data: Result<Vec<u8>, String> = slip_datagrams.deserialize(slip_data);
        assert_eq!(raw_data, Err("Invalid SLIP sequence".to_string()));
    }

    #[test]
    fn deserialize_test3() {
        // no END at end is an error
        let slip_data: &[u8] = &[
            0x55,
            0x55,
            0x55,
            0x55,
        ];

        let mut slip_datagrams: SlipDatagrams = SlipDatagrams::new();
        let raw_data: Result<Vec<u8>, String> = slip_datagrams.deserialize(slip_data);
        assert_eq!(raw_data, Err("Invalid SLIP sequence".to_string()));
    }
}
